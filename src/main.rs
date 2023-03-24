//! # `Seri`
//! A Domain Specific Language Compiler for organising events and exporting details to different formats

// Setup Linting options
#![deny(clippy::all)]       // All clippy default warn & errors set as errors

// Warning on missing documentation
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use crate::passes::{parser::ParseTimetable, CompilingPass};
use std::{
    fs,
    io::Read,
};
use clap::Parser;

pub mod event;
pub mod passes;

/// Structure meant to store CLAP command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// An optional path to a file
    #[arg(help = "File to compile. If not present, will read from standard input")]
    file: Option<String>,
}

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
