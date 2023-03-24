#![deny(clippy::all)]

use crate::passes::{parser::ParseTimetable, CompilingPass};
use std::fs;

pub mod event;
pub mod passes;

// fn get_descriptions(events: Vec<Event>) -> String {
//     let mut r = String::new();
//     for e in events {
//         r += "=============\n";
//         r += &e.description;
//         r += "\n";
//     }
//     r
// }

fn main() {
    let content = fs::read_to_string("data/example.todo").expect("Could not read file");
    println!("{:?}", ParseTimetable::apply(&content).unwrap());
}
