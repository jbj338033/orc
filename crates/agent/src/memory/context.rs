use crate::runtime::Message;

pub struct ContextBudget {
    pub total_tokens: u32,
    pub used_tokens: u32,
    pub compaction_threshold: f32,
}

impl ContextBudget {
    pub fn new(total_tokens: u32) -> Self {
        Self {
            total_tokens,
            used_tokens: 0,
            compaction_threshold: 0.835,
        }
    }

    pub fn should_compact(&self) -> bool {
        let ratio = self.used_tokens as f32 / self.total_tokens as f32;
        ratio >= self.compaction_threshold
    }

    pub fn remaining(&self) -> u32 {
        self.total_tokens.saturating_sub(self.used_tokens)
    }
}

pub fn compact_messages(messages: &[Message], seed_goal: &str) -> Vec<Message> {
    // seed_goal is never compacted — it stays in system prompt
    // strategy: keep first message, last N messages, summarize middle
    let keep_recent = 10;

    if messages.len() <= keep_recent + 2 {
        return messages.to_vec();
    }

    let mut result = Vec::new();

    // keep first message (initial request)
    result.push(messages[0].clone());

    // summarize middle section
    let middle = &messages[1..messages.len() - keep_recent];
    let summary_text = format!(
        "[compacted {} messages. seed goal: {}]",
        middle.len(),
        seed_goal
    );
    result.push(Message::user(summary_text));

    // keep recent messages
    result.extend_from_slice(&messages[messages.len() - keep_recent..]);

    result
}
