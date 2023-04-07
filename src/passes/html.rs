//! HTML backend
use crate::{event::Event, passes::CompilingPass};
use chrono::Timelike;
use std::{cmp::min, str::FromStr};
use thiserror::Error;

/// Backend outputing events to a standalone HTML document containing a timetable
pub struct HTMLBackend {}

/// A Trait for converting a value to a [`String`] HTML representation
pub trait ToHTML {
    /// Convert the given value to a [`String`] HTML representation
    fn to_html(&self) -> String;
}
/// Options for the HTML backend
pub struct HTMLBackendOptions {
    /// Path to the template file. If not set, the default template (`data/template.html`) will be used.
    pub template_path: Option<String>,
}

/// Error that can occur during the compilation of the HTML backend
#[derive(Debug, Error)]
pub enum HTMLBackendCompilationError {
    /// The event could not be parsed.
    #[error(transparent)]
    CouldNotParseEvent(#[from] <Event as FromStr>::Err),
    /// An error occurred while trying to read the template file
    #[error("Error while trying to read the template file: {0}")]
    CouldNotReadTemplate(#[from] std::io::Error),
}

#[allow(clippy::option_if_let_else)]
fn get_template(template_path: Option<String>) -> Result<String, std::io::Error> {
    match template_path {
        None => Ok(include_str!("../../data/template.html").to_string()),
        Some(path) => std::fs::read_to_string(path),
    }
}

impl ToHTML for Event {
    fn to_html(&self) -> String {
        let duration = std::cmp::max(self.duration / 30, 1);
        let class = self.event_type.to_string();
        let mut res = format!("\t\t<td class=\"{class}\" rowspan=\"{duration}\">");
        let (title, _) = self.title.split_at(min(self.title.len(), 25));
        res.push_str(title);
        res.push_str("<span>");
        for speaker in &self.speakers {
            res.push_str(speaker.as_str());
        }
        res.push_str("</span></td>\n");
        res
    }
}

impl CompilingPass<Vec<Event>> for HTMLBackend {
    type Residual = String;
    type Error = HTMLBackendCompilationError;
    fn apply(events: Vec<Event>) -> Result<Self::Residual, Self::Error> {
        Self::apply_with(
            events,
            HTMLBackendOptions {
                template_path: None,
            },
        )
    }
}

impl CompilingPass<Vec<Event>, HTMLBackendOptions> for HTMLBackend {
    type Residual = String;
    type Error = HTMLBackendCompilationError;

    fn apply(events: Vec<Event>) -> Result<Self::Residual, Self::Error> {
        Self::apply_with(
            events,
            HTMLBackendOptions {
                template_path: None,
            },
        )
    }

    fn apply_with(
        events: Vec<Event>,
        options: HTMLBackendOptions,
    ) -> Result<Self::Residual, Self::Error> {
        let template = get_template(options.template_path)?;
        let mut str = String::new();
        str.push_str("<table><tbody>\n");
        for i in 9..21 {
            str.push_str(format!("\t<tr><th>{i}:00</th>").as_str());
            for event in events
                .iter()
                .filter(|ev| ev.start_date.hour() >= i && ev.start_date.hour() < i + 1)
            {
                str.push_str(event.to_html().as_str());
            }
            str.push_str("</tr>\n");
            str.push_str(format!("\t<tr><th class=\"light\">{i}:30</th></tr>\n").as_str());
        }
        str.push_str("</tbody></table>\n");
        Ok(template.replace("{{ CALENDAR }}", &str))
    }
}
