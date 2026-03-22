use std::future::Future;
use std::pin::Pin;

use anyhow::Result;

use super::{Tool, ToolContext, ToolDefinition, ToolResult};

pub struct FileReadTool;

impl Tool for FileReadTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "file_read".to_string(),
            description: "Read the contents of a file".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The file path to read"
                    }
                },
                "required": ["path"]
            }),
        }
    }

    fn execute(
        &self,
        input: serde_json::Value,
        ctx: &ToolContext,
    ) -> Pin<Box<dyn Future<Output = Result<ToolResult>> + Send + '_>> {
        let path = input["path"].as_str().unwrap_or("").to_string();
        let cwd = ctx.working_dir.clone();

        Box::pin(async move {
            let full_path = if std::path::Path::new(&path).is_absolute() {
                std::path::PathBuf::from(&path)
            } else {
                cwd.join(&path)
            };

            match tokio::fs::read_to_string(&full_path).await {
                Ok(content) => Ok(ToolResult::ok(content)),
                Err(e) => Ok(ToolResult::err(format!(
                    "failed to read {}: {e}",
                    full_path.display()
                ))),
            }
        })
    }
}
