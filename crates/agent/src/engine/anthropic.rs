use std::pin::Pin;

use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures::stream::StreamExt;
use futures::Stream;
use orc_core::provider::{
    CompletionMessage, CompletionProvider, CompletionRequest, ErrorCode, FinishReason,
    ProviderError, StreamPart,
};
use tokio_util::sync::CancellationToken;

pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: "https://api.anthropic.com".into(),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    fn build_body(&self, request: &CompletionRequest) -> serde_json::Value {
        let mut body = serde_json::json!({
            "model": request.model,
            "max_tokens": request.max_tokens.unwrap_or(4096),
            "stream": true,
        });

        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        if let Some(system) = &request.system {
            body["system"] = serde_json::json!(system);
        }

        let messages: Vec<serde_json::Value> = request
            .messages
            .iter()
            .map(|m| message_to_json(m))
            .collect();
        body["messages"] = serde_json::json!(messages);

        if !request.tools.is_empty() {
            let tools: Vec<serde_json::Value> = request
                .tools
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "name": t.name,
                        "description": t.description,
                        "input_schema": t.input_schema,
                    })
                })
                .collect();
            body["tools"] = serde_json::json!(tools);
        }

        body
    }
}

fn message_to_json(msg: &CompletionMessage) -> serde_json::Value {
    let content: Vec<serde_json::Value> = msg
        .content
        .iter()
        .map(|block| match block {
            orc_core::provider::ContentBlock::Text { text } => {
                serde_json::json!({"type": "text", "text": text})
            }
            orc_core::provider::ContentBlock::ToolUse { id, name, input } => {
                serde_json::json!({"type": "tool_use", "id": id, "name": name, "input": input})
            }
            orc_core::provider::ContentBlock::ToolResult {
                id,
                output,
                is_error,
            } => {
                serde_json::json!({"type": "tool_result", "tool_use_id": id, "content": output, "is_error": is_error})
            }
        })
        .collect();

    serde_json::json!({
        "role": msg.role,
        "content": content,
    })
}

fn parse_sse_event(event_type: &str, data: &str) -> Option<StreamPart> {
    match event_type {
        "content_block_delta" => {
            let v: serde_json::Value = serde_json::from_str(data).ok()?;
            let delta = v.get("delta")?;
            match delta.get("type")?.as_str()? {
                "text_delta" => {
                    let text = delta.get("text")?.as_str()?.to_string();
                    Some(StreamPart::TextDelta(text))
                }
                "thinking_delta" => {
                    let text = delta.get("thinking")?.as_str()?.to_string();
                    Some(StreamPart::ReasoningDelta(text))
                }
                "input_json_delta" => {
                    let partial = delta.get("partial_json")?.as_str()?.to_string();
                    let index = v.get("index")?.as_u64()? as usize;
                    // We need content_block_start to know the tool id/name
                    // For now, emit a delta with index as placeholder
                    Some(StreamPart::ToolCallDelta {
                        id: format!("pending_{index}"),
                        name: String::new(),
                        args_delta: partial,
                    })
                }
                _ => None,
            }
        }
        "content_block_start" => {
            let v: serde_json::Value = serde_json::from_str(data).ok()?;
            let block = v.get("content_block")?;
            if block.get("type")?.as_str()? == "tool_use" {
                let id = block.get("id")?.as_str()?.to_string();
                let name = block.get("name")?.as_str()?.to_string();
                Some(StreamPart::ToolCallDelta {
                    id,
                    name,
                    args_delta: String::new(),
                })
            } else {
                None
            }
        }
        "content_block_stop" => None,
        "message_delta" => {
            let v: serde_json::Value = serde_json::from_str(data).ok()?;
            let delta = v.get("delta")?;
            let reason = match delta.get("stop_reason")?.as_str()? {
                "end_turn" | "stop_sequence" => FinishReason::Stop,
                "max_tokens" => FinishReason::MaxTokens,
                "tool_use" => FinishReason::ToolUse,
                _ => FinishReason::Stop,
            };

            let usage = v.get("usage");
            if let Some(u) = usage {
                let output = u.get("output_tokens").and_then(|t| t.as_u64()).unwrap_or(0) as u32;
                return Some(StreamPart::Usage {
                    input_tokens: 0,
                    output_tokens: output,
                });
            }

            Some(StreamPart::Finish(reason))
        }
        "message_start" => {
            let v: serde_json::Value = serde_json::from_str(data).ok()?;
            let usage = v.get("message")?.get("usage")?;
            let input = usage
                .get("input_tokens")
                .and_then(|t| t.as_u64())
                .unwrap_or(0) as u32;
            Some(StreamPart::Usage {
                input_tokens: input,
                output_tokens: 0,
            })
        }
        "message_stop" => Some(StreamPart::Finish(FinishReason::Stop)),
        "error" => {
            let v: serde_json::Value = serde_json::from_str(data).ok()?;
            let msg = v
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("unknown error")
                .to_string();
            Some(StreamPart::Error(ProviderError {
                code: ErrorCode::Unknown,
                message: msg,
                retriable: false,
            }))
        }
        "ping" => None,
        _ => None,
    }
}

#[async_trait]
impl CompletionProvider for AnthropicProvider {
    async fn stream(
        &self,
        request: CompletionRequest,
        cancel: CancellationToken,
    ) -> Result<
        Pin<Box<dyn Stream<Item = Result<StreamPart, ProviderError>> + Send>>,
        ProviderError,
    > {
        let body = self.build_body(&request);

        let response = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError {
                code: ErrorCode::NetworkError,
                message: e.to_string(),
                retriable: true,
            })?;

        let status = response.status();
        if !status.is_success() {
            let code = match status.as_u16() {
                429 => ErrorCode::RateLimit,
                529 => ErrorCode::Overloaded,
                401 => ErrorCode::AuthFailed,
                _ => ErrorCode::Unknown,
            };
            let retriable = matches!(code, ErrorCode::RateLimit | ErrorCode::Overloaded);
            let body_text = response.text().await.unwrap_or_default();
            return Err(ProviderError {
                code,
                message: format!("HTTP {status}: {body_text}"),
                retriable,
            });
        }

        let byte_stream = response.bytes_stream();
        let event_stream = byte_stream.eventsource();

        let stream = event_stream.filter_map(move |result| {
            let cancel = cancel.clone();
            async move {
                if cancel.is_cancelled() {
                    return Some(Err(ProviderError {
                        code: ErrorCode::Unknown,
                        message: "cancelled".into(),
                        retriable: false,
                    }));
                }

                match result {
                    Ok(event) => {
                        parse_sse_event(&event.event, &event.data).map(Ok)
                    }
                    Err(e) => Some(Err(ProviderError {
                        code: ErrorCode::NetworkError,
                        message: e.to_string(),
                        retriable: true,
                    })),
                }
            }
        });

        Ok(Box::pin(stream))
    }

    fn provider_name(&self) -> &str {
        "anthropic"
    }
}
