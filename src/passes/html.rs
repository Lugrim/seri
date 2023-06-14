//! HTML backend
use crate::{
    event::{find_bounding_box, Event, InvalidDatetime, Type},
    passes::CompilingPass,
    templating::{replace, Error},
};
use chrono::{Datelike, Duration};
use isolang::Language;
use markdown::mdast::Node;
use std::str::FromStr;
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
    /// An error occurred while trying to replace text in the template
    #[error("Error while trying to replace in template file: {0}")]
    CouldNotReplaceTemplate(#[from] Error),
    /// The datetime of either the first day or last day of the bounding box is not valid.
    #[error(transparent)]
    InvalidDatetime(#[from] InvalidDatetime),
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
        let duration = self.duration * 100 / (8 * 60);
        let class = self.event_type.to_string();
        // Create a div for the event
        let mut res = format!("\t<div class=\"event {class}\" style=\"height: {duration}%;\">");

        // Display the title and author of the event
        res += "<div class=\"title\">";
        res += format!(
            "{}<b>{}</b><br>",
            self.language
                .map_or_else(String::new, |l| l.to_html() + " "),
            self.title
        )
        .as_str();
        if self.event_type == Type::Talk && !self.speakers.is_empty() {
            res += "<span>";
            res += &self.speakers_string();
            res += "</span>";
        }
        res += "</div>\n";

        // Display the abstract of the event
        res += format!(
            "<div class=\"abstract\"><p>{}</p></div>",
            &self.description.to_html()
        )
        .as_str();
        res += "</div>";

        res
    }
}

impl ToHTML for Language {
    fn to_html(&self) -> String {
        match self {
            Self::Fra => "🇫🇷",
            Self::Eng => "🇬🇧",
            _ => "?",
        }
        .to_owned()
    }
}

impl ToHTML for Node {
    fn to_html(&self) -> String {
        use Node::*;

        match &self {
            Root(root) => root
                .children
                .iter()
                .map(|e| e.to_html())
                .collect::<Vec<String>>()
                .join("\n"),
            BlockQuote(blockquote) => {
                "<blockquote>".to_owned()
                    + &blockquote
                        .children
                        .iter()
                        .map(|e| e.to_html())
                        .collect::<Vec<String>>()
                        .join("\n")
                    + "</blockquote>"
            }
            List(list) => {
                if list.ordered { "<ol>" } else { "<ul>" }.to_owned()
                    + &list
                        .children
                        .iter()
                        .map(|e| e.to_html())
                        .collect::<Vec<String>>()
                        .join("\n")
                    + if list.ordered { "</ol>" } else { "</ul>" }
            }
            ListItem(listitem) => {
                "<li>".to_owned()
                    + &listitem
                        .children
                        .iter()
                        .map(|e| e.to_html())
                        .collect::<Vec<String>>()
                        .join("\n")
                    + "</li>"
            }
            Delete(delete) => {
                "<del>".to_owned()
                    + &delete
                        .children
                        .iter()
                        .map(|e| e.to_html())
                        .collect::<Vec<String>>()
                        .join("\n")
                    + "</del>"
            }
            Emphasis(emphasis) => {
                "<em>".to_owned()
                    + &emphasis
                        .children
                        .iter()
                        .map(|e| e.to_html())
                        .collect::<Vec<String>>()
                        .join("\n")
                    + "</em>"
            }
            Link(link) => {
                "<a href=\"".to_owned()
                    + &link.url
                    + "\" title=\""
                    + &link.title.clone().unwrap_or_default()
                    + "\">"
                    + &link
                        .children
                        .iter()
                        .map(|e| e.to_html())
                        .collect::<Vec<String>>()
                        .join("\n")
                    + "</a>"
            }
            Strong(strong) => "<b>".to_owned() +
                &strong
                .children
                .iter()
                .map(|e| e.to_html())
                .collect::<Vec<String>>()
                .join("\n")
                + "</b>",
            Heading(heading) => {
                "<h".to_owned()
                    + &heading.depth.to_string()
                    + ">"
                    + &heading
                        .children
                        .iter()
                        .map(|e| e.to_html())
                        .collect::<Vec<String>>()
                        .join("\n")
                    + "</h"
                    + &heading.depth.to_string()
                    + ">"
            }
            Table(table) => {
                "<table>".to_owned()
                    + &table
                        .children
                        .iter()
                        .map(|e| e.to_html())
                        .collect::<Vec<String>>()
                        .join("\n")
                    + "</table>"
            }
            TableRow(tablerow) => {
                "<tr>".to_owned()
                    + &tablerow
                        .children
                        .iter()
                        .map(|e| e.to_html())
                        .collect::<Vec<String>>()
                        .join("\n")
                    + "</tr>"
            }
            TableCell(tablecell) => {
                "<td>".to_owned()
                    + &tablecell
                        .children
                        .iter()
                        .map(|e| e.to_html())
                        .collect::<Vec<String>>()
                        .join("\n")
                    + "</td>"
            }
            Paragraph(paragraph) => {
                "<p>".to_owned()
                    + &paragraph
                        .children
                        .iter()
                        .map(|e| e.to_html())
                        .collect::<Vec<String>>()
                        .join("\n")
                    + "</p>"
            }
            InlineCode(inlinecode) => "<code>".to_owned() + &inlinecode.value + "</code>",
            // TODO Use metadata
            Code(code) => "<pre>".to_owned() + &code.value + "</pre>",
            Text(text) => text.value.clone(),
            // === TODO ===
            FootnoteDefinition(footnotedefinition) => footnotedefinition
                .children
                .iter()
                .map(|e| e.to_html())
                .collect::<Vec<String>>()
                .join("\n"),
            MdxJsxFlowElement(mdxjsxflowelement) => mdxjsxflowelement
                .children
                .iter()
                .map(|e| e.to_html())
                .collect::<Vec<String>>()
                .join("\n"),
            MdxJsxTextElement(mdxjsxtextelement) => mdxjsxtextelement
                .children
                .iter()
                .map(|e| e.to_html())
                .collect::<Vec<String>>()
                .join("\n"),
            LinkReference(linkreference) => linkreference
                .children
                .iter()
                .map(|e| e.to_html())
                .collect::<Vec<String>>()
                .join("\n"),
            MdxjsEsm(mdxjsesm) => format!("{:?}", mdxjsesm),
            Toml(toml) => format!("{:?}", toml),
            Yaml(yaml) => format!("{:?}", yaml),
            Break(brk) => format!("{:?}", brk),
            InlineMath(inlinemath) => format!("{:?}", inlinemath),
            MdxTextExpression(mdxtextexpression) => format!("{:?}", mdxtextexpression),
            FootnoteReference(footnotereference) => format!("{:?}", footnotereference),
            Html(html) => format!("{:?}", html),
            Image(image) => format!("{:?}", image),
            ImageReference(imagereference) => format!("{:?}", imagereference),
            Math(math) => format!("{:?}", math),
            MdxFlowExpression(mdxflowexpression) => format!("{:?}", mdxflowexpression),
            ThematicBreak(thematicbreak) => format!("{:?}", thematicbreak),
            Definition(definition) => format!("{:?}", definition),
        }
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

        // Find the number of days and the first day of the events
        let bounding_box = find_bounding_box(&events).ok_or(InvalidDatetime {})?;
        let nb_days = bounding_box.last_day()?.day() - bounding_box.first_day().unwrap().day() + 1;
        let first_day = bounding_box.first_day()?.date_naive();

        let mut str = String::new();
        // Create a div for each day, containing the event of the day
        for i in 0..nb_days {
            let curr_day = first_day + Duration::days(i64::from(i));
            let mut day_events: Vec<&Event> = events
                .iter()
                .filter(|ev| ev.start_date.date_naive() == curr_day)
                .collect();
            day_events.sort_by_key(|e| e.start_date.time());

            str += "<div class=\"day\">";
            str.push_str(format!("<h2>{}</h2>", curr_day.format("%A, %B %e")).as_str());
            let mut previous_hour = None;
            for event in day_events {
                // Display the start time, if needed
                if previous_hour.is_none() || previous_hour.unwrap() == event.start_date {
                    // Display the end time
                    str += event.start_date.format("%H:%M").to_string().as_str();
                    previous_hour = Some(event.start_date);
                }
                // Display the event
                str += event.to_html().as_str();
                // Display the end time
                str += (event.start_date + Duration::minutes(i64::from(event.duration)))
                    .format("%H:%M")
                    .to_string()
                    .as_str();
            }
            str += "</div>";
        }
        Ok(replace(&template, "CALENDAR", &str)?)
    }
}
