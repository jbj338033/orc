mod anthropic;
mod gemini;
mod message;
mod ollama;
mod openai;
mod stream;
mod traits;

pub use message::*;
pub use stream::*;
pub use traits::*;

use std::collections::BTreeMap;

use anyhow::Result;

use crate::config::ProviderEntry;

pub struct ProviderRegistry {
    factories: BTreeMap<&'static str, Box<dyn ProviderFactory>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            factories: BTreeMap::new(),
        };
        registry.register(Box::new(anthropic::AnthropicFactory));
        registry.register(Box::new(openai::OpenAiFactory));
        registry.register(Box::new(gemini::GeminiFactory));
        registry.register(Box::new(ollama::OllamaFactory));
        registry
    }

    fn register(&mut self, factory: Box<dyn ProviderFactory>) {
        self.factories.insert(factory.id(), factory);
    }

    pub fn factory(&self, id: &str) -> Option<&dyn ProviderFactory> {
        self.factories.get(id).map(|f| f.as_ref())
    }

    pub fn factories(&self) -> Vec<&dyn ProviderFactory> {
        self.factories.values().map(|f| f.as_ref()).collect()
    }

    pub fn create_provider(&self, entry: &ProviderEntry) -> Result<Box<dyn Provider>> {
        let factory = self
            .factory(&entry.provider_type)
            .ok_or_else(|| anyhow::anyhow!("unknown provider type: {}", entry.provider_type))?;
        factory.create(entry)
    }
}
