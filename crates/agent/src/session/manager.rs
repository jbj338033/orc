use std::collections::HashMap;
use std::sync::Arc;

use futures::StreamExt;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::config::ConfigStore;
use crate::runtime::{AgentEngine, AgentEvent, Message, MessageRole};

use super::error::AgentError;
use super::state::{AgentSession, AgentStatus};

pub trait AgentEventHandler: Send + Sync + 'static {
    fn on_event(&self, session_id: &str, event: AgentEvent);
}

pub struct AgentManager {
    sessions: RwLock<HashMap<String, AgentSession>>,
    engine: Arc<dyn AgentEngine>,
    config: Arc<dyn ConfigStore>,
    handler: Arc<dyn AgentEventHandler>,
}

impl AgentManager {
    pub fn new(
        engine: Arc<dyn AgentEngine>,
        config: Arc<dyn ConfigStore>,
        handler: Arc<dyn AgentEventHandler>,
    ) -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            engine,
            config,
            handler,
        }
    }

    pub async fn spawn(&self, id: String, profile_id: &str) -> Result<(), AgentError> {
        let profile = self.config.load_profile(profile_id)?;

        let mut sessions = self.sessions.write().await;
        if sessions.contains_key(&id) {
            return Err(AgentError::SessionAlreadyExists(id));
        }
        sessions.insert(id, AgentSession::new(profile));
        Ok(())
    }

    pub async fn send(&self, id: &str, content: String) -> Result<(), AgentError> {
        let (messages, cancel_token) = {
            let mut sessions = self.sessions.write().await;
            let session = sessions
                .get_mut(id)
                .ok_or_else(|| AgentError::SessionNotFound(id.to_string()))?;

            session.messages.push(Message {
                role: MessageRole::User,
                content,
            });
            session.status = AgentStatus::Thinking;
            session.cancel_token = CancellationToken::new();

            (session.messages.clone(), session.cancel_token.clone())
        };

        let mut stream = self.engine.send(&messages, &[], cancel_token).await?;

        let handler = Arc::clone(&self.handler);
        let sessions = &self.sessions;
        let session_id = id.to_string();

        let mut full_response = String::new();

        while let Some(event) = stream.next().await {
            match &event {
                AgentEvent::TextDelta(delta) => {
                    full_response.push_str(delta);
                }
                AgentEvent::Done => {
                    let mut sessions = sessions.write().await;
                    if let Some(session) = sessions.get_mut(&session_id) {
                        session.messages.push(Message {
                            role: MessageRole::Assistant,
                            content: full_response.clone(),
                        });
                        session.status = AgentStatus::Idle;
                    }
                }
                AgentEvent::Error(_) => {
                    let mut sessions = sessions.write().await;
                    if let Some(session) = sessions.get_mut(&session_id) {
                        session.status = AgentStatus::Error;
                    }
                }
                _ => {}
            }
            handler.on_event(&session_id, event);
        }

        Ok(())
    }

    pub async fn kill(&self, id: &str) -> Result<(), AgentError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get(id)
            .ok_or_else(|| AgentError::SessionNotFound(id.to_string()))?;
        session.cancel_token.cancel();
        sessions.remove(id);
        Ok(())
    }

    pub async fn status(&self, id: &str) -> Result<AgentStatus, AgentError> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(id)
            .ok_or_else(|| AgentError::SessionNotFound(id.to_string()))?;
        Ok(session.status)
    }
}
