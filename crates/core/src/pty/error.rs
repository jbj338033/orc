use std::fmt;
use std::io;

#[derive(Debug)]
pub enum PtyError {
    SpawnFailed(String),
    SessionNotFound(String),
    IoError(io::Error),
}

impl fmt::Display for PtyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SpawnFailed(msg) => write!(f, "spawn failed: {msg}"),
            Self::SessionNotFound(id) => write!(f, "session not found: {id}"),
            Self::IoError(e) => write!(f, "io error: {e}"),
        }
    }
}

impl std::error::Error for PtyError {}

impl From<io::Error> for PtyError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}
