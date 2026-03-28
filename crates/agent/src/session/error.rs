use std::fmt;

use crate::config::ConfigError;
use crate::runtime::EngineError;

#[derive(Debug)]
pub enum AgentError {
    SessionNotFound(String),
    SessionAlreadyExists(String),
    ConfigError(ConfigError),
    EngineError(EngineError),
    InternalError(String),
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SessionNotFound(id) => write!(f, "agent session not found: {id}"),
            Self::SessionAlreadyExists(id) => write!(f, "agent session already exists: {id}"),
            Self::ConfigError(e) => write!(f, "config error: {e}"),
            Self::EngineError(e) => write!(f, "engine error: {e}"),
            Self::InternalError(msg) => write!(f, "internal error: {msg}"),
        }
    }
}

impl std::error::Error for AgentError {}

impl From<ConfigError> for AgentError {
    fn from(e: ConfigError) -> Self {
        Self::ConfigError(e)
    }
}

impl From<EngineError> for AgentError {
    fn from(e: EngineError) -> Self {
        Self::EngineError(e)
    }
}
