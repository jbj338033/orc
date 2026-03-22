use crate::provider::Message;

#[derive(Debug)]
pub struct Session {
    pub provider_id: String,
    pub model: String,
    pub messages: Vec<Message>,
    streaming_text: String,
    tool_input_buffer: String,
}

impl Session {
    pub fn new(provider_id: String, model: String) -> Self {
        Self {
            provider_id,
            model,
            messages: Vec::new(),
            streaming_text: String::new(),
            tool_input_buffer: String::new(),
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
}
