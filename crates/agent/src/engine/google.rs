use std::pin::Pin;

use async_trait::async_trait;
use futures::stream::StreamExt;
use futures::Stream;
use orc_core::provider::{
    CompletionProvider, CompletionRequest, ErrorCode, FinishReason, ProviderError, StreamPart,
};
use tokio_util::sync::CancellationToken;

pub struct GeminiProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl GeminiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: "https://generativelanguage.googleapis.com".into(),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    fn build_body(&self, request: &CompletionRequest) -> serde_json::Value {
        let mut contents: Vec<serde_json::Value> = Vec::new();

        for msg in &request.messages {
            let role = match msg.role.as_str() {
                "assistant" => "model",
                _ => "user",
            };

            let parts: Vec<serde_json::Value> = msg
                .content
                .iter()
                .filter_map(|block| match block {
                    orc_core::provider::ContentBlock::Text { text } => {
                        Some(serde_json::json!({"text": text}))
                    }
                    orc_core::provider::ContentBlock::ToolUse { name, input, .. } => {
                        Some(serde_json::json!({
                            "functionCall": { "name": name, "args": input }
                        }))
                    }
                    orc_core::provider::ContentBlock::ToolResult { output, .. } => {
                        Some(serde_json::json!({
                            "functionResponse": { "name": "", "response": { "result": output } }
                        }))
                    }
                })
                .collect();

            contents.push(serde_json::json!({"role": role, "parts": parts}));
        }

        let mut body = serde_json::json!({
            "contents": contents,
            "generationConfig": {},
        });

        if let Some(system) = &request.system {
            body["systemInstruction"] = serde_json::json!({
                "parts": [{"text": system}]
            });
        }

        if let Some(max) = request.max_tokens {
            body["generationConfig"]["maxOutputTokens"] = serde_json::json!(max);
        }
        if let Some(temp) = request.temperature {
            body["generationConfig"]["temperature"] = serde_json::json!(temp);
        }

        if !request.tools.is_empty() {
            let functions: Vec<serde_json::Value> = request
                .tools
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.input_schema,
                    })
                })
                .collect();
            body["tools"] = serde_json::json!([{"functionDeclarations": functions}]);
        }

        body
    }
}

fn parse_gemini_chunk(data: &[u8]) -> Option<StreamPart> {
    let v: serde_json::Value = serde_json::from_slice(data).ok()?;

    // array response from SSE
    let candidate = if v.is_array() {
        v.get(0)?.get("candidates")?.get(0)?
    } else {
        v.get("candidates")?.get(0)?
    };

    let content = candidate.get("content")?;
    let parts = content.get("parts")?.as_array()?;

    for part in parts {
        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
            return Some(StreamPart::TextDelta(text.to_string()));
        }
        if let Some(fc) = part.get("functionCall") {
            let name = fc.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
            let args = fc.get("args").cloned().unwrap_or(serde_json::json!({}));
            return Some(StreamPart::ToolCallComplete {
                id: format!("gemini_{name}"),
                name,
                args,
            });
        }
    }

    // finish reason
    if let Some(reason) = candidate.get("finishReason").and_then(|r| r.as_str()) {
        return match reason {
            "STOP" => Some(StreamPart::Finish(FinishReason::Stop)),
            "MAX_TOKENS" => Some(StreamPart::Finish(FinishReason::MaxTokens)),
            _ => Some(StreamPart::Finish(FinishReason::Stop)),
        };
    }

    // usage
    if let Some(usage) = v.get("usageMetadata").or_else(|| {
        if v.is_array() { v.get(0)?.get("usageMetadata") } else { None }
    }) {
        let input = usage.get("promptTokenCount").and_then(|t| t.as_u64()).unwrap_or(0) as u32;
        let output = usage.get("candidatesTokenCount").and_then(|t| t.as_u64()).unwrap_or(0) as u32;
        return Some(StreamPart::Usage {
            input_tokens: input,
            output_tokens: output,
        });
    }

    None
}

#[async_trait]
impl CompletionProvider for GeminiProvider {
    async fn stream(
        &self,
        request: CompletionRequest,
        cancel: CancellationToken,
    ) -> Result<
        Pin<Box<dyn Stream<Item = Result<StreamPart, ProviderError>> + Send>>,
        ProviderError,
    > {
        let body = self.build_body(&request);
        let model = &request.model;

        let response = self
            .client
            .post(format!(
                "{}/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
                self.base_url, model, self.api_key
            ))
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
                401 | 403 => ErrorCode::AuthFailed,
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

        let stream = byte_stream.filter_map(move |result| {
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
                    Ok(bytes) => parse_gemini_chunk(&bytes).map(Ok),
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
        "google"
    }
}
