use async_trait::async_trait;
use orc_core::provider::ToolDef;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use crate::runtime::ToolResult;
use crate::tool::Tool;

pub struct Grep;

#[async_trait]
impl Tool for Grep {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "grep".into(),
            description: "search file contents using ripgrep. returns matching lines.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string", "description": "regex pattern to search" },
                    "path": { "type": "string", "description": "directory or file to search (default: .)" },
                    "glob": { "type": "string", "description": "file glob filter (e.g. *.rs)" }
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
        let glob_filter = input.get("glob").and_then(|v| v.as_str());

        let mut cmd = Command::new("rg");
        cmd.arg("--no-heading").arg("--line-number").arg("--color=never").arg("--max-count=100");
        if let Some(g) = glob_filter {
            cmd.arg("--glob").arg(g);
        }
        cmd.arg(pattern).arg(path);

        let result: Result<std::process::Output, std::io::Error> = tokio::select! {
            r = cmd.output() => r,
            _ = cancel.cancelled() => {
                return ToolResult { id: String::new(), output: "cancelled".into(), is_error: true };
            }
        };

        match result {
            Ok(output) => {
                let text = String::from_utf8_lossy(&output.stdout).to_string();
                if text.is_empty() {
                    ToolResult { id: String::new(), output: "no matches found".into(), is_error: false }
                } else {
                    ToolResult { id: String::new(), output: text, is_error: false }
                }
            }
            Err(e) => ToolResult { id: String::new(), output: format!("rg not found or failed: {e}"), is_error: true },
        }
    }
}
