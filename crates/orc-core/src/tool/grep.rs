use std::future::Future;
use std::pin::Pin;

use anyhow::Result;

use super::{Tool, ToolContext, ToolDefinition, ToolResult};

pub struct GrepTool;

impl Tool for GrepTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "grep".to_string(),
            description: "Search file contents using a regex pattern".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "The regex pattern to search for"
                    },
                    "path": {
                        "type": "string",
                        "description": "The directory or file to search in (defaults to current directory)"
                    }
                },
                "required": ["pattern"]
            }),
        }
    }

    fn execute(
        &self,
        input: serde_json::Value,
        ctx: &ToolContext,
    ) -> Pin<Box<dyn Future<Output = Result<ToolResult>> + Send + '_>> {
        let pattern = input["pattern"].as_str().unwrap_or("").to_string();
        let path = input["path"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| ".".to_string());
        let cwd = ctx.working_dir.clone();

        Box::pin(async move {
            let search_path = if std::path::Path::new(&path).is_absolute() {
                path
            } else {
                cwd.join(&path).to_string_lossy().to_string()
            };

            let output = tokio::process::Command::new("grep")
                .args(["-rn", "--include=*", &pattern, &search_path])
                .output()
                .await?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.is_empty() {
                Ok(ToolResult::ok("no matches found"))
            } else {
                // 결과가 너무 길면 잘라냄
                let truncated: String = stdout.lines().take(100).collect::<Vec<_>>().join("\n");
                let total = stdout.lines().count();
                if total > 100 {
                    Ok(ToolResult::ok(format!(
                        "{truncated}\n\n... ({total} total matches, showing first 100)"
                    )))
                } else {
                    Ok(ToolResult::ok(truncated))
                }
            }
        })
    }
}
