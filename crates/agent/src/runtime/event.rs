#[derive(Debug, Clone)]
pub enum AgentEvent {
    TextDelta(String),
    ToolCall {
        id: String,
        name: String,
        input: String,
    },
    ToolResult {
        id: String,
        output: String,
        is_error: bool,
    },
    Done,
    Error(String),
}
