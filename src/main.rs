#![deny(clippy::all)]

use crate::passes::{parser::ParseTimetable, CompilingPass};
use std::{
    path::Path,
    fs, io::Read,
};
use clap::Parser;

pub mod event;
pub mod passes;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(help = "File to compile. If not present, will read from standard input")]
    file: Option<String>,
}


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
    let args = Args::parse();

    let content =
        if let Some(filepath) = args.file {
            fs::read_to_string(filepath).expect("Could not read file")
        } else {
            let mut buffer = Vec::new();
            std::io::stdin()
                .read_to_end(&mut buffer).unwrap();
            String::from_utf8(buffer).unwrap()
        };

    println!("{:?}", ParseTimetable::apply(&content).unwrap());
}
