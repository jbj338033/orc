use std::future::Future;
use std::pin::Pin;

use anyhow::{Context, Result};
use futures::stream::StreamExt;
use futures::Stream;

use super::{
    ConfigField, FieldType, Message, ModelInfo, Provider, ProviderFactory, SseParser, StreamEvent,
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
        let api_key = entry
            .auth
            .resolve_api_key()
            .context("anthropic api key not found")?;
        Ok(Box::new(AnthropicProvider {
            name: entry.id.clone(),
            api_key,
        }))
    }
}

struct AnthropicProvider {
    name: String,
    api_key: String,
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
    ) -> Pin<Box<dyn Future<Output = Result<Pin<Box<dyn Stream<Item = StreamEvent> + Send>>>> + Send + '_>>
    {
        let model = model.to_string();
        let messages = messages.to_vec();
        let tools = tools.to_vec();

        Box::pin(async move {
            let api_messages = messages
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "role": m.role,
                        "content": m.content,
                    })
                })
                .collect::<Vec<_>>();

            let mut body = serde_json::json!({
                "model": model,
                "max_tokens": 8192,
                "stream": true,
                "messages": api_messages,
            });

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
                .header("x-api-key", &self.api_key)
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

            let event_stream = byte_stream.flat_map(move |chunk| {
                let events = match chunk {
                    Ok(bytes) => parser
                        .feed(&bytes)
                        .into_iter()
                        .filter_map(|sse| parse_anthropic_sse(&sse))
                        .collect::<Vec<_>>(),
                    Err(e) => vec![StreamEvent::Error(e.to_string())],
                };
                futures::stream::iter(events)
            });

            Ok(Box::pin(event_stream) as Pin<Box<dyn Stream<Item = StreamEvent> + Send>>)
        })
    }
}

fn parse_anthropic_sse(sse: &super::SseEvent) -> Option<StreamEvent> {
    let event_type = sse.event_type.as_deref()?;
    let json = sse.json()?;

    match event_type {
        "content_block_start" => {
            let block = &json["content_block"];
            match block["type"].as_str()? {
                "tool_use" => Some(StreamEvent::ToolUseStart {
                    id: block["id"].as_str()?.to_string(),
                    name: block["name"].as_str()?.to_string(),
                }),
                _ => None,
            }
        }
        "content_block_delta" => {
            let delta = &json["delta"];
            match delta["type"].as_str()? {
                "text_delta" => Some(StreamEvent::Delta(
                    delta["text"].as_str()?.to_string(),
                )),
                "input_json_delta" => Some(StreamEvent::ToolUseInput(
                    delta["partial_json"].as_str()?.to_string(),
                )),
                _ => None,
            }
        }
        "content_block_stop" => {
            let index = json["index"].as_u64()?;
            if index > 0 {
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
