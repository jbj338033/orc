use std::collections::HashMap;
use std::sync::Arc;

use super::traits::CompletionProvider;
use super::types::ProviderError;

pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn CompletionProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register(&mut self, provider: Arc<dyn CompletionProvider>) {
        let name = provider.provider_name().to_string();
        self.providers.insert(name, provider);
    }

    pub fn get(&self, name: &str) -> Result<&Arc<dyn CompletionProvider>, ProviderError> {
        self.providers.get(name).ok_or_else(|| ProviderError {
            code: super::types::ErrorCode::InvalidRequest,
            message: format!("provider not found: {name}"),
            retriable: false,
        })
    }

    pub fn list(&self) -> Vec<&str> {
        self.providers.keys().map(|k| k.as_str()).collect()
    }
}
