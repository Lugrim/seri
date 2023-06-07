use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// The language provided is not valid.
#[derive(Debug, Error)]
#[error("`{0} is not a valid language`")]
pub struct InvalidLanguage(pub String);

/// A language representation
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Lang {
    French,
    English,
}

impl FromStr for Lang {
    type Err = InvalidLanguage;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "fr" => Ok(Self::French),
            "en" => Ok(Self::English),
            tt => Err(InvalidLanguage(tt.to_owned())),
        }
    }
}

impl fmt::Display for Lang {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::French => write!(f, "French"),
            Self::English => write!(f, "English"),
        }
    }
}
