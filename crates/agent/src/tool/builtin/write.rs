use async_trait::async_trait;
use orc_core::provider::ToolDef;
use tokio_util::sync::CancellationToken;

use crate::runtime::ToolResult;
use crate::tool::Tool;

pub struct WriteFile;

#[async_trait]
impl Tool for WriteFile {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "write".into(),
            description: "write content to a file, creating directories if needed.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "absolute file path" },
                    "content": { "type": "string", "description": "file content to write" }
                },
                "required": ["path", "content"]
            }),
        }
    }

    async fn execute(&self, input: serde_json::Value, _cancel: CancellationToken) -> ToolResult {
        let path = match input.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult { id: String::new(), output: "missing path".into(), is_error: true },
        };
        let content = match input.get("content").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => return ToolResult { id: String::new(), output: "missing content".into(), is_error: true },
        };

        if let Some(parent) = std::path::Path::new(path).parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                return ToolResult { id: String::new(), output: e.to_string(), is_error: true };
            }
        }

        match tokio::fs::write(path, content).await {
            Ok(()) => ToolResult { id: String::new(), output: format!("wrote {path}"), is_error: false },
            Err(e) => ToolResult { id: String::new(), output: e.to_string(), is_error: true },
        }
    }
}
