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

const DEFAULT_BASE_URL: &str = "https://api.openai.com";

const MODELS: &[(&str, &str, u64)] = &[
    ("gpt-5.4-2026-03-05", "GPT-5.4", 1_000_000),
    ("gpt-5.4-mini-2026-03-17", "GPT-5.4 Mini", 400_000),
    ("gpt-5.4-nano", "GPT-5.4 Nano", 400_000),
    ("o3", "O3", 200_000),
    ("o4-mini", "O4 Mini", 200_000),
    ("codex-mini-latest", "Codex Mini", 200_000),
];

pub struct OpenAiFactory;

impl ProviderFactory for OpenAiFactory {
    fn id(&self) -> &'static str {
        "openai"
    }

    fn display_name(&self) -> &'static str {
        "OpenAI (GPT / Codex)"
    }

    fn config_fields(&self) -> Vec<ConfigField> {
        vec![
            ConfigField {
                key: "api_key_env",
                label: "API Key Env Var",
                field_type: FieldType::Text,
                required: true,
                default: Some("OPENAI_API_KEY"),
            },
            ConfigField {
                key: "base_url",
                label: "Base URL",
                field_type: FieldType::Text,
                required: false,
                default: Some(DEFAULT_BASE_URL),
            },
        ]
    }

    fn create(&self, entry: &ProviderEntry) -> Result<Box<dyn Provider>> {
        let api_key = entry
            .auth
            .resolve_api_key()
            .context("openai api key not found")?;
        let base_url = entry
            .base_url
            .clone()
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        Ok(Box::new(OpenAiProvider {
            name: entry.id.clone(),
            api_key,
            base_url,
        }))
    }
}

pub(crate) struct OpenAiProvider {
    pub name: String,
    pub api_key: String,
    pub base_url: String,
}

impl OpenAiProvider {
    pub fn with_models(
        name: String,
        api_key: String,
        base_url: String,
    ) -> Self {
        Self {
            name,
            api_key,
            base_url,
        }
    }
}

impl Provider for OpenAiProvider {
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
        let url = format!("{}/v1/chat/completions", self.base_url);

        Box::pin(async move {
            let api_messages: Vec<_> = messages
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "role": m.role,
                        "content": m.text(),
                    })
                })
                .collect();

            let mut body = serde_json::json!({
                "model": model,
                "stream": true,
                "messages": api_messages,
            });

            if !tools.is_empty() {
                let tool_defs: Vec<_> = tools
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
                body["tools"] = serde_json::Value::Array(tool_defs);
            }

            let response = reqwest::Client::new()
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await
                .context("failed to send request to openai")?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("openai api error {status}: {text}");
            }

            let byte_stream = response.bytes_stream();
            let mut parser = SseParser::new();

            let event_stream = byte_stream.flat_map(move |chunk| {
                let events = match chunk {
                    Ok(bytes) => parser
                        .feed(&bytes)
                        .into_iter()
                        .flat_map(|sse| parse_openai_sse(&sse))
                        .collect::<Vec<_>>(),
                    Err(e) => vec![StreamEvent::Error(e.to_string())],
                };
                futures::stream::iter(events)
            });

            Ok(Box::pin(event_stream) as Pin<Box<dyn Stream<Item = StreamEvent> + Send>>)
        })
    }
}

fn parse_openai_sse(sse: &super::SseEvent) -> Vec<StreamEvent> {
    if sse.data.trim() == "[DONE]" {
        return vec![StreamEvent::Done];
    }

    let json = match sse.json() {
        Some(j) => j,
        None => return vec![],
    };
    let choice = match json["choices"].get(0) {
        Some(c) => c,
        None => return vec![],
    };
    let delta = &choice["delta"];
    let mut events = Vec::new();

    if let Some(content) = delta["content"].as_str() {
        if !content.is_empty() {
            events.push(StreamEvent::Delta(content.to_string()));
        }
    }

    if let Some(tool_calls) = delta["tool_calls"].as_array() {
        for tc in tool_calls {
            if let Some(function) = tc.get("function") {
                // 첫 chunk: id + name
                if let Some(name) = function["name"].as_str() {
                    if !name.is_empty() {
                        let id = tc["id"].as_str().unwrap_or("").to_string();
                        events.push(StreamEvent::ToolUseStart { id, name: name.to_string() });
                    }
                }
                // 이어지는 chunk: arguments (점진적)
                if let Some(args) = function["arguments"].as_str() {
                    if !args.is_empty() {
                        events.push(StreamEvent::ToolUseInput(args.to_string()));
                    }
                }
            }
        }
    }

    if choice["finish_reason"].as_str() == Some("tool_calls") {
        events.push(StreamEvent::ToolUseEnd);
    }

    events
}
