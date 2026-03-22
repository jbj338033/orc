use crate::provider::Message;

#[derive(Debug)]
pub struct PendingToolCall {
    pub id: String,
    pub name: String,
}

#[derive(Debug)]
pub struct Session {
    pub provider_id: String,
    pub model: String,
    pub messages: Vec<Message>,
    streaming_text: String,
    tool_input_buffer: String,
    pending_tool: Option<PendingToolCall>,
}

impl Session {
    pub fn new(provider_id: String, model: String) -> Self {
        Self {
            provider_id,
            model,
            messages: Vec::new(),
            streaming_text: String::new(),
            tool_input_buffer: String::new(),
            pending_tool: None,
        }
    }

    pub fn push(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn append_delta(&mut self, text: &str) {
        self.streaming_text.push_str(text);
    }

    pub fn streaming_text(&self) -> &str {
        &self.streaming_text
    }

    pub fn append_tool_input(&mut self, chunk: &str) {
        self.tool_input_buffer.push_str(chunk);
    }

    pub fn take_tool_input(&mut self) -> String {
        std::mem::take(&mut self.tool_input_buffer)
    }

    pub fn finish_streaming(&mut self) -> String {
        std::mem::take(&mut self.streaming_text)
    }

    pub fn set_pending_tool(&mut self, id: String, name: String) {
        self.pending_tool = Some(PendingToolCall { id, name });
    }

    pub fn take_pending_tool(&mut self) -> Option<PendingToolCall> {
        self.pending_tool.take()
    }
}
