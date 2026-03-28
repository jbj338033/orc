use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;
use tokio_util::sync::CancellationToken;

use super::event::AgentEvent;
use super::message::Message;
use super::tool::ToolDef;

#[derive(Debug)]
pub enum EngineError {
    RequestFailed(String),
    Cancelled,
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RequestFailed(msg) => write!(f, "engine request failed: {msg}"),
            Self::Cancelled => write!(f, "engine request cancelled"),
        }
    }
}

impl std::error::Error for EngineError {}

#[async_trait]
pub trait AgentEngine: Send + Sync {
    async fn send(
        &self,
        messages: &[Message],
        tools: &[ToolDef],
        cancel: CancellationToken,
    ) -> Result<Pin<Box<dyn Stream<Item = AgentEvent> + Send>>, EngineError>;
}
