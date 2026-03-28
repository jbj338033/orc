use async_trait::async_trait;
use orc_core::provider::ToolDef;
use tokio_util::sync::CancellationToken;

use crate::runtime::ToolResult;
use crate::tool::Tool;

pub struct WebFetch;

#[async_trait]
impl Tool for WebFetch {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "web_fetch".into(),
            description: "fetch content from a url and return as text.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": { "type": "string", "description": "url to fetch" }
                },
                "required": ["url"]
            }),
        }
    }

    async fn execute(&self, input: serde_json::Value, cancel: CancellationToken) -> ToolResult {
        let url = match input.get("url").and_then(|v| v.as_str()) {
            Some(u) => u,
            None => return ToolResult { id: String::new(), output: "missing url".into(), is_error: true },
        };

        let client = reqwest::Client::new();
        let result = tokio::select! {
            r = client.get(url).send() => r,
            _ = cancel.cancelled() => {
                return ToolResult { id: String::new(), output: "cancelled".into(), is_error: true };
            }
        };

        match result {
            Ok(response) => {
                let status = response.status();
                match response.text().await {
                    Ok(text) => {
                        let truncated = if text.len() > 50000 {
                            format!("{}...\n[truncated at 50000 chars]", &text[..50000])
                        } else {
                            text
                        };
                        ToolResult {
                            id: String::new(),
                            output: format!("[HTTP {status}]\n{truncated}"),
                            is_error: !status.is_success(),
                        }
                    }
                    Err(e) => ToolResult { id: String::new(), output: e.to_string(), is_error: true },
                }
            }
            Err(e) => ToolResult { id: String::new(), output: e.to_string(), is_error: true },
        }
    }
}
