use anyhow::Result;

use super::openai::OpenAiProvider;
use super::{ConfigField, FieldType, ModelInfo, Provider, ProviderFactory};
use crate::config::ProviderEntry;

const DEFAULT_BASE_URL: &str = "http://localhost:11434";

pub struct OllamaFactory;

impl ProviderFactory for OllamaFactory {
    fn id(&self) -> &'static str {
        "ollama"
    }

    fn display_name(&self) -> &'static str {
        "Ollama (Local)"
    }

    fn config_fields(&self) -> Vec<ConfigField> {
        vec![ConfigField {
            key: "base_url",
            label: "Base URL",
            field_type: FieldType::Text,
            required: false,
            default: Some(DEFAULT_BASE_URL),
        }]
    }

    fn create(&self, entry: &ProviderEntry) -> Result<Box<dyn Provider>> {
        let base_url = entry
            .base_url
            .clone()
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        Ok(Box::new(OllamaProvider {
            inner: OpenAiProvider::with_models(entry.id.clone(), String::new(), base_url.clone()),
            base_url,
        }))
    }
}

struct OllamaProvider {
    inner: OpenAiProvider,
    base_url: String,
}

impl Provider for OllamaProvider {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn models(&self) -> Vec<ModelInfo> {
        let url = format!("{}/api/tags", self.base_url);
        // tokio runtime 밖의 별도 OS 스레드에서 blocking HTTP 호출
        let handle = std::thread::spawn(move || {
            reqwest::blocking::Client::new()
                .get(&url)
                .timeout(std::time::Duration::from_secs(2))
                .send()
                .and_then(|r| r.json::<serde_json::Value>())
        });

        let result = handle.join().ok().and_then(|r| r.ok());

        match result {
            Some(json) => json["models"]
                .as_array()
                .map(|models| {
                    models
                        .iter()
                        .filter_map(|m| {
                            let name = m["name"].as_str()?;
                            Some(ModelInfo {
                                id: name.to_string(),
                                display_name: name.to_string(),
                                context_window: None,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default(),
            None => vec![],
        }
    }

    fn stream(
        &self,
        model: &str,
        messages: &[super::Message],
        tools: &[crate::tool::ToolDefinition],
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        std::pin::Pin<
                            Box<dyn futures::Stream<Item = super::StreamEvent> + Send>,
                        >,
                    >,
                > + Send
                + '_,
        >,
    > {
        self.inner.stream(model, messages, tools)
    }
}
