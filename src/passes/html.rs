//! HTML backend
use crate::{event::Event, passes::CompilingPass};
use chrono::Timelike;
use std::{cmp::min, str::FromStr, sync::Mutex};
use thiserror::Error;

/// Backend outputing events to a standalone HTML document containing a timetable
pub struct HTMLBackend {}

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
    /// The event could not be parsed.
    #[error("Error while trying to read the template file: {0}")]
    CouldNotReadTemplate(#[from] std::io::Error),
}

fn get_template() -> Result<String, HTMLBackendCompilationError> {
    let options = OPTIONS.lock().unwrap();
    if let Some(path) = &options.template_path {
        match std::fs::read_to_string(path) {
            Ok(template) => Ok(template),
            Err(err) => Err(HTMLBackendCompilationError::CouldNotReadTemplate(err)),
        }
    } else {
        Ok(include_str!("../../data/template.html").to_string())
    }
}

fn event_to_string(event: &Event) -> String {
    let duration = std::cmp::max(event.duration / 30, 1);
    let class = event.event_type.to_string();
    let mut res = String::from(format!("\t\t<td class=\"{class}\" rowspan=\"{duration}\">"));
    let (title, _) = event.title.split_at(min(event.title.len(), 25));
    res.push_str(format!("{}", title).as_str());
    res.push_str("<span>");
    for speaker in &event.speakers {
        res.push_str(speaker.as_str());
    }
    res.push_str("</span></td>\n");
    res
}

impl HTMLBackend {
    /// Configure the HTML backend
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
            str.push_str(format!("\t<tr><th>{}:00</th>", i).as_str());
            for event in events
                .iter()
                .filter(|ev| ev.start_date.hour() >= i && ev.start_date.hour() < i + 1)
            {
                str.push_str(event_to_string(&event).as_str());
            }
            str.push_str("</tr>\n");
            str.push_str(format!("\t<tr><th class=\"light\">{}:30</th></tr>\n", i).as_str());
        }
        str.push_str("</tbody></table>\n");
        Ok(template.replace("{{ CALENDAR }}", &str))
    }
}
