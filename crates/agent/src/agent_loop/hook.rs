use async_trait::async_trait;

use crate::runtime::ToolResult;
use orc_core::provider::ToolDef;

#[derive(Debug, Clone)]
pub enum HookDecision {
    Allow,
    Deny(String),
}

#[async_trait]
pub trait Hook: Send + Sync {
    async fn pre_tool_use(&self, _tool: &ToolDef, _input: &serde_json::Value) -> HookDecision {
        HookDecision::Allow
    }

    async fn post_tool_use(
        &self,
        _tool: &ToolDef,
        _input: &serde_json::Value,
        _result: &ToolResult,
    ) {
    }

    async fn on_stop(&self) {}
}
