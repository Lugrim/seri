//!Small templating engine

use regex::{NoExpand, Regex};
use thiserror::Error;

/// An error in template engine used
#[derive(Debug, Error)]
pub enum Error {
    /// The template could not be replaced
    #[error(transparent)]
    CouldReplaceInTemplate(#[from] regex::Error),
}

/// Replace the `{{ key }}` template string by the `replacement` in the `source`
///
/// # Errors
///
/// Since the engine currently uses `Regex`, the module can return an error if an expression is
/// malformed
///
/// Known issue: the regex is injectable
pub fn replace(source: &str, key: &str, replacement: &str) -> Result<String, Error> {
    let expression = r"\{\{ *".to_owned() + key + r" *\}\}";
    let re = Regex::new(&expression)?;
    Ok(re.replace(source, NoExpand(replacement)).into_owned())
}
