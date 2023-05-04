//!Small templating engine

use regex::{NoExpand, Regex};
use thiserror::Error;

/// An error in template engine used
#[derive(Debug, Error)]
pub enum Error {
    /// The key contains disallowed characters
    #[error("Your key should only contain alphanumeric characters, dots, and underscores")]
    KeyNotAlphaNumeric,
    /// The template could not be replaced
    #[error("Could not execute regex replacement in templates: {0}")]
    TemplatingRegexError(#[from] regex::Error),
}

#[must_use]
/// Will check if the key is valid. For now, checks it against regex `^[[:word:]]+$`
pub fn validate_key(key: &str) -> bool {
    Regex::new("^[[:word:]]+$").unwrap().is_match(key)
}

/// Replace each occurence of `{{ key }}` template string by the `replacement` in the `source`.
/// If no key is found, will do nothing.
/// 
/// For allowed key format, see [`validate_key`]
///
/// # Errors
///
/// Since the engine currently uses `Regex`, the module can return an error if an expression is
/// malformed
///
/// Known issue: the regex is injectable
pub fn replace(source: &str, key: &str, replacement: &str) -> Result<String, Error> {
    if validate_key(key) {
        let expression = r"\{\{ *".to_owned() + key + r" *\}\}";
        let re = Regex::new(&expression)?;
        Ok(re.replace_all(source, NoExpand(replacement)).into_owned())
    } else {
        Err(Error::KeyNotAlphaNumeric)
    }
}
