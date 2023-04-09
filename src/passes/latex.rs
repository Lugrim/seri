//! Latex frontends

use std::str::FromStr;

use chrono::{DateTime, Datelike, Duration, Local, Timelike};
use thiserror::Error;

use crate::{
    event::{find_bounding_box, Event, InvalidDatetime},
    passes::CompilingPass,
};

/// Backend outputing events to a standalone LaTeX document containing a Tikz timetable
pub struct TikzBackend {}

/// Error occuring when compiling an event list to tikz.
#[derive(Debug, Error)]
pub enum TikzBackendCompilationError {
    /// The event could not be parsed.
    #[error(transparent)]
    CouldNotParseEvent(#[from] <Event as FromStr>::Err),
    #[error("no event was provided")]
    /// The list of events was empty.
    NoEventProvided,
    #[error(transparent)]
    /// The datetime of either the first day or last day of the bounding box is not valid.
    InvalidDatetime(#[from] InvalidDatetime),
}

// TODO Load this string from file or config
// TODO Programmatically generate formats (tikzset)?
const LATEX_INTRO: &str = r"\documentclass{standalone}
\usepackage{tikz}

\begin{document}

% These set the width of a day and the height of an hour.
\newcommand*\daywidth{6cm}
\newcommand*\hourheight{4em}

% The entry style will have two options:
% * the first option sets how many hours the entry will be (i.e. its height);
% * the second option sets how many overlapping entries there are (thus
%   determining the width).
\tikzset{entry/.style 2 args={
    xshift=(0.5334em+0.8pt)/2,
    draw,
    line width=0.8pt,
    font=\sffamily,
    rectangle,
    rounded corners,
    fill=blue!20,
    anchor=north west,
    inner sep=0.3333em,
    text width={\daywidth/#2-1.2em-1.6pt},
    minimum height=#1*\hourheight,
    align=center
}}

\tikzset{talk/.style 2 args={
		entry={#1}{#2},
		fill=red!40
	}
}

\tikzset{meal/.style 2 args={
		entry={#1}{#2},
		fill=green!40
	}
}

\tikzset{fun/.style 2 args={
		entry={#1}{#2},
		fill=blue!40
	}
}

\tikzset{transport/.style 2 args={
		entry={#1}{#2},
		fill=gray!20
	}
}

% Start the picture and set the x coordinate to correspond to days and the y
% coordinate to correspond to hours (y should point downwards).
\begin{tikzpicture}[y=-\hourheight,x=\daywidth]

    % First print a list of times.";

fn speaker_string(e: &Event) -> String {
    match e.speakers.len() {
        0 => {
            let (short, overflow) = e.title.split_at(std::cmp::min(25, e.title.len()));
            let rest = if overflow.is_empty() { "" } else { "..." };
            short.to_owned() + rest
        }
        1 => e.speakers[0].clone(),
        _ => format!("{} et~al.", e.speakers[0]),
    }
}

fn generate_foreach_hour(first_hour: u32, last_hour: u32) -> String {
    r"
    \foreach \time   [evaluate=\time] in "
        .to_owned()
        + &format!("{{{first_hour},...,{last_hour}}}")
}
fn generate_hour_dividers(first_hour: u32, last_hour: u32, day_count: u32) -> String {
    // Draw the horizontal dividers on hours
    generate_foreach_hour(first_hour, last_hour)
        + r"
        \draw (1,\time) -- ("
        + &format!("{}", day_count + 1)
        + r", \time);"
}

fn generate_hour_marks(first_hour: u32, last_hour: u32) -> String {
    // For each hour, write it on the left
    generate_foreach_hour(first_hour, last_hour)
        + r"
            \node[anchor=east] at (1,\time) {\time:00};"
}

fn generate_tikz_node(e: &Event, up_left_day: u32) -> String {
    let mut r = r"\node[".to_owned();
    // declare the event type for the format
    r += &format!("{}", e.event_type);
    r += "={";
    // Compute event length as an hour fraction (block height)
    r += &format!("{:.2}", f64::from(e.duration) / 60.);
    r += "}{";
    r += "1"; // TODO Compute simultaneous event count
    r += "}] at (";
    // Compute beginning day number (x position)
    r += &format!("{}", e.start_date.day() - up_left_day + 1);
    r += ",";
    // Compute beginning hour (y position)
    r += &format!(
        "{}.{:.2}",
        e.start_date.format("%H"),
        e.start_date.minute() * 5 / 3
    );
    r += ") {";
    // Create the string to fill up the event block
    r += &speaker_string(e);
    r += "};";
    r
}

fn generate_day_dividers(first_hour: u32, last_hour: u32, day_count: u32) -> String {
    // Draw the vertical dividers between days
    let mut r = r"
    % Draw some day dividers.
    \foreach \day   [evaluate=\day] in "
        .to_owned();
    r += &format!("{{1,...,{}}}", day_count + 1);

    r += r"
        \draw (\day,";
    r += &format!("{}", first_hour - 1);
    r += r") -- (\day,";
    r += &format!("{last_hour}");
    r += ");";
    r
}

fn generate_date_headers(first_hour: u32, day_count: u32, up_left: DateTime<Local>) -> String {
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
        r += r"};";
    }
    r
}

fn generate_postamble() -> String {
    r"
\end{tikzpicture}
\end{document}"
        .to_owned()
}

impl CompilingPass<Vec<Event>> for TikzBackend {
    type Residual = String;
    type Error = TikzBackendCompilationError;

    // TODO: Split this huge function into smaller ones.
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn apply(events: Vec<Event>) -> Result<Self::Residual, Self::Error> {
        // Get the bounding box to adjust the timetable shown (hours and days)
        let bb = find_bounding_box(&events).ok_or(TikzBackendCompilationError::NoEventProvided)?;

        let first_hour = bb.up_left.hour();

        // TODO Do not add 1 if the hour is an integer
        let last_hour = bb.down_right.hour() + u32::from(bb.down_right.minute() != 0);

        let day_count = ((bb.last_day()? - bb.first_day()?).num_days() + 1) as u32;

        // Create the return string
        let mut r: String = LATEX_INTRO.to_owned();

        r += &generate_hour_marks(first_hour, last_hour);
        r += &generate_hour_dividers(first_hour, last_hour, day_count);

        r += &generate_day_dividers(first_hour, last_hour, day_count);
        r += &generate_date_headers(first_hour, day_count, bb.up_left);

        // Display all our event nodes
        for e in events {
            r += &generate_tikz_node(&e, bb.up_left.day());
        }

        r += &generate_postamble();

        Ok(r)
    }
}
