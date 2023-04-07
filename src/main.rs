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
    event::Event,
    passes::{
        html::{HTMLBackend, HTMLBackendCompilationError, HTMLBackendOptions},
        latex::{TikzBackend, TikzBackendCompilationError},
        parser::ParseTimetable,
        PassInput,
    },
};

use clap::Parser;
use std::{fs, io::Read};
use thiserror::Error;

pub mod event;
pub mod passes;

/// Help me to do something cleaner than this please
#[derive(Debug, Error)]
pub enum CompilerError {
    /// An error occured in the HTML backend
    #[error("Error while trying to generate the HTML output: {0}")]
    CouldNotGenerateHTML(#[from] HTMLBackendCompilationError),
    /// An error occured in the Tikz backend
    #[error("Error while trying to generate the Tikz output: {0}")]
    CouldNotGenerateTikz(#[from] TikzBackendCompilationError),
    /// The output format selected is not supported
    #[error("Backend not implemented yet: {0}")]
    BackendNotImplemented(String),
}

/// Structure meant to store CLAP command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// An optional path to a file
    #[arg(help = "File to compile. If not present, will read from standard input")]
    file: Option<String>,
    #[arg(short, long, value_enum, value_name = "FORMAT", help = "Output format", default_value_t=Format::Tikz)]
    format: Format,
    #[arg(short, long, value_name = "TEMPLATE", help = "Template to use, if any")]
    template: Option<String>,
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "Output file. If not present, will output to stdout"
    )]
    output: Option<String>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum Format {
    Tikz,
    HTML,
}

impl PassInput for &str {}
impl PassInput for Vec<Event> {}

fn generate_tikz(content: &str) -> Result<String, TikzBackendCompilationError> {
    content
        .chain_pass::<ParseTimetable>()?
        .chain_pass::<TikzBackend>()
}

fn generate_html(
    options: HTMLBackendOptions,
    content: &str,
) -> Result<String, HTMLBackendCompilationError> {
    HTMLBackend::configure(options);
    content
        .chain_pass::<ParseTimetable>()?
        .chain_pass::<HTMLBackend>()
}

/// Write the output to a file or to stdout
///
/// # Arguments
///
/// * `output` - The path to the output file. If `None`, will output to stdout
/// * `data` - The data to write
///
/// # Returns
///
/// * `Ok(())` if the output was written successfully
/// * `Err(std::io::Error)` if the output could not be written
///
fn write_output(output: &Option<String>, data: String) -> Result<(), std::io::Error> {
    match output {
        Some(file_path) => fs::write(file_path, data)?,
        None => println!("{data}"),
    }
    Ok(())
}

fn main() {
    let args = Args::parse();

    let template = args.template.clone();
    let content = args.file.map_or_else(
        || {
            let mut buffer = Vec::new();
            std::io::stdin().read_to_end(&mut buffer).unwrap();
            String::from_utf8(buffer).unwrap()
        },
        |filepath| fs::read_to_string(filepath).expect("Could not read file"),
    );

    let html_opts = HTMLBackendOptions {
        template_path: template,
    };

    let output: Result<String, CompilerError> = match args.format {
        Format::Tikz => generate_tikz(&content).map_err(CompilerError::from),
        Format::HTML => generate_html(html_opts, &content).map_err(CompilerError::from),
    };

    match output {
        Ok(data) => write_output(&args.output, data).unwrap_or_else(|e| {
            panic!(
                "Couldn't write to file {}: {}",
                &args.output.unwrap_or_else(|| "stdout".to_string()),
                e
            )
        }),
        Err(err) => eprintln!("{err}"),
    }
}
