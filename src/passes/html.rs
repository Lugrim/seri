//! HTML backend
use crate::{event::Event, passes::CompilingPass};
use chrono::Timelike;
use std::{cmp::min, str::FromStr, sync::Mutex};
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

/// Options of the HTML backend
static OPTIONS: Mutex<HTMLBackendOptions> = Mutex::new(HTMLBackendOptions {
    template_path: None,
});

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

fn get_template() -> Result<String, std::io::Error> {
    let template_path = OPTIONS.lock().unwrap().template_path.clone();
    match template_path.as_ref() {
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

impl HTMLBackend {
    /// Configure the HTML backend
    ///
    /// # Arguments
    ///
    /// * `opts` - The options to use, see [`HTMLBackendOptions`]
    ///
    pub fn configure(opts: HTMLBackendOptions) {
        *OPTIONS.lock().unwrap() = opts;
    }
}

impl CompilingPass<Vec<Event>> for HTMLBackend {
    type Residual = String;
    type Error = HTMLBackendCompilationError;

    fn apply(events: Vec<Event>) -> Result<Self::Residual, Self::Error> {
        let template = get_template()?;
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
