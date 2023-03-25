//! Specification of a timetable event

use std::collections::HashMap;
use std::str::FromStr;
use std::fmt;

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

impl FromStr for Type {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "talk" => Ok(Self::Talk),
            "meal" => Ok(Self::Meal),
            "break" => Ok(Self::Break),
            "fun" => Ok(Self::Fun),
            _ => Err(()),
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
    /// The event description
    pub description: String,
}

/// Split header (cf grammar)
fn split_pairs(string: &str) -> HashMap<&str, &str> {
    string
        .split('\n')
        .map(|s| s.split_at(s.find(':').unwrap()))
        .map(|(key, val)| (key, val[1..].trim()))
        .collect()
}

impl FromStr for Event {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if let Some((header, description)) = trimmed.split_once("\n\n") {
            let settings = split_pairs(header);
            let event_type = settings.get("type")
                .as_ref()
                .map_or(Type::Talk, |e| Type::from_str(e).unwrap_or_default());
            let title = settings.get("title").map_or("(no title)", |&e| e);

            Ok(Self {
                event_type,
                title: title.to_owned(),
                description: description.to_owned(),
            })
        } else {
            Err(())
        }
    }
}
