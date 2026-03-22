use std::future::Future;
use std::pin::Pin;

use anyhow::{Context, Result};
use futures::Stream;
use futures::stream::StreamExt;

use super::{
    ConfigField, FieldType, Message, ModelInfo, Provider, ProviderFactory, Role, SseParser,
    StreamEvent,
};
use crate::config::ProviderEntry;
use crate::tool::ToolDefinition;

const MODELS: &[(&str, &str, u64)] = &[
    ("gemini-3.1-pro-preview", "Gemini 3.1 Pro", 2_000_000),
    ("gemini-3-1-flash", "Gemini 3.1 Flash", 1_000_000),
    (
        "gemini-3.1-flash-lite-preview",
        "Gemini 3.1 Flash Lite",
        1_000_000,
    ),
];

pub struct GeminiFactory;

impl ProviderFactory for GeminiFactory {
    fn id(&self) -> &'static str {
        "gemini"
    }

    fn display_name(&self) -> &'static str {
        "Google Gemini"
    }

    fn config_fields(&self) -> Vec<ConfigField> {
        vec![ConfigField {
            key: "api_key_env",
            label: "API Key Env Var",
            field_type: FieldType::Text,
            required: true,
            default: Some("GEMINI_API_KEY"),
        }]
    }

    fn create(&self, entry: &ProviderEntry) -> Result<Box<dyn Provider>> {
        let api_key = entry
            .auth
            .resolve_api_key()
            .context("gemini api key not found")?;
        Ok(Box::new(GeminiProvider {
            name: entry.id.clone(),
            api_key,
        }))
    }
}

struct GeminiProvider {
    name: String,
    api_key: String,
}

impl Provider for GeminiProvider {
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
            // system 메시지 분리
            let system_text: Option<String> = messages
                .iter()
                .filter(|m| matches!(m.role, Role::System))
                .map(|m| m.text())
                .reduce(|a, b| format!("{a}\n{b}"));

            let contents: Vec<_> = messages
                .iter()
                .filter(|m| !matches!(m.role, Role::System))
                .map(|m| {
                    let role = match m.role {
                        Role::User => "user",
                        Role::Assistant => "model",
                        Role::System => unreachable!(),
                    };
                    serde_json::json!({
                        "role": role,
                        "parts": [{"text": m.text()}],
                    })
                })
                .collect();

            let mut body = serde_json::json!({
                "contents": contents,
            });

            if let Some(sys) = system_text {
                body["systemInstruction"] = serde_json::json!({
                    "parts": [{"text": sys}]
                });
            }

            if !tools.is_empty() {
                let function_declarations: Vec<_> = tools
                    .iter()
                    .map(|t| {
                        serde_json::json!({
                            "name": t.name,
                            "description": t.description,
                            "parameters": t.input_schema,
                        })
                    })
                    .collect();
                body["tools"] = serde_json::json!([{
                    "functionDeclarations": function_declarations,
                }]);
            }

            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
                model, self.api_key
            );

            let response = reqwest::Client::new()
                .post(&url)
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await
                .context("failed to send request to gemini")?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("gemini api error {status}: {text}");
            }

            let byte_stream = response.bytes_stream();
            let mut parser = SseParser::new();

            let event_stream = byte_stream.flat_map(move |chunk| {
                let events = match chunk {
                    Ok(bytes) => parser
                        .feed(&bytes)
                        .into_iter()
                        .flat_map(|sse| parse_gemini_sse(&sse))
                        .collect::<Vec<_>>(),
                    Err(e) => vec![StreamEvent::Error(e.to_string())],
                };
                futures::stream::iter(events)
            });

            Ok(Box::pin(event_stream) as Pin<Box<dyn Stream<Item = StreamEvent> + Send>>)
        })
    }
}

fn parse_gemini_sse(sse: &super::SseEvent) -> Vec<StreamEvent> {
    let json = match sse.json() {
        Some(j) => j,
        None => return vec![],
    };

    let mut events = Vec::new();

    if let Some(candidates) = json["candidates"].as_array() {
        if let Some(candidate) = candidates.first() {
            if let Some(parts) = candidate["content"]["parts"].as_array() {
                for part in parts {
                    if let Some(text) = part["text"].as_str() {
                        events.push(StreamEvent::Delta(text.to_string()));
                    }
                    // function call
                    if let Some(fc) = part.get("functionCall") {
                        if let Some(name) = fc["name"].as_str() {
                            events.push(StreamEvent::ToolUseStart {
                                id: name.to_string(),
                                name: name.to_string(),
                            });
                            if let Some(args) = fc.get("args") {
                                events.push(StreamEvent::ToolUseInput(args.to_string()));
                            }
                            events.push(StreamEvent::ToolUseEnd);
                        }
                    }
                }
            }
            if candidate["finishReason"].as_str().is_some() {
                events.push(StreamEvent::Done);
            }
        }
    }

    events
}
