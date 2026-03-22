use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;

use anyhow::{Context, Result};
use futures::Stream;
use futures::stream::StreamExt;

use super::{
    ConfigField, FieldType, Message, ModelInfo, Provider, ProviderFactory, Role, SseParser,
    StreamEvent,
    oauth::{self, OAuthTokens},
};
use crate::config::ProviderEntry;
use crate::tool::ToolDefinition;

const MODELS: &[(&str, &str, u64)] = &[
    ("claude-opus-4-6", "Claude Opus 4.6", 1_000_000),
    ("claude-sonnet-4-6", "Claude Sonnet 4.6", 1_000_000),
    (
        "claude-haiku-4-5-20251001",
        "Claude Haiku 4.5",
        200_000,
    ),
];

pub struct AnthropicFactory;

impl ProviderFactory for AnthropicFactory {
    fn id(&self) -> &'static str {
        "anthropic"
    }

    fn display_name(&self) -> &'static str {
        "Anthropic (Claude)"
    }

    fn config_fields(&self) -> Vec<ConfigField> {
        vec![
            ConfigField {
                key: "auth_method",
                label: "Auth Method",
                field_type: FieldType::Select(vec!["api_key", "oauth"]),
                required: true,
                default: Some("api_key"),
            },
            ConfigField {
                key: "api_key_env",
                label: "API Key Env Var",
                field_type: FieldType::Text,
                required: false,
                default: Some("ANTHROPIC_API_KEY"),
            },
        ]
    }

    fn create(&self, entry: &ProviderEntry) -> Result<Box<dyn Provider>> {
        let auth_method = entry.auth.method.as_deref().unwrap_or("api_key");

        match auth_method {
            "oauth" => {
                let tokens = oauth::load_tokens(&entry.id)?;
                Ok(Box::new(AnthropicProvider {
                    name: entry.id.clone(),
                    auth: AuthMethod::OAuth {
                        tokens: Mutex::new(tokens),
                        provider_id: entry.id.clone(),
                    },
                }))
            }
            _ => {
                let api_key = entry
                    .auth
                    .resolve_api_key()
                    .context("anthropic api key not found")?;
                Ok(Box::new(AnthropicProvider {
                    name: entry.id.clone(),
                    auth: AuthMethod::ApiKey(api_key),
                }))
            }
        }
    }
}

enum AuthMethod {
    ApiKey(String),
    OAuth {
        tokens: Mutex<Option<OAuthTokens>>,
        provider_id: String,
    },
}

struct AnthropicProvider {
    name: String,
    auth: AuthMethod,
}

impl AnthropicProvider {
    async fn get_auth_header(&self) -> Result<(&'static str, String)> {
        match &self.auth {
            AuthMethod::ApiKey(key) => Ok(("x-api-key", key.clone())),
            AuthMethod::OAuth {
                tokens,
                provider_id,
            } => {
                let current = { tokens.lock().unwrap().clone() };

                match current {
                    Some(t) if !t.is_expired() => {
                        Ok(("Authorization", format!("Bearer {}", t.access_token)))
                    }
                    Some(t) if t.refresh_token.is_some() => {
                        let refresh = t.refresh_token.as_ref().unwrap();
                        let new_tokens =
                            oauth::refresh_access_token(provider_id, refresh).await?;
                        let header = format!("Bearer {}", new_tokens.access_token);
                        *tokens.lock().unwrap() = Some(new_tokens);
                        Ok(("Authorization", header))
                    }
                    _ => {
                        let new_tokens = oauth::run_oauth_flow(provider_id).await?;
                        let header = format!("Bearer {}", new_tokens.access_token);
                        *tokens.lock().unwrap() = Some(new_tokens);
                        Ok(("Authorization", header))
                    }
                }
            }
        }
    }
}

impl Provider for AnthropicProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn models(&self) -> Vec<ModelInfo> {
        MODELS
            .iter()
            .map(|(id, name, ctx)| ModelInfo {
                id: id.to_string(),
                display_name: name.to_string(),
                context_window: Some(*ctx),
            })
            .collect()
    }

    fn stream(
        &self,
        model: &str,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<Pin<Box<dyn Stream<Item = StreamEvent> + Send>>>,
                > + Send
                + '_,
        >,
    > {
        let model = model.to_string();
        let messages = messages.to_vec();
        let tools = tools.to_vec();

        Box::pin(async move {
            let (header_name, header_value) = self.get_auth_header().await?;

            // system 메시지 분리
            let system_text: Option<String> = messages
                .iter()
                .filter(|m| matches!(m.role, Role::System))
                .map(|m| m.text())
                .reduce(|a, b| format!("{a}\n{b}"));

            let api_messages = messages
                .iter()
                .filter(|m| !matches!(m.role, Role::System))
                .map(|m| build_anthropic_message(m))
                .collect::<Vec<_>>();

            let mut body = serde_json::json!({
                "model": model,
                "max_tokens": 8192,
                "stream": true,
                "messages": api_messages,
            });

            if let Some(sys) = system_text {
                body["system"] = serde_json::Value::String(sys);
            }

            if !tools.is_empty() {
                let tool_defs: Vec<_> = tools
                    .iter()
                    .map(|t| {
                        serde_json::json!({
                            "name": t.name,
                            "description": t.description,
                            "input_schema": t.input_schema,
                        })
                    })
                    .collect();
                body["tools"] = serde_json::Value::Array(tool_defs);
            }

            let response = reqwest::Client::new()
                .post("https://api.anthropic.com/v1/messages")
                .header(header_name, &header_value)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await
                .context("failed to send request to anthropic")?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("anthropic api error {status}: {text}");
            }

            let byte_stream = response.bytes_stream();
            let mut parser = SseParser::new();
            // block type 추적: index → is_tool_use
            let mut block_types: HashMap<u64, bool> = HashMap::new();

            let event_stream = byte_stream.flat_map(move |chunk| {
                let events = match chunk {
                    Ok(bytes) => parser
                        .feed(&bytes)
                        .into_iter()
                        .filter_map(|sse| {
                            parse_anthropic_sse(&sse, &mut block_types)
                        })
                        .collect::<Vec<_>>(),
                    Err(e) => vec![StreamEvent::Error(e.to_string())],
                };
                futures::stream::iter(events)
            });

            Ok(Box::pin(event_stream) as Pin<Box<dyn Stream<Item = StreamEvent> + Send>>)
        })
    }
}

/// Anthropic API 메시지 형식으로 변환
fn build_anthropic_message(msg: &Message) -> serde_json::Value {
    use super::ContentBlock;

    let content: Vec<serde_json::Value> = msg
        .content
        .iter()
        .map(|block| match block {
            ContentBlock::Text { text } => {
                serde_json::json!({ "type": "text", "text": text })
            }
            ContentBlock::ToolUse { id, name, input } => {
                serde_json::json!({
                    "type": "tool_use",
                    "id": id,
                    "name": name,
                    "input": input,
                })
            }
            ContentBlock::ToolResult {
                tool_use_id,
                content,
                is_error,
            } => {
                let mut val = serde_json::json!({
                    "type": "tool_result",
                    "tool_use_id": tool_use_id,
                    "content": [{ "type": "text", "text": content }],
                });
                if *is_error {
                    val["is_error"] = serde_json::Value::Bool(true);
                }
                val
            }
        })
        .collect();

    serde_json::json!({
        "role": msg.role,
        "content": content,
    })
}

fn parse_anthropic_sse(
    sse: &super::SseEvent,
    block_types: &mut HashMap<u64, bool>,
) -> Option<StreamEvent> {
    let event_type = sse.event_type.as_deref()?;
    let json = sse.json()?;

    match event_type {
        "content_block_start" => {
            let index = json["index"].as_u64().unwrap_or(0);
            let block = &json["content_block"];
            let block_type = block["type"].as_str().unwrap_or("");
            let is_tool = block_type == "tool_use";
            block_types.insert(index, is_tool);

            if is_tool {
                Some(StreamEvent::ToolUseStart {
                    id: block["id"].as_str().unwrap_or("").to_string(),
                    name: block["name"].as_str().unwrap_or("").to_string(),
                })
            } else {
                None
            }
        }
        "content_block_delta" => {
            let delta = &json["delta"];
            match delta["type"].as_str()? {
                "text_delta" => {
                    Some(StreamEvent::Delta(delta["text"].as_str()?.to_string()))
                }
                "input_json_delta" => Some(StreamEvent::ToolUseInput(
                    delta["partial_json"].as_str()?.to_string(),
                )),
                _ => None,
            }
        }
        "content_block_stop" => {
            let index = json["index"].as_u64().unwrap_or(0);
            if block_types.get(&index).copied().unwrap_or(false) {
                Some(StreamEvent::ToolUseEnd)
            } else {
                None
            }
        }
        "message_stop" => Some(StreamEvent::Done),
        "error" => Some(StreamEvent::Error(
            json["error"]["message"]
                .as_str()
                .unwrap_or("unknown error")
                .to_string(),
        )),
        _ => None,
    }
}
