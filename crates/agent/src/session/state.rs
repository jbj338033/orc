use tokio_util::sync::CancellationToken;

use crate::config::AgentProfile;
use crate::runtime::Message;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Idle,
    Thinking,
    Waiting,
    Error,
}

pub struct AgentSession {
    pub profile: AgentProfile,
    pub messages: Vec<Message>,
    pub status: AgentStatus,
    pub cancel_token: CancellationToken,
}

impl AgentSession {
    pub fn new(profile: AgentProfile) -> Self {
        Self {
            profile,
            messages: Vec::new(),
            status: AgentStatus::Idle,
            cancel_token: CancellationToken::new(),
        }
    }
}
