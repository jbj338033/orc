use std::future::Future;
use std::pin::Pin;

use anyhow::Result;

use super::{Tool, ToolContext, ToolDefinition, ToolResult};

pub struct FileEditTool;

impl Tool for FileEditTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "file_edit".to_string(),
            description: "Replace a string in a file with a new string".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The file path to edit"
                    },
                    "old_string": {
                        "type": "string",
                        "description": "The exact string to find and replace"
                    },
                    "new_string": {
                        "type": "string",
                        "description": "The replacement string"
                    }
                },
                "required": ["path", "old_string", "new_string"]
            }),
        }
    }

    fn execute(
        &self,
        input: serde_json::Value,
        ctx: &ToolContext,
    ) -> Pin<Box<dyn Future<Output = Result<ToolResult>> + Send + '_>> {
        let path = input["path"].as_str().unwrap_or("").to_string();
        let old_string = input["old_string"].as_str().unwrap_or("").to_string();
        let new_string = input["new_string"].as_str().unwrap_or("").to_string();
        let cwd = ctx.working_dir.clone();

        Box::pin(async move {
            let full_path = if std::path::Path::new(&path).is_absolute() {
                std::path::PathBuf::from(&path)
            } else {
                cwd.join(&path)
            };

            let content = match tokio::fs::read_to_string(&full_path).await {
                Ok(c) => c,
                Err(e) => {
                    return Ok(ToolResult::err(format!(
                        "failed to read {}: {e}",
                        full_path.display()
                    )));
                }
            };

            let count = content.matches(&old_string).count();
            if count == 0 {
                return Ok(ToolResult::err("old_string not found in file"));
            }
            if count > 1 {
                return Ok(ToolResult::err(format!(
                    "old_string found {count} times, must be unique"
                )));
            }

            let new_content = content.replacen(&old_string, &new_string, 1);
            match tokio::fs::write(&full_path, &new_content).await {
                Ok(()) => Ok(ToolResult::ok(format!(
                    "edited {}",
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
