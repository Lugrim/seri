//! Specification of a timetable event

use chrono::prelude::*;
use chrono::{DateTime, Duration, Local};
use isolang::Language;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// The type of a timetable event
#[derive(Debug, PartialEq, Eq, Default, Clone)]
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
#[derive(Debug, Clone)]
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
    /// The language of the talk
    pub language: Option<Language>,
    /// The list of declared speakers
    pub speakers: Vec<String>,
}

/// Cut a text to be at most `length` characters
/// If `length < 3`, will actually be `length + 3` characters
fn cut_text(text: &str, length: usize) -> String {
    let text = text.chars();
    if text.clone().count() <= length {
        text.collect::<String>()
    } else {
        if length > 3 { // If possible, take the exact given character count
            text.take(length - 3)
        } else { // Otherwise, take 3 characters more
            text.take(length)
        }.collect::<String>() + "..."
    }
}

impl Event {
    /// Generate a short version of the title, up to `length` characters
    #[must_use]
    pub fn short_title(&self, length: usize) -> String {
        cut_text(&self.title, length)
    }

    /// Generate a long version of all the speakers
    #[must_use]
    pub fn speakers_string(&self) -> String {
        self.speakers.join(", ")
    }

    /// Generate the text content of an event in the calendar.
    /// For now, if speakers of an event are given, will print the first one (eventually succeeded by
    /// `et~al.` if there are more) or the title, eventually truncated to 25 characters
    #[must_use]
    pub fn short_text(&self) -> String {
        match self.event_type {
            Type::Talk => match self.speakers.len() {
                0 => self.short_title(30),
                1 => self.speakers[0].clone(),
                2 => format!("{} and {}", self.speakers[0], self.speakers[1]),
                _ => format!("{} et~al.", self.speakers[0]),
            },
            _ => self.short_title(30),
        }
    }
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

        let language = settings.get("lang").and_then(|l| Language::from_639_1(l));

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
            l.replace(['[', ']'], "")
                .split(',')
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
                .collect()
        });

        let mut nonempty_description: Option<String> = description.map(|d| d.trim().into());
        if let Some(d) = &nonempty_description {
            if d.is_empty() {
                nonempty_description = None;
            }
        }

        Ok(Self {
            event_type,
            start_date,
            duration,
            title: title.to_owned(),
            description: nonempty_description,
            language,
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

    /// Get how many days the event is lasting
    #[must_use]
    pub fn nb_days(&self) -> u32 {
        let duration = self.down_right.day() - self.up_left.day();
        duration + 1
    }

    fn boundary(
        order: std::cmp::Ordering,
        first: &DateTime<Local>,
        second: &DateTime<Local>,
    ) -> Option<DateTime<Local>> {
        let mut res = *first;

        if second.time().cmp(&first.time()) == order {
            res = res.with_hour(second.hour())?.with_minute(second.minute())?;
        }

        if second.date_naive().cmp(&first.date_naive()) == order {
            res = res
                .with_day(second.day())?
                .with_month(second.month())?
                .with_year(second.year())?;
        }

        Some(res)
    }

    /// Create a datetime that represent the day of the earlier of both datetimes and at the time of
    /// day of the earlier of them.
    ///
    /// # Example
    ///
    /// If first is the November, 06 at 9:00 and the second is November, 08 at 8:00, the result is
    /// the November, 06 at 8:00.
    #[must_use]
    pub fn top_left_boundary(
        first: &DateTime<Local>,
        second: &DateTime<Local>,
    ) -> Option<DateTime<Local>> {
        Self::boundary(std::cmp::Ordering::Less, first, second)
    }

    /// Create a datetime that represent the day of the later of both datetimes and at the time of
    /// day of the later of them.
    ///
    /// # Example
    ///
    /// If first is the November, 06 at 9:00 and the second is November, 08 at 8:00, the result is
    /// the November, 08 at 9:00.
    #[must_use]
    pub fn bottom_right_boundary(
        first: &DateTime<Local>,
        second: &DateTime<Local>,
    ) -> Option<DateTime<Local>> {
        Self::boundary(std::cmp::Ordering::Greater, first, second)
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

        up_left = BoundingBox::top_left_boundary(&up_left, &e.start_date)?;
        down_right = BoundingBox::bottom_right_boundary(&down_right, &end_of_event)?;
    }

    Some(BoundingBox {
        up_left,
        down_right,
    })
}

#[cfg(test)]
fn create_empty_datetime() -> DateTime<Local> {
    NaiveDate::from_ymd_opt(0, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap()
}
#[test]
fn test_number_days() {
    // Test for an event on the same day (0h->18h)
    let dur = create_empty_datetime().with_hour(18).unwrap();
    let bb = BoundingBox {
        down_right: dur,
        up_left: create_empty_datetime(),
    };
    assert!(bb.nb_days() == 1);

    // Test for an event spanning on two days (1 January 0h -> 2 January 18h)
    let dur = create_empty_datetime()
        .with_day(2)
        .unwrap()
        .with_hour(18)
        .unwrap();
    let bb = BoundingBox {
        down_right: dur,
        up_left: create_empty_datetime(),
    };
    assert!(bb.nb_days() == 2);
}
