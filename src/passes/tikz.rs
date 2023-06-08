//! Latex backends

use std::str::FromStr;

use chrono::{DateTime, Datelike, Duration, Local, Timelike};

use crate::{
    event::{find_bounding_box, Event, InvalidDatetime},
    passes::CompilingPass,
    templating,
};

/// Backend outputing events to a standalone LaTeX document containing a `TikZ` timetable
pub struct Pass {}

/// Options for the `TikZ` backend
pub struct Options {
    /// Path to the template file. If not set, the default template (`data/template_tikz.tex`) will be used.
    pub template_path: Option<String>,
}

/// Error occuring when compiling an event list to `TikZ`.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The event could not be parsed.
    #[error(transparent)]
    CouldNotParseEvent(#[from] <Event as FromStr>::Err),
    /// The list of events was empty.
    #[error("no event was provided")]
    NoEventProvided,
    /// The datetime of either the first day or last day of the bounding box is not valid.
    #[error(transparent)]
    InvalidDatetime(#[from] InvalidDatetime),
    /// An error occurred while trying to read the template file
    #[error("Error while trying to read the template file: {0}")]
    CouldNotReadTemplate(#[from] std::io::Error),
    /// An error occurred while trying to replace text in the template
    #[error("Error while trying to replace in template file: {0}")]
    CouldNotReplaceTemplate(#[from] templating::Error),
}

#[allow(clippy::option_if_let_else)]
fn get_template(template_path: Option<String>) -> Result<String, std::io::Error> {
    match template_path {
        None => Ok(include_str!("../../data/template_tikz.tex").to_string()),
        Some(path) => std::fs::read_to_string(path),
    }
}

/// Generate a \foreach macro to iterate over each hour of the calendar
fn foreach_hour(first_hour: u32, last_hour: u32) -> String {
    r"
    \foreach \time   [evaluate=\time] in "
        .to_owned()
        + &format!("{{{first_hour},...,{last_hour}}}")
}

/// Generate horizontal line on each hour of `day count` width
fn hour_dividers(first_hour: u32, last_hour: u32, day_count: u32) -> String {
    // Draw the horizontal dividers on hours
    foreach_hour(first_hour, last_hour)
        + r"
        \draw (1,\time) -- ("
        + &format!("{}", day_count + 1)
        + r", \time);"
}

/// Generate hour marks "hh:00" for each hour
fn hour_marks(first_hour: u32, last_hour: u32) -> String {
    // For each hour, write it on the left
    foreach_hour(first_hour, last_hour)
        + r"
        \node[anchor=east] at (1,\time) {\time:00};"
}

/// Generate a tikz node in the calendar for a given event
fn tikz_node(e: &Event, up_left_day: u32) -> String {
    r"
    \node[".to_owned()
        // declare the event type for the format
        + &format!("{}", e.event_type)
        + "={"
        // Compute event length as an hour fraction (block height)
        + &format!("{:.2}", f64::from(e.duration) / 60.)
        + "}{"
        + "1" // TODO Compute simultaneous event count
        + "}] at ("
        // Compute beginning day number (x position)
        + &format!("{}", e.start_date.day() - up_left_day + 1)
        + ","
        // Compute beginning hour (y position)
        + &format!(
            "{}.{:02}",
            e.start_date.format("%H"),
            e.start_date.minute() * 5 / 3
        )
        + ") {"
        // Create the string to fill up the event block
        + &e.short_text()
        + "};"
}

/// Draw the vertical dividers between days
fn day_dividers(first_hour: u32, last_hour: u32, day_count: u32) -> String {
    r"
    % Draw some day dividers.
    \foreach \day   [evaluate=\day] in "
        .to_owned()
        + &format!("{{1,...,{}}}", day_count + 1)
        + r"
        \draw (\day,"
        + &format!("{}", first_hour - 1)
        + r") -- (\day,"
        + &format!("{last_hour}")
        + ");"
}

/// Generate the date headers at the top of the columns
fn date_headers(first_hour: u32, day_count: u32, up_left: DateTime<Local>) -> String {
    let mut r = String::new();
    // Display the date headers
    for i in 0..day_count {
        let col = i + 1;
        r += r"
    \node[anchor=south] at (";
        r += &format!("{col}");
        r += r".5, ";
        r += &format!("{}", first_hour - 1);
        r += ".5) {";
        r += &format!(
            "{}",
            (up_left + Duration::days(i64::from(i))).format("%A, %B %e")
        );
        r += "};";
    }
    r
}

impl CompilingPass<Vec<Event>> for Pass {
    type Residual = String;
    type Error = Error;

    fn apply(events: Vec<Event>) -> Result<Self::Residual, Self::Error> {
        Self::apply_with(
            events,
            Options {
                template_path: None,
            },
        )
    }
}

impl CompilingPass<Vec<Event>, Options> for Pass {
    type Residual = String;
    type Error = Error;

    fn apply(events: Vec<Event>) -> Result<Self::Residual, Self::Error> {
        Self::apply_with(
            events,
            Options {
                template_path: None,
            },
        )
    }

    // TODO Programmatically generate formats (tikzset)?
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn apply_with(events: Vec<Event>, options: Options) -> Result<Self::Residual, Self::Error> {
        let template = get_template(options.template_path)?;
        // Get the bounding box to adjust the timetable shown (hours and days)
        let bb = find_bounding_box(&events).ok_or(Error::NoEventProvided)?;

        let first_hour = bb.up_left.hour();

        let last_hour = bb.down_right.hour() + u32::from(bb.down_right.minute() != 0);

        let day_count = ((bb.last_day()? - bb.first_day()?).num_days() + 1) as u32;

        let mut r = hour_marks(first_hour, last_hour);
        r += &hour_dividers(first_hour, last_hour, day_count);

        r += &day_dividers(first_hour, last_hour, day_count);
        r += &date_headers(first_hour, day_count, bb.up_left);

        // Display all our event nodes
        for e in events {
            r += &tikz_node(&e, bb.up_left.day());
        }

        Ok(templating::replace(&template, "CALENDAR", &r)?)
    }
}
