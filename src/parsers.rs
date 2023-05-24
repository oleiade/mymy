use std::error::Error;
use std::fmt::Display;
use std::num::ParseIntError;
use std::str::FromStr;
use std::time::Duration;

use anyhow::Result;

pub fn parse_duration(input: &str) -> Result<Duration, ParseDurationError> {
    let input = input.trim();

    let (value_str, unit_str) = match input.find(|c: char| c.is_whitespace() || c.is_alphabetic()) {
        Some(pos) => input.split_at(pos),
        None => return Err(ParseDurationError::InvalidFormat),
    };

    let value: u64 = u64::from_str(value_str.trim())?;
    let unit = unit_str.trim();

    let duration = match unit {
        "ms" => Duration::from_millis(value),
        "s" => Duration::from_secs(value),
        "m" => Duration::from_secs(value * 60),
        _ => return Err(ParseDurationError::InvalidFormat),
    };

    Ok(duration)
}

#[derive(Debug)]
pub enum ParseDurationError {
    InvalidFormat,
    InvalidValue(ParseIntError),
}

impl From<ParseIntError> for ParseDurationError {
    fn from(err: ParseIntError) -> Self {
        ParseDurationError::InvalidValue(err)
    }
}

impl Display for ParseDurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseDurationError::InvalidFormat => write!(f, "invalid duration format"),
            ParseDurationError::InvalidValue(err) => write!(f, "invalid duration value: {}", err),
        }
    }
}

impl Error for ParseDurationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseDurationError::InvalidFormat => None,
            ParseDurationError::InvalidValue(err) => Some(err),
        }
    }
}