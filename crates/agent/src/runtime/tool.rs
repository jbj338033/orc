use serde::{Deserialize, Serialize};

// ToolDef is re-exported from orc_core::provider::ToolDef via mod.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub id: String,
    pub output: String,
    pub is_error: bool,
}
