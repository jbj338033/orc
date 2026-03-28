use std::sync::Arc;

use orc_agent::runtime::AgentEvent;
use orc_agent::session::{AgentEventHandler, AgentManager, AgentStatus};
use tauri::{AppHandle, Emitter};

struct TauriAgentHandler {
    app: AppHandle,
}

impl AgentEventHandler for TauriAgentHandler {
    fn on_event(&self, session_id: &str, event: AgentEvent) {
        match &event {
            AgentEvent::TextDelta(text) => {
                let _ = self
                    .app
                    .emit(&format!("agent-text-{session_id}"), text.clone());
            }
            AgentEvent::ToolCall { id, name, .. } => {
                let _ = self.app.emit(
                    &format!("agent-tool-call-{session_id}"),
                    serde_json::json!({"id": id, "name": name}),
                );
            }
            AgentEvent::ToolResult {
                id,
                output,
                is_error,
            } => {
                let _ = self.app.emit(
                    &format!("agent-tool-result-{session_id}"),
                    serde_json::json!({"id": id, "output": output, "is_error": is_error}),
                );
            }
            AgentEvent::Done => {
                let _ = self.app.emit(&format!("agent-done-{session_id}"), ());
            }
            AgentEvent::Error(msg) => {
                let _ = self
                    .app
                    .emit(&format!("agent-error-{session_id}"), msg.clone());
            }
        }
    }
}

pub struct AgentState {
    pub manager: AgentManager,
}

impl AgentState {
    pub fn new(app: AppHandle) -> Self {
        // placeholder engine — will be replaced when user configures a provider
        let handler = Arc::new(TauriAgentHandler { app });
        let config = Arc::new(orc_agent::config::FileConfigStore::new(
            dirs_next::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("orc")
                .join("profiles"),
        ).expect("failed to create config store"));
        let engine = Arc::new(PlaceholderEngine);

        Self {
            manager: AgentManager::new(engine, config, handler),
        }
    }
}

// placeholder until user sets up an API key
struct PlaceholderEngine;

#[async_trait::async_trait]
impl orc_agent::runtime::AgentEngine for PlaceholderEngine {
    async fn send(
        &self,
        _request: orc_agent::runtime::EngineRequest<'_>,
    ) -> Result<
        std::pin::Pin<Box<dyn futures::Stream<Item = AgentEvent> + Send>>,
        orc_agent::runtime::EngineError,
    > {
        Err(orc_agent::runtime::EngineError::RequestFailed(
            "no provider configured. set an api key first".into(),
        ))
    }
}

#[tauri::command]
pub async fn agent_spawn(
    state: tauri::State<'_, AgentState>,
    id: String,
    profile_id: String,
) -> Result<(), String> {
    state
        .manager
        .spawn(id, &profile_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn agent_send(
    state: tauri::State<'_, AgentState>,
    id: String,
    content: String,
) -> Result<(), String> {
    state
        .manager
        .send(&id, content)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn agent_kill(
    state: tauri::State<'_, AgentState>,
    id: String,
) -> Result<(), String> {
    state
        .manager
        .kill(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn agent_status(
    state: tauri::State<'_, AgentState>,
    id: String,
) -> Result<String, String> {
    let status = state
        .manager
        .status(&id)
        .await
        .map_err(|e| e.to_string())?;
    let s = match status {
        AgentStatus::Idle => "idle",
        AgentStatus::Thinking => "thinking",
        AgentStatus::Waiting => "waiting",
        AgentStatus::Error => "error",
    };
    Ok(s.to_string())
}
