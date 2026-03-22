use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use orc_core::event::Event;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};

use crate::component::{Chat, Input, StatusBar};

pub struct MainScreen {
    chat: Chat,
    input: Input,
}

impl MainScreen {
    pub fn new() -> Self {
        Self {
            chat: Chat::new(),
            input: Input::new(),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Vec<Event> {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::PageUp) => {
                self.chat.scroll_up();
                vec![]
            }
            (KeyModifiers::NONE, KeyCode::PageDown) => {
                self.chat.scroll_down();
                vec![]
            }
            _ => {
                if let Some(event) = self.input.handle_key(key) {
                    vec![event]
                } else {
                    vec![]
                }
            }
        }
    }

    pub fn on_stream_delta(&mut self) {
        self.chat.scroll_to_bottom();
    }

    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        provider_name: &str,
        model: &str,
        messages: &[orc_core::provider::Message],
        streaming_text: &str,
    ) {
        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .split(area);

        StatusBar::render(frame, layout[0], provider_name, model);
        self.chat
            .render(frame, layout[1], messages, streaming_text);
        self.input.render(frame, layout[2]);
    }
}
