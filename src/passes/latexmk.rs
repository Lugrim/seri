//! Will call eventually call Latexmk on a previous pass input

use glob::glob;
use rand::Fill;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::passes::CompilingPass;

/// Options for Latexmk call pass
pub struct Options {
    /// Path of the input file
    pub input_path: Option<String>,
    /// Path of the output file
    pub output_path: Option<String>,
    /// Save temporary files
    pub save_temps: bool,
}

use thiserror::Error;

/// Error occurring when compiling an event list to `TikZ`.
#[derive(Debug, Error)]
pub enum Error {
    /// Error from launching Latexmk
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    /// Error on temporary file creation
    #[error(transparent)]
    CouldNotCreateTempFile(#[from] TempFileCreationError),
}

/// Will call Latexmk with the `$pdflatex` target, if found on the system
pub struct Pass {}

/// Error occurring when creating a temporary file
#[derive(Debug, Error)]
pub enum TempFileCreationError {
    /// Error returned from random generator
    #[error("Error while trying to generate a random string: {0}")]
    CouldNotCreateRandomString(#[from] rand::Error),
}

/// Generate a random String of size `len` that should be valid as a file name
///
/// # Errors
///
/// On file temporary creation, RNG or io errors can happen
fn random_filename(len: usize) -> Result<String, TempFileCreationError> {
    let alphabet: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
        .chars()
        .collect();
    let alpabet_length = alphabet.len();

    let mut rng = rand::thread_rng();

    let mut src: Vec<u8> = vec![0; len];

    src.try_fill(&mut rng)?;

    Ok(src
        .into_iter()
        .map(|l| alphabet[l as usize % alpabet_length])
        .collect())
}

fn tex_pathbuf_from_random_string(len: usize) -> Result<PathBuf, TempFileCreationError> {
    Ok(PathBuf::from(random_filename(len)? + ".tex"))
}

/// Will try to get a random file name that does not exist
///
/// # Errors
///
/// Errors can happen in RNG or on IO operations.
pub fn random_valid_filename(len: usize) -> Result<PathBuf, TempFileCreationError> {
    let mut filepath = tex_pathbuf_from_random_string(len)?;

    while filepath.exists() {
        filepath = tex_pathbuf_from_random_string(len)?;
    }

    Ok(filepath)
}

// TODO Error management, get rid of unwraps
fn cleanup(input_path: &Path) {
    for entry in glob(input_path.with_extension("*").to_str().unwrap()).unwrap() {
        let e = entry.unwrap();
        if e.is_file() {
            if let Err(err) = fs::remove_file(e) {
                eprintln!("[Warning] Could not delete temporary file: {err}");
            }
        }
    }
}

impl CompilingPass<&str, Options> for Pass {
    type Residual = Vec<u8>;
    type Error = Error;
    fn apply_with(latex: &str, options: Options) -> Result<Self::Residual, Self::Error> {
        let input_file = options
            .input_path
            .map_or_else(|| random_valid_filename(16), |s| Ok(PathBuf::from(s)));

        let input_unwrapped = input_file?;

        fs::write(&input_unwrapped, latex)?;

        let latexmk = Command::new("latexmk")
            .arg("-pdflua")
            .arg(&input_unwrapped)
            .spawn()?;

        latexmk.wait_with_output().expect("failed to wait on child");

        let ret = fs::read(input_unwrapped.with_extension("pdf")).map_err(Error::from);

        cleanup(&input_unwrapped);

        ret
    }

    fn apply(latex: &str) -> Result<Self::Residual, Self::Error> {
        Self::apply_with(
            latex,
            Options {
                input_path: None,
                output_path: None,
                save_temps: false,
            },
        )
    }
}
