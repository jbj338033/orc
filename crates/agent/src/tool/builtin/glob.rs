use async_trait::async_trait;
use orc_core::provider::ToolDef;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use crate::runtime::ToolResult;
use crate::tool::Tool;

pub struct GlobSearch;

#[async_trait]
impl Tool for GlobSearch {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "glob".into(),
            description: "find files matching a glob pattern. returns file paths.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string", "description": "glob pattern (e.g. **/*.rs)" },
                    "path": { "type": "string", "description": "base directory (default: .)" }
                },
                "required": ["pattern"]
            }),
        }
    }

    async fn execute(&self, input: serde_json::Value, cancel: CancellationToken) -> ToolResult {
        let pattern = match input.get("pattern").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult { id: String::new(), output: "missing pattern".into(), is_error: true },
        };
        let path = input.get("path").and_then(|v| v.as_str()).unwrap_or(".");

        let result: Result<std::process::Output, std::io::Error> = tokio::select! {
            r = Command::new("find").arg(path).arg("-path").arg(pattern).arg("-type").arg("f").output() => r,
            _ = cancel.cancelled() => {
                return ToolResult { id: String::new(), output: "cancelled".into(), is_error: true };
            }
        };

        match result {
            Ok(output) => {
                let text = String::from_utf8_lossy(&output.stdout).to_string();
                if text.is_empty() {
                    ToolResult { id: String::new(), output: "no files found".into(), is_error: false }
                } else {
                    ToolResult { id: String::new(), output: text.trim().to_string(), is_error: false }
                }
            }
            Err(e) => ToolResult { id: String::new(), output: e.to_string(), is_error: true },
        }
    }
}
