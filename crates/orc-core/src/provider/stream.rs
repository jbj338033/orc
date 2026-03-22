use serde_json::Value;

#[derive(Debug, Clone)]
pub enum StreamEvent {
    Delta(String),
    ToolUseStart { id: String, name: String },
    ToolUseInput(String),
    ToolUseEnd,
    Done,
    Error(String),
}

pub struct SseParser {
    buffer: String,
}

impl SseParser {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    pub fn feed(&mut self, chunk: &[u8]) -> Vec<SseEvent> {
        let text = String::from_utf8_lossy(chunk);
        self.buffer.push_str(&text);

        let mut events = Vec::new();
        while let Some(pos) = self.buffer.find("\n\n") {
            let block = self.buffer[..pos].to_string();
            self.buffer = self.buffer[pos + 2..].to_string();

            let mut event_type = None;
            let mut data_lines = Vec::new();

            for line in block.lines() {
                if let Some(val) = line.strip_prefix("event:") {
                    event_type = Some(val.trim().to_string());
                } else if let Some(val) = line.strip_prefix("data:") {
                    data_lines.push(val.trim().to_string());
                }
            }

            if !data_lines.is_empty() {
                let data = data_lines.join("\n");
                events.push(SseEvent { event_type, data });
            }
        }

        events
    }
}

#[derive(Debug)]
pub struct SseEvent {
    pub event_type: Option<String>,
    pub data: String,
}

impl SseEvent {
    pub fn json(&self) -> Option<Value> {
        serde_json::from_str(&self.data).ok()
    }
}
