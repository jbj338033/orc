use std::future::Future;
use std::pin::Pin;

use anyhow::Result;

use super::{Tool, ToolContext, ToolDefinition, ToolResult};

pub struct GlobTool;

impl Tool for GlobTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "glob".to_string(),
            description: "Find files matching a glob pattern".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "The glob pattern (e.g., '**/*.rs', 'src/**/*.ts')"
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
        let cwd = ctx.working_dir.clone();

        Box::pin(async move {
            let full_pattern = if std::path::Path::new(&pattern).is_absolute() {
                pattern
            } else {
                format!("{}/{}", cwd.display(), pattern)
            };

            match glob::glob(&full_pattern) {
                Ok(paths) => {
                    let mut results: Vec<String> = Vec::new();
                    for entry in paths.take(200) {
                        if let Ok(path) = entry {
                            results.push(path.display().to_string());
                        }
                    }
                    if results.is_empty() {
                        Ok(ToolResult::ok("no files matched"))
                    } else {
                        Ok(ToolResult::ok(results.join("\n")))
                    }
                }
                Err(e) => Ok(ToolResult::err(format!("invalid glob pattern: {e}"))),
            }
        })
    }
}
