use std::future::Future;
use std::pin::Pin;

use anyhow::Result;

use super::{Tool, ToolContext, ToolDefinition, ToolResult};

pub struct FileWriteTool;

impl Tool for FileWriteTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "file_write".to_string(),
            description: "Write content to a file, creating it if it doesn't exist".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The file path to write to"
                    },
                    "content": {
                        "type": "string",
                        "description": "The content to write"
                    }
                },
                "required": ["path", "content"]
            }),
        }
    }

    fn execute(
        &self,
        input: serde_json::Value,
        ctx: &ToolContext,
    ) -> Pin<Box<dyn Future<Output = Result<ToolResult>> + Send + '_>> {
        let path = input["path"].as_str().unwrap_or("").to_string();
        let content = input["content"].as_str().unwrap_or("").to_string();
        let cwd = ctx.working_dir.clone();

        Box::pin(async move {
            let full_path = if std::path::Path::new(&path).is_absolute() {
                std::path::PathBuf::from(&path)
            } else {
                cwd.join(&path)
            };

            if let Some(parent) = full_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            match tokio::fs::write(&full_path, &content).await {
                Ok(()) => Ok(ToolResult::ok(format!(
                    "wrote {} bytes to {}",
                    content.len(),
                    full_path.display()
                ))),
                Err(e) => Ok(ToolResult::err(format!(
                    "failed to write {}: {e}",
                    full_path.display()
                ))),
            }
        })
    }
}
