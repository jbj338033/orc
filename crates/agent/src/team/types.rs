use serde::{Deserialize, Serialize};

use crate::config::AgentProfile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamConfig {
    pub name: String,
    pub orchestrator: AgentProfile,
    pub members: Vec<TeamMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub role: String,
    pub profile: AgentProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamTask {
    pub id: String,
    pub description: String,
    pub assigned_to: Option<String>,
    pub status: TaskStatus,
    pub file_ownership: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub enum TeamEvent {
    TaskAssigned { task_id: String, member_role: String },
    TaskCompleted { task_id: String, member_role: String },
    TaskFailed { task_id: String, member_role: String, reason: String },
    MemberMessage { from: String, content: String },
    AllComplete,
}
