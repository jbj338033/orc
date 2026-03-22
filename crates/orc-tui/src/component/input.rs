use crossterm::event::KeyEvent;
use orc_core::event::Event;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::widget::TextInput;

pub struct Input {
    text_input: TextInput,
}

impl Input {
    pub fn new() -> Self {
        Self {
            text_input: TextInput::new(),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Event> {
        if let Some(text) = self.text_input.handle_key(key) {
            Some(Event::SendMessage(text))
        } else {
            None
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        self.text_input.render(frame, area);
    }
}
