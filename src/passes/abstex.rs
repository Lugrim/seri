//! `LaTeX` abstracts backend

use std::str::FromStr;

use chrono::{DateTime, Days, Local};

use crate::{
    event::{find_bounding_box, Event, InvalidDatetime, Type},
    passes::CompilingPass,
    templating,
};

/// Backend outputing events to a standalone LaTeX document containing a `LaTeX` abstracts
pub struct Pass {}

/// Options for the `LaTeX` abstracts backend
pub struct Options {
    /// Path to the template file. If not set, the default template (`data/template_abstex.tex`) will be used.
    pub template_path: Option<String>,
}

/// Error occuring when compiling an event list to `LaTeX` abstracts.
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
        None => Ok(include_str!("../../data/template_abstex.tex").to_string()),
        Some(path) => std::fs::read_to_string(path),
    }
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

fn day_delimiter(day: &DateTime<Local>) -> String {
    format!(r"\section{{{}}}", day.format("%A, %B %e"))
}

fn talk_title(e: &Event) -> String {
    let mut r = r"\subsection{".to_owned();
    r += &talk_language(e);
    r += &e.title;
    r += "}\n";
    r
}

fn talk_subtitle(e: &Event) -> String {
    let mut r = r"\paragraph{} \textit{".to_owned();
    r += &format!("{}", e.start_date.time().format("%H:%M"));
    if !e.speakers.is_empty() {
        r += &format!(" - {}", e.speakers.join(r", "));
    }
    r += "}\n";
    r
}

fn talk_language(e: &Event) -> String {
    e.language
        .map_or_else(String::new, |l| "\\".to_owned() + l.to_639_3() + " ")
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

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn apply_with(mut events: Vec<Event>, options: Options) -> Result<Self::Residual, Self::Error> {
        // let mut events = events.clone();
        events.sort_by_key(|e| e.start_date);
        let template = get_template(options.template_path)?;

        let mut r = String::new();

        // Get the bounding box to get the ranges
        let bb = find_bounding_box(&events).ok_or(Error::NoEventProvided)?;

        let mut day = bb.first_day()? - Days::new(1);

        // Display all our event
        for e in events {
            if e.start_date.date_naive() > day.date_naive() {
                day = e.start_date;
                r += &day_delimiter(&day);
            }
            match &e.event_type {
                Type::Talk | Type::Fun => {
                    r += &talk_title(&e);
                    r += &talk_subtitle(&e);
                    r += &e
                        .description
                        .map(|d| r"\paragraph{} ".to_owned() + &d)
                        .unwrap_or_default();
                    r += "\n\n";
                }
                _ => (),
            }
        }

        let t = templating::replace(
            &template,
            "BEGIN_DATE",
            &format!("{}", bb.first_day()?.format("%A, %B %e")),
        )?;
        let t = templating::replace(
            &t,
            "END_DATE",
            &format!("{}", bb.last_day()?.format("%A, %B %e")),
        )?;
        Ok(templating::replace(&t, "ABSTRACTS", &r)?)
    }
}
