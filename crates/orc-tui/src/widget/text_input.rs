use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::theme::Theme;

pub struct TextInput {
    content: String,
    cursor: usize,
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<String> {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(c)) => {
                self.content.insert(self.cursor, c);
                self.cursor += c.len_utf8();
            }
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                if self.cursor > 0 {
                    let prev = self.content[..self.cursor]
                        .chars()
                        .last()
                        .map(|c| c.len_utf8())
                        .unwrap_or(0);
                    self.cursor -= prev;
                    self.content.remove(self.cursor);
                }
            }
            (KeyModifiers::NONE, KeyCode::Delete) => {
                if self.cursor < self.content.len() {
                    self.content.remove(self.cursor);
                }
            }
            (KeyModifiers::NONE, KeyCode::Left) => {
                if self.cursor > 0 {
                    let prev = self.content[..self.cursor]
                        .chars()
                        .last()
                        .map(|c| c.len_utf8())
                        .unwrap_or(0);
                    self.cursor -= prev;
                }
            }
            (KeyModifiers::NONE, KeyCode::Right) => {
                if self.cursor < self.content.len() {
                    let next = self.content[self.cursor..]
                        .chars()
                        .next()
                        .map(|c| c.len_utf8())
                        .unwrap_or(0);
                    self.cursor += next;
                }
            }
            (KeyModifiers::CONTROL, KeyCode::Char('a')) => {
                self.cursor = 0;
            }
            (KeyModifiers::CONTROL, KeyCode::Char('e')) => {
                self.cursor = self.content.len();
            }
            (KeyModifiers::CONTROL, KeyCode::Char('u')) => {
                self.content.drain(..self.cursor);
                self.cursor = 0;
            }
            (KeyModifiers::NONE, KeyCode::Enter) => {
                if !self.content.trim().is_empty() {
                    let text = std::mem::take(&mut self.content);
                    self.cursor = 0;
                    return Some(text);
                }
            }
            _ => {}
        }
        None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Theme::border())
            .title(" > ")
            .title_style(Theme::accent());

        let inner = block.inner(area);

        let display = if self.content.is_empty() {
            Paragraph::new("Type a message...")
                .style(Theme::dimmed())
                .block(block)
        } else {
            Paragraph::new(self.content.as_str())
                .style(Theme::input())
                .block(block)
        };

        frame.render_widget(display, area);

        // 커서 위치
        let cursor_x = inner.x + self.content[..self.cursor].chars().count() as u16;
        let cursor_y = inner.y;
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}
