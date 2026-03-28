use async_trait::async_trait;
use orc_core::provider::ToolDef;
use tokio_util::sync::CancellationToken;

use crate::runtime::ToolResult;
use crate::tool::Tool;

pub struct AskUser;

#[async_trait]
impl Tool for AskUser {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "ask_user".into(),
            description: "ask the user a question and wait for their response.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "question": { "type": "string", "description": "question to ask the user" }
                },
                "required": ["question"]
            }),
        }
    }

    async fn execute(&self, input: serde_json::Value, _cancel: CancellationToken) -> ToolResult {
        let question = input
            .get("question")
            .and_then(|v| v.as_str())
            .unwrap_or("(no question provided)");

        // in the real implementation, this will emit an event to the frontend
        // and wait for user input via a channel. for now, return a placeholder.
        ToolResult {
            id: String::new(),
            output: format!("[awaiting user response to: {question}]"),
            is_error: false,
        }
    }
}
