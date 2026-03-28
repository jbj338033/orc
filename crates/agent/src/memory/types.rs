use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub parent_session_id: Option<String>,
    pub summary: String,
    pub seed_goal: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMemoryEntry {
    pub key: String,
    pub value: String,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLogEntry {
    pub session_id: String,
    pub event_type: EventType,
    pub payload: serde_json::Value,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Message,
    ToolCall,
    ToolResult,
    Compaction,
    Branch,
    Error,
}
