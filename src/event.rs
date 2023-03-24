//! Specification of a timetable event

use std::collections::HashMap;
use std::str::FromStr;

/// The type of a timetable event
#[derive(Debug, PartialEq, Default)]
enum EventType {
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

impl FromStr for EventType {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "talk" => Ok(EventType::Talk),
            "meal" => Ok(EventType::Meal),
            "break" => Ok(EventType::Break),
            "fun" => Ok(EventType::Fun),
            _ => Err(()),
        }
    }
}

/// A timetable event
#[derive(Debug)]
pub struct Event {
    /// The type of the event
    event_type: EventType,
    /// The event description
    description: String,
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
            // println!("{:#?}", settings);
            let event_type = if let Some(e) = &settings.get("type") {
                EventType::from_str(e).unwrap_or_default()
            } else {
                EventType::Talk
            };

            Ok(Event {
                event_type,
                description: description.to_owned(),
            })
        } else {
            Err(())
        }
    }
}
