use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ConfigError {
    NotFound(String),
    IoError(io::Error),
    ParseError(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "profile not found: {id}"),
            Self::IoError(e) => write!(f, "config io error: {e}"),
            Self::ParseError(msg) => write!(f, "config parse error: {msg}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(e: serde_json::Error) -> Self {
        Self::ParseError(e.to_string())
    }
}
