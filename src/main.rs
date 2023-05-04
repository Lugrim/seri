//! # `Seri`
//! A Domain Specific Language Compiler for organizing events and exporting details to different formats

// Make Clippy quite nasty
#![deny(clippy::cargo)] // Checks for garbage in the Cargo TOML files
#![allow(clippy::multiple_crate_versions)] // Dependencies doing bad things
#![deny(clippy::complexity)] // Checks for needlessly complex structures
#![deny(clippy::correctness)] // Checks for common invalid usage and workarounds
#![deny(clippy::nursery)] // Checks for things that are typically forgotten by learners
#![allow(clippy::option_if_let_else)] // Always suggests to use map_or_else instead of match, which
// is hard to read
#![deny(clippy::pedantic)] // Checks for mildly annoying comments it could make about your code
#![deny(clippy::perf)] // Checks for inefficient ways to perform common tasks
#![deny(clippy::style)] // Checks for inefficient styling of code
#![deny(clippy::suspicious)] // Checks for potentially malicious behavior
// Add some new Clippy lints
#![deny(clippy::use_self)] // Checks for the use of a struct's name in its `impl`
// Add some default lints
#![warn(unused_variables)] // Checks for unused variables
// Warn on missing documentation
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

use crate::{
    event::{Event, ParsingError},
    passes::{
        abstex,
        html::{HTMLBackend, HTMLBackendCompilationError, HTMLBackendOptions},
        latexmk,
        parser::ParseTimetable,
        tikz, PassInput,
    },
};

use clap::Parser;

use std::{
    fs,
    io::{Read, Write},
};

use thiserror::Error;

pub mod event;
pub mod passes;
pub mod templating;

/// Help me to do something cleaner than this please
#[derive(Debug, Error)]
pub enum CompilerError {
    /// An error occurred in the Parser
    #[error("Error while trying to parse Seri input: {0}")]
    CouldNotParseSeri(#[from] ParsingError),
    /// An error occurred in the HTML backend
    #[error("Error while trying to generate the HTML output: {0}")]
    CouldNotGenerateHTML(#[from] HTMLBackendCompilationError),
    /// An error occurred in the LaTeX abstracts backend
    #[error("Error while trying to generate the LaTeX abstract output: {0}")]
    CouldNotGenerateAbsTex(#[from] abstex::Error),
    /// An error occurred in the TikZ backend
    #[error("Error while trying to generate the TikZ output: {0}")]
    CouldNotGenerateTikz(#[from] tikz::Error),
    /// An error occurred calling Latexmk
    #[error("Error while trying to call Latexmk output: {0}")]
    CouldNotCallLatexmk(#[from] latexmk::Error),
    /// The output format selected is not supported
    #[error("Backend not implemented yet: {0}")]
    BackendNotImplemented(String),
    /// An error occurred writing to stdout
    #[error("Error while trying to write output: {0}")]
    CouldNotReadUtf8(#[from] std::io::Error),
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
    #[arg(short, long, help = "Keep intermediate files", default_value_t = false)]
    save_tmp: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
#[allow(clippy::upper_case_acronyms)]
enum Format {
    Tikz,
    TikzPDF,
    AbstractLatex,
    AbstractPDF,
    HTML,
}

impl PassInput for &str {}
impl PassInput for Vec<Event> {}

fn generate_abstract_pdf(
    content: &str,
    abstex_options: abstex::Options,
    latexmk_options: latexmk::Options,
) -> Result<Vec<u8>, CompilerError> {
    content
        .chain_pass::<ParseTimetable>()?
        .chain_pass_with::<abstex::Pass, abstex::Options>(abstex_options)?
        .chain_pass_with::<latexmk::Pass, latexmk::Options>(latexmk_options)
        .map_err(CompilerError::from)
}

fn generate_tikz_pdf(
    content: &str,
    tikz_options: tikz::Options,
    latexmk_options: latexmk::Options,
) -> Result<Vec<u8>, CompilerError> {
    content
        .chain_pass::<ParseTimetable>()?
        .chain_pass_with::<tikz::Pass, tikz::Options>(tikz_options)?
        .chain_pass_with::<latexmk::Pass, latexmk::Options>(latexmk_options)
        .map_err(CompilerError::from)
}

fn generate_tikz(options: tikz::Options, content: &str) -> Result<Vec<u8>, CompilerError> {
    content
        .chain_pass::<ParseTimetable>()?
        .chain_pass_with::<tikz::Pass, tikz::Options>(options)
        .map(String::into_bytes)
        .map_err(CompilerError::from)
}

fn generate_abstex(options: abstex::Options, content: &str) -> Result<Vec<u8>, CompilerError> {
    content
        .chain_pass::<ParseTimetable>()?
        .chain_pass_with::<abstex::Pass, abstex::Options>(options)
        .map(String::into_bytes)
        .map_err(CompilerError::from)
}

fn generate_html(options: HTMLBackendOptions, content: &str) -> Result<Vec<u8>, CompilerError> {
    content
        .chain_pass::<ParseTimetable>()?
        .chain_pass_with::<HTMLBackend, HTMLBackendOptions>(options)
        .map(String::into_bytes)
        .map_err(CompilerError::from)
}

fn open_output_file(path: Option<String>) -> Result<Box<dyn Write>, std::io::Error> {
    match path {
        Some(p) => Ok(fs::File::create(p).map(Box::new)?),
        None => Ok(Box::new(std::io::stdout())),
    }
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
fn write_output(output: &mut impl Write, data: &[u8]) -> Result<(), std::io::Error> {
    output.write_all(data)
}

fn main() -> Result<(), CompilerError> {
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

    let mut outfile = open_output_file(args.output.clone())?;

    let output = match args.format {
        Format::Tikz => generate_tikz(
            tikz::Options {
                template_path: template,
            },
            &content,
        ),
        Format::TikzPDF => generate_tikz_pdf(
            &content,
            tikz::Options {
                template_path: template,
            },
            latexmk::Options {
                input_path: None,
                output_path: args.output,
                save_temps: args.save_tmp,
            },
        ),
        Format::AbstractLatex => generate_abstex(
            abstex::Options {
                template_path: template,
            },
            &content,
        ),
        Format::AbstractPDF => generate_abstract_pdf(
            &content,
            abstex::Options {
                template_path: template,
            },
            latexmk::Options {
                input_path: None,
                output_path: args.output,
                save_temps: args.save_tmp,
            },
        ),
        Format::HTML => generate_html(
            HTMLBackendOptions {
                template_path: template,
            },
            &content,
        ),
    }?;

    write_output(&mut outfile, &output).map_err(CompilerError::from)
}
