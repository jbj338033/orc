use std::pin::Pin;

use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures::stream::StreamExt;
use futures::Stream;
use orc_core::provider::{
    CompletionProvider, CompletionRequest, ErrorCode, FinishReason, ProviderError, StreamPart,
};
use tokio_util::sync::CancellationToken;

pub struct OpenAiProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: "https://api.openai.com".into(),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    fn build_body(&self, request: &CompletionRequest) -> serde_json::Value {
        let mut messages: Vec<serde_json::Value> = Vec::new();

        if let Some(system) = &request.system {
            messages.push(serde_json::json!({"role": "system", "content": system}));
        }

        for msg in &request.messages {
            let content: Vec<serde_json::Value> = msg
                .content
                .iter()
                .filter_map(|block| match block {
                    orc_core::provider::ContentBlock::Text { text } => {
                        Some(serde_json::json!({"type": "text", "text": text}))
                    }
                    orc_core::provider::ContentBlock::ToolResult { id, output, .. } => {
                        // OpenAI uses separate tool messages
                        Some(serde_json::json!({"type": "text", "text": output, "_tool_call_id": id}))
                    }
                    _ => None,
                })
                .collect();

            // handle tool_use blocks as tool_calls in assistant message
            let tool_calls: Vec<serde_json::Value> = msg
                .content
                .iter()
                .filter_map(|block| match block {
                    orc_core::provider::ContentBlock::ToolUse { id, name, input } => {
                        Some(serde_json::json!({
                            "id": id,
                            "type": "function",
                            "function": { "name": name, "arguments": input.to_string() }
                        }))
                    }
                    _ => None,
                })
                .collect();

            // handle tool results as separate "tool" role messages
            let tool_results: Vec<&orc_core::provider::ContentBlock> = msg
                .content
                .iter()
                .filter(|b| matches!(b, orc_core::provider::ContentBlock::ToolResult { .. }))
                .collect();

            if !tool_results.is_empty() {
                for tr in tool_results {
                    if let orc_core::provider::ContentBlock::ToolResult { id, output, .. } = tr {
                        messages.push(serde_json::json!({
                            "role": "tool",
                            "tool_call_id": id,
                            "content": output
                        }));
                    }
                }
            } else if !tool_calls.is_empty() {
                let text_content: String = content
                    .iter()
                    .filter_map(|c| c.get("text").and_then(|t| t.as_str()))
                    .collect::<Vec<_>>()
                    .join("");
                let mut msg_json = serde_json::json!({
                    "role": msg.role,
                    "tool_calls": tool_calls
                });
                if !text_content.is_empty() {
                    msg_json["content"] = serde_json::json!(text_content);
                }
                messages.push(msg_json);
            } else {
                let text: String = content
                    .iter()
                    .filter_map(|c| c.get("text").and_then(|t| t.as_str()))
                    .collect::<Vec<_>>()
                    .join("");
                messages.push(serde_json::json!({"role": msg.role, "content": text}));
            }
        }

        let mut body = serde_json::json!({
            "model": request.model,
            "messages": messages,
            "stream": true,
            "stream_options": { "include_usage": true },
        });

        if let Some(max) = request.max_tokens {
            body["max_tokens"] = serde_json::json!(max);
        }
        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        if !request.tools.is_empty() {
            let tools: Vec<serde_json::Value> = request
                .tools
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "description": t.description,
                            "parameters": t.input_schema,
                        }
                    })
                })
                .collect();
            body["tools"] = serde_json::json!(tools);
        }

        body
    }
}

fn parse_sse_chunk(data: &str) -> Option<StreamPart> {
    if data == "[DONE]" {
        return Some(StreamPart::Finish(FinishReason::Stop));
    }

    let v: serde_json::Value = serde_json::from_str(data).ok()?;

    // usage chunk
    if let Some(usage) = v.get("usage") {
        let input = usage.get("prompt_tokens").and_then(|t| t.as_u64()).unwrap_or(0) as u32;
        let output = usage.get("completion_tokens").and_then(|t| t.as_u64()).unwrap_or(0) as u32;
        return Some(StreamPart::Usage {
            input_tokens: input,
            output_tokens: output,
        });
    }

    let choice = v.get("choices")?.get(0)?;
    let delta = choice.get("delta")?;

    // finish reason
    if let Some(reason) = choice.get("finish_reason").and_then(|r| r.as_str()) {
        return match reason {
            "stop" => Some(StreamPart::Finish(FinishReason::Stop)),
            "length" => Some(StreamPart::Finish(FinishReason::MaxTokens)),
            "tool_calls" => Some(StreamPart::Finish(FinishReason::ToolUse)),
            _ => Some(StreamPart::Finish(FinishReason::Stop)),
        };
    }

    // text delta
    if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
        if !content.is_empty() {
            return Some(StreamPart::TextDelta(content.to_string()));
        }
    }

    // tool call delta
    if let Some(tool_calls) = delta.get("tool_calls").and_then(|tc| tc.as_array()) {
        if let Some(tc) = tool_calls.first() {
            let id = tc.get("id").and_then(|i| i.as_str()).unwrap_or("").to_string();
            let name = tc
                .get("function")
                .and_then(|f| f.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string();
            let args = tc
                .get("function")
                .and_then(|f| f.get("arguments"))
                .and_then(|a| a.as_str())
                .unwrap_or("")
                .to_string();

            return Some(StreamPart::ToolCallDelta {
                id,
                name,
                args_delta: args,
            });
        }
    }

    None
}

#[async_trait]
impl CompletionProvider for OpenAiProvider {
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
            .post(format!("{}/v1/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
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
                401 => ErrorCode::AuthFailed,
                _ => ErrorCode::Unknown,
            };
            let retriable = matches!(code, ErrorCode::RateLimit);
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
                    Ok(event) => parse_sse_chunk(&event.data).map(Ok),
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
        "openai"
    }
}
