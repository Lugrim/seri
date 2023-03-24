use std::collections::HashMap;
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
pub struct Event {
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
