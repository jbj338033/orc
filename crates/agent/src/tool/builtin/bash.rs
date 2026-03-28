use async_trait::async_trait;
use orc_core::provider::ToolDef;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use crate::runtime::ToolResult;
use crate::tool::Tool;

pub struct BashExec;

#[async_trait]
impl Tool for BashExec {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "bash".into(),
            description: "execute a shell command and return stdout/stderr.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string", "description": "shell command to execute" },
                    "timeout": { "type": "integer", "description": "timeout in seconds (default 120)" }
                },
                "required": ["command"]
            }),
        }
    }

    async fn execute(&self, input: serde_json::Value, cancel: CancellationToken) -> ToolResult {
        let command = match input.get("command").and_then(|v| v.as_str()) {
            Some(c) => c.to_string(),
            None => return ToolResult { id: String::new(), output: "missing command".into(), is_error: true },
        };
        let timeout_secs = input.get("timeout").and_then(|v| v.as_u64()).unwrap_or(120);

        let child = match Command::new("/bin/sh")
            .arg("-c")
            .arg(&command)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => return ToolResult { id: String::new(), output: format!("spawn failed: {e}"), is_error: true },
        };

        let timeout = tokio::time::sleep(std::time::Duration::from_secs(timeout_secs));
        let cancelled = cancel.cancelled();

        tokio::pin!(timeout);
        tokio::pin!(cancelled);

        let output_fut = child.wait_with_output();
        tokio::pin!(output_fut);

        let result: Result<std::process::Output, std::io::Error> = tokio::select! {
            r = &mut output_fut => r,
            _ = &mut timeout => {
                return ToolResult { id: String::new(), output: format!("command timed out after {timeout_secs}s"), is_error: true };
            }
            _ = &mut cancelled => {
                return ToolResult { id: String::new(), output: "cancelled".into(), is_error: true };
            }
        };

        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let code = output.status.code().unwrap_or(-1);
                let text = if stderr.is_empty() {
                    stdout.to_string()
                } else {
                    format!("{stdout}\n[stderr]\n{stderr}")
                };
                ToolResult {
                    id: String::new(),
                    output: format!("[exit code: {code}]\n{text}"),
                    is_error: code != 0,
                }
            }
            Err(e) => ToolResult { id: String::new(), output: e.to_string(), is_error: true },
        }
    }
}
