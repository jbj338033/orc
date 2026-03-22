use std::pin::Pin;

use anyhow::Result;
use futures::Stream;

use super::{Message, StreamEvent};
use crate::config::ProviderEntry;
use crate::tool::ToolDefinition;

#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub context_window: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct ConfigField {
    pub key: &'static str,
    pub label: &'static str,
    pub field_type: FieldType,
    pub required: bool,
    pub default: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub enum FieldType {
    Text,
    Secret,
    Select(Vec<&'static str>),
    Toggle,
}

pub trait ProviderFactory: Send + Sync {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn config_fields(&self) -> Vec<ConfigField>;
    fn create(&self, entry: &ProviderEntry) -> Result<Box<dyn Provider>>;
}

pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    fn models(&self) -> Vec<ModelInfo>;
    fn stream(
        &self,
        model: &str,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Pin<Box<dyn Future<Output = Result<Pin<Box<dyn Stream<Item = StreamEvent> + Send>>>> + Send + '_>>;
}

use std::future::Future;
