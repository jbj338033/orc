use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHandle {
    pub provider: String,
    pub model: String,
}

impl ModelHandle {
    pub fn parse(s: &str) -> Option<Self> {
        let (provider, model) = s.split_once('/')?;
        Some(Self {
            provider: provider.into(),
            model: model.into(),
        })
    }

    pub fn as_string(&self) -> String {
        format!("{}/{}", self.provider, self.model)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<CompletionMessage>,
    pub system: Option<String>,
    pub tools: Vec<ToolDef>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub extensions: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionMessage {
    pub role: String,
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        id: String,
        output: String,
        is_error: bool,
    },
}

#[derive(Debug, Clone)]
pub enum StreamPart {
    TextDelta(String),
    ReasoningDelta(String),
    ToolCallDelta {
        id: String,
        name: String,
        args_delta: String,
    },
    ToolCallComplete {
        id: String,
        name: String,
        args: serde_json::Value,
    },
    Usage {
        input_tokens: u32,
        output_tokens: u32,
    },
    Finish(FinishReason),
    Error(ProviderError),
}

#[derive(Debug, Clone)]
pub enum FinishReason {
    Stop,
    MaxTokens,
    ToolUse,
}

#[derive(Debug, Clone)]
pub struct ProviderError {
    pub code: ErrorCode,
    pub message: String,
    pub retriable: bool,
}

#[derive(Debug, Clone)]
pub enum ErrorCode {
    RateLimit,
    Overloaded,
    AuthFailed,
    InvalidRequest,
    NetworkError,
    Unknown,
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for ProviderError {}
