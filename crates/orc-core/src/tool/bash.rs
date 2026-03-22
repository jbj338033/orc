use std::pin::Pin;
use std::future::Future;

use anyhow::Result;

use super::{Tool, ToolContext, ToolDefinition, ToolResult};

pub struct BashTool;

impl Tool for BashTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "bash".to_string(),
            description: "Run a shell command and return its output".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The shell command to execute"
                    }
                },
                "required": ["command"]
            }),
        }
    }

    fn execute(
        &self,
        input: serde_json::Value,
        ctx: &ToolContext,
    ) -> Pin<Box<dyn Future<Output = Result<ToolResult>> + Send + '_>> {
        let command = input["command"].as_str().unwrap_or("").to_string();
        let cwd = ctx.working_dir.clone();

        Box::pin(async move {
            let output = tokio::process::Command::new("sh")
                .arg("-c")
                .arg(&command)
                .current_dir(&cwd)
                .output()
                .await?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            let mut result = String::new();
            if !stdout.is_empty() {
                result.push_str(&stdout);
            }
            if !stderr.is_empty() {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str("stderr: ");
                result.push_str(&stderr);
            }

            if output.status.success() {
                Ok(ToolResult::ok(result))
            } else {
                Ok(ToolResult::err(format!(
                    "exit code {}: {result}",
                    output.status.code().unwrap_or(-1)
                )))
            }
        })
    }
}
