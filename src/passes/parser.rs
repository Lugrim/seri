use crate::{
    passes::CompilingPass,
    event::Event
};

use std::str::FromStr;

pub struct ParseTimetable {
}

impl CompilingPass<&str, Vec<Event>, ()> for ParseTimetable {
    fn apply(s: &str) -> Result<Vec<Event>, ()> {
    Ok(s
        .split("---")
        .filter_map(|e| Event::from_str(e).ok())
        .collect())
    }
}
