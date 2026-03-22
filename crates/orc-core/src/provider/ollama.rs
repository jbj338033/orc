use anyhow::Result;

use super::{ConfigField, FieldType, ModelInfo, Provider, ProviderFactory};
use super::openai::OpenAiProvider;
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
            inner: OpenAiProvider::with_models(
                entry.id.clone(),
                String::new(),
                base_url,
            ),
        }))
    }
}

struct OllamaProvider {
    inner: OpenAiProvider,
}

impl Provider for OllamaProvider {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn models(&self) -> Vec<ModelInfo> {
        // 동적 조회는 향후 구현. 일단 빈 목록 반환하고 사용자가 직접 모델명 입력
        vec![]
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
