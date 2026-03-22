use std::fmt;

#[derive(Debug)]
pub enum Error {
    Config(String),
    Provider(String),
    Tool(String),
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Config(msg) => write!(f, "config: {msg}"),
            Error::Provider(msg) => write!(f, "provider: {msg}"),
            Error::Tool(msg) => write!(f, "tool: {msg}"),
            Error::Io(err) => write!(f, "io: {err}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}
