//! Specification of a timetable event

use chrono::prelude::*;
use chrono::{DateTime, Duration, Local};
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
    /// Transportation (car, bus...)
    Transport,
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
            "transport" => Ok(Self::Transport),
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
            Self::Transport => write!(f, "transport"),
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
    pub description: Option<String>,
    /// The list of declared speakers
    pub speakers: Vec<String>,
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

    /// the given date does not respect the expected format.
    #[error("the give date `{0}` does not respect the expected format: `%Y-%m-%d %H:%M`")]
    InvalidDateShape(String),
}

impl FromStr for Event {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        let split_result = trimmed.split_once("\n\n");

        let (header, description) = if let Some((first, second)) = split_result {
            let text = second.trim();
            (first.trim(), (!text.is_empty()).then(|| text.to_owned()))
        } else {
            (trimmed, None)
        };

        let settings = split_pairs(header)?;

        let event_type = settings
            .get("type")
            .as_ref()
            .map_or(Ok(Type::Talk), |talk_type| {
                Type::from_str(talk_type).map_err(ParsingError::from)
            })?;

        let title = settings.get("title").map_or("(no title)", |&e| e);

        let date_name = String::from("date");
        let start_date = settings
            .get(date_name.as_str())
            .ok_or(ParsingError::SettingNotFound { name: date_name })
            .and_then(|datetime| {
                Local
                    .datetime_from_str(datetime, "%Y-%m-%d %H:%M")
                    .map_err(|_| ParsingError::InvalidDateShape((*datetime).to_string()))
            })?;

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
        let speakers = settings.get("speakers").map_or_else(Vec::new, |l| {
            l.split(',')
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
                .collect()
        });

        Ok(Self {
            event_type,
            start_date,
            duration,
            title: title.to_owned(),
            description,
            speakers,
        })
    }
}

/// Bounding box of event.
///
/// This structure contains datetimes that allows to draw a box containing all the events from which
/// it was built in a calendar view.
pub struct BoundingBox {
    /// Upper left point
    pub up_left: DateTime<Local>,
    /// Lower right point
    pub down_right: DateTime<Local>,
}

/// The datetime is not valid.
#[derive(Debug, Error)]
#[error("the datetime created is invalid")]
pub struct InvalidDatetime;

impl BoundingBox {
    /// Get a datetime of the first day at 00:00 of the bounding box.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidDatetime`] if it is not possible to build the datetime.
    pub fn first_day(&self) -> Result<DateTime<Local>, InvalidDatetime> {
        self.up_left
            .with_hour(0)
            .and_then(|dt| dt.with_minute(0))
            .ok_or(InvalidDatetime)
    }

    /// Get a datetime of the last day at 00:00 of the bounding box
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidDatetime`] if it is not possible to build the datetime.
    pub fn last_day(&self) -> Result<DateTime<Local>, InvalidDatetime> {
        self.down_right
            .with_hour(0)
            .and_then(|dt| dt.with_minute(0))
            .ok_or(InvalidDatetime)
    }

    /// Create a datetime that represent the day of the earlier of both datetimes and at the time of
    /// day of the earlier of them.
    ///
    /// # Example
    ///
    /// If first is the November, 06 at 9:00 and the second is November, 08 at 8:00, the result is
    /// the November, 06 at 8:00.
    #[must_use]
    pub fn most_top_left(
        first: &DateTime<Local>,
        second: &DateTime<Local>,
    ) -> Option<DateTime<Local>> {
        let mut res = *first;

        if second.time() < first.time() {
            res = res.with_hour(second.hour())?.with_minute(second.minute())?;
        }

        if second.date_naive() < first.date_naive() {
            res = res
                .with_day(second.day())?
                .with_month(second.month())?
                .with_year(second.year())?;
        }

        Some(res)
    }

    /// Create a datetime that represent the day of the later of both datetimes and at the time of
    /// day of the later of them.
    ///
    /// # Example
    ///
    /// If first is the November, 06 at 9:00 and the second is November, 08 at 8:00, the result is
    /// the November, 08 at 9:00.
    #[must_use]
    pub fn most_bottom_right(
        first: &DateTime<Local>,
        second: &DateTime<Local>,
    ) -> Option<DateTime<Local>> {
        let mut res = *first;

        if first.time() < second.time() {
            res = res.with_hour(second.hour())?.with_minute(second.minute())?;
        }

        if first.date_naive() < second.date_naive() {
            res = res
                .with_day(second.day())?
                .with_month(second.month())?
                .with_year(second.year())?;
        }

        Some(res)
    }
}

/// TODO Do not assume everything is in the same month
/// Will find the bounding box (date, times) to generate a timetable
#[must_use]
pub fn find_bounding_box(events: &Vec<Event>) -> Option<BoundingBox> {
    let first = events.get(0)?;
    let mut up_left = first.start_date;
    let mut down_right = first.start_date;

    for e in events {
        let end_of_event = e.start_date + Duration::minutes(i64::from(e.duration));

        up_left = BoundingBox::most_top_left(&up_left, &e.start_date)?;
        down_right = BoundingBox::most_bottom_right(&down_right, &end_of_event)?;
    }

    Some(BoundingBox {
        up_left,
        down_right,
    })
}
