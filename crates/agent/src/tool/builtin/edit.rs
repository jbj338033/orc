use async_trait::async_trait;
use orc_core::provider::ToolDef;
use tokio_util::sync::CancellationToken;

use crate::runtime::ToolResult;
use crate::tool::Tool;

pub struct EditFile;

#[async_trait]
impl Tool for EditFile {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "edit".into(),
            description: "replace an exact string in a file. old_string must be unique.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "absolute file path" },
                    "old_string": { "type": "string", "description": "exact string to find" },
                    "new_string": { "type": "string", "description": "replacement string" }
                },
                "required": ["path", "old_string", "new_string"]
            }),
        }
    }

    async fn execute(&self, input: serde_json::Value, _cancel: CancellationToken) -> ToolResult {
        let path = match input.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult { id: String::new(), output: "missing path".into(), is_error: true },
        };
        let old = match input.get("old_string").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return ToolResult { id: String::new(), output: "missing old_string".into(), is_error: true },
        };
        let new = match input.get("new_string").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return ToolResult { id: String::new(), output: "missing new_string".into(), is_error: true },
        };

        let content = match tokio::fs::read_to_string(path).await {
            Ok(c) => c,
            Err(e) => return ToolResult { id: String::new(), output: e.to_string(), is_error: true },
        };

        let count = content.matches(old).count();
        if count == 0 {
            return ToolResult { id: String::new(), output: "old_string not found".into(), is_error: true };
        }
        if count > 1 {
            return ToolResult { id: String::new(), output: format!("old_string found {count} times, must be unique"), is_error: true };
        }

        let updated = content.replacen(old, new, 1);
        match tokio::fs::write(path, updated).await {
            Ok(()) => ToolResult { id: String::new(), output: format!("edited {path}"), is_error: false },
            Err(e) => ToolResult { id: String::new(), output: e.to_string(), is_error: true },
        }
    }
}
