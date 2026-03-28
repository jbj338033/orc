use async_trait::async_trait;
use orc_core::provider::ToolDef;
use tokio_util::sync::CancellationToken;

use crate::runtime::ToolResult;
use crate::tool::Tool;

pub struct ReadFile;

#[async_trait]
impl Tool for ReadFile {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "read".into(),
            description: "read a file from disk. supports optional line range.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "absolute file path" },
                    "offset": { "type": "integer", "description": "start line (1-based, optional)" },
                    "limit": { "type": "integer", "description": "number of lines to read (optional)" }
                },
                "required": ["path"]
            }),
        }
    }

    async fn execute(&self, input: serde_json::Value, _cancel: CancellationToken) -> ToolResult {
        let path = match input.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult { id: String::new(), output: "missing path".into(), is_error: true },
        };

        let content = match tokio::fs::read_to_string(path).await {
            Ok(c) => c,
            Err(e) => return ToolResult { id: String::new(), output: e.to_string(), is_error: true },
        };

        let lines: Vec<&str> = content.lines().collect();
        let offset = input.get("offset").and_then(|v| v.as_u64()).unwrap_or(1).max(1) as usize - 1;
        let limit = input.get("limit").and_then(|v| v.as_u64()).map(|v| v as usize);

        let end = limit.map(|l| (offset + l).min(lines.len())).unwrap_or(lines.len());
        let slice = &lines[offset.min(lines.len())..end.min(lines.len())];

        let numbered: String = slice
            .iter()
            .enumerate()
            .map(|(i, line)| format!("{:>6}\t{}", offset + i + 1, line))
            .collect::<Vec<_>>()
            .join("\n");

        ToolResult { id: String::new(), output: numbered, is_error: false }
    }
}
