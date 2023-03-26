//! # `Seri`
//! A Domain Specific Language Compiler for organising events and exporting details to different formats

// Make clippy quite nasty
#![deny(clippy::cargo)] // Checks for garbage in the Cargo TOML files
#![allow(clippy::multiple_crate_versions)] // Dependencies doing bad things
#![deny(clippy::complexity)] // Checks for needlessly complex structures
#![deny(clippy::correctness)] // Checks for common invalid usage and workarounds
#![deny(clippy::nursery)] // Checks for things that are typically forgotten by learners
#![deny(clippy::pedantic)] // Checks for mildly annoying comments it could make about your code
#![deny(clippy::perf)] // Checks for inefficient ways to perform common tasks
#![deny(clippy::style)] // Checks for inefficient styling of code
#![deny(clippy::suspicious)] // Checks for potentially malicious behaviour
// Add some new clippy lints
#![deny(clippy::use_self)] // Checks for the use of a struct's name in its `impl`
// Add some default lints
#![warn(unused_variables)] // Checks for unused variables
// Warn on missing documentation
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]


use crate::{
    passes::{
        parser::ParseTimetable,
        latex::TikzFrontend,
        PassInput,
    },
    event::Event,
};
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

impl PassInput for &str {}
impl PassInput for Vec<Event> {}

fn main() {
    let args = Args::parse();

    let content = args.file.map_or_else(||
            {
                let mut buffer = Vec::new();
                std::io::stdin().read_to_end(&mut buffer).unwrap();
                String::from_utf8(buffer).unwrap()
            }, |filepath| fs::read_to_string(filepath).expect("Could not read file"));

    let out = &content.as_str()
        .chain_pass::<ParseTimetable, Vec<Event>, ()>().unwrap()
        .chain_pass::<TikzFrontend, String, ()>().unwrap();
    println!("{:}", out);
}
