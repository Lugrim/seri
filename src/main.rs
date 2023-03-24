#![deny(clippy::all)]

use std::collections::HashMap;
use std::fs;
use std::str::FromStr;

#[derive(Debug, PartialEq, Default)]
enum EventType {
    #[default]
    Talk,
    Meal,
    Break,
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

#[derive(Debug)]
struct Event {
    event_type: EventType,
    description: String,
}

fn split_kv(string: &str) -> HashMap<&str, &str> {
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
            let settings = split_kv(header);
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

fn get_descriptions(events: Vec<Event>) -> String {
    let mut r = String::new();
    for e in events {
        r += "=============\n";
        r += &e.description;
        r += "\n";
    }
    r
}

fn parse_file_content(content: &str) -> Vec<Event> {
    content
        .split("---")
        .filter_map(|e| Event::from_str(e).ok())
        .collect()
}

fn main() {
    let content = fs::read_to_string("data/example.todo").expect("Could not read file");
    println!("{}", get_descriptions(parse_file_content(&content)));
}
