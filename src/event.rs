//! Specification of a timetable event

use chrono::prelude::*;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// The type of a timetable event
#[derive(Debug, PartialEq, Eq, Default)]
pub enum Type {
    /// A Talk by someone
    #[default]
    Talk,
    /// A Meal (we like food)
    Meal,
    /// A Coffee Break
    Break,
    /// Fun time!
    Fun,
}

/// The type of talk provided is not valid.
#[derive(Debug, Error)]
#[error("`{0} is not a valid type of talk`")]
pub struct InvalidTalkType(pub String);

impl FromStr for Type {
    type Err = InvalidTalkType;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "talk" => Ok(Self::Talk),
            "meal" => Ok(Self::Meal),
            "break" => Ok(Self::Break),
            "fun" => Ok(Self::Fun),
            tt => Err(InvalidTalkType(tt.to_owned())),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Talk => write!(f, "talk"),
            Self::Meal => write!(f, "meal"),
            Self::Break => write!(f, "break"),
            Self::Fun => write!(f, "fun"),
        }
    }
}

/// A timetable event
#[derive(Debug)]
pub struct Event {
    /// The type of the event
    pub event_type: Type,
    /// The title of the event
    pub title: String,
    /// The beginning of the event
    pub start_date: DateTime<Local>,
    /// The duration of the event (in minutes)
    pub duration: u32,
    /// The event description
    pub description: String,
}

/// The line of configuration given by the user is not a valid "key:value" pair.
#[derive(Debug, Error)]
#[error("line `{0}` is not a valid field")]
pub struct InvalidField(pub String);

/// Split header (cf grammar)
fn split_pairs(string: &str) -> Result<HashMap<&str, &str>, InvalidField> {
    string
        .split('\n')
        .map(|s| {
            s.find(':')
                .ok_or_else(|| InvalidField(s.to_owned()))
                .map(|pos| s.split_at(pos))
        })
        .map(|field| field.map(|(key, val)| (key, val[1..].trim())))
        .collect()
}

/// The parsing of an event failed.
#[derive(Debug, Error)]
pub enum ParsingError {
    /// The duration setting could not be parsed as an integer.
    #[error("could not parse duration: `{source}`")]
    CouldNotParseDuration {
        /// the underlying error
        #[source]
        source: <u32 as FromStr>::Err,
    },

    /// No setting named `name` was found in the input.
    #[error("setting named `{name}` not found")]
    SettingNotFound {
        /// the setting name
        name: String,
    },

    /// The input did not have empty-newline separated header and description.
    #[error("could not split the header and description of the event")]
    CouldNotSplit,

    /// The type of talk provided by the user is not valid.
    #[error(transparent)]
    InvalidTalkType(#[from] InvalidTalkType),

    /// a line of configuration given as input is not a valid "key:value" pair.
    #[error(transparent)]
    InvalidField(#[from] InvalidField),
}

impl FromStr for Event {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if let Some((header, description)) = trimmed.split_once("\n\n") {
            let settings = split_pairs(header)?;

            let event_type = settings
                .get("type")
                .as_ref()
                .map_or(Ok(Type::Talk), |talk_type| {
                    Type::from_str(talk_type).map_err(ParsingError::from)
                })?;

            let title = settings.get("title").map_or("(no title)", |&e| e);
            let start_date = Local
                .datetime_from_str(
                    settings.get("date").expect("No `date` field found"),
                    "%Y-%m-%d %H:%M",
                )
                .expect("Invalid date shape.");

            let duration_name = String::from("duration");
            let duration = settings
                .get(duration_name.as_str())
                .ok_or(ParsingError::SettingNotFound {
                    name: duration_name,
                })
                .and_then(|duration_setting| {
                    duration_setting
                        .parse()
                        .map_err(|err| ParsingError::CouldNotParseDuration { source: err })
                })?;

            Ok(Self {
                event_type,
                start_date,
                duration,
                title: title.to_owned(),
                description: description.to_owned(),
            })
        } else {
            Err(ParsingError::CouldNotSplit)
        }
    }
}
