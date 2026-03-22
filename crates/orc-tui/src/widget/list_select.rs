use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

use crate::theme::Theme;

pub struct ListSelect {
    items: Vec<String>,
    state: ListState,
}

impl ListSelect {
    pub fn new(items: Vec<String>) -> Self {
        let mut state = ListState::default();
        if !items.is_empty() {
            state.select(Some(0));
        }
        Self { items, state }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<usize> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_down();
            }
            KeyCode::Enter => {
                return self.state.selected();
            }
            _ => {}
        }
        None
    }

    fn move_up(&mut self) {
        if let Some(i) = self.state.selected() {
            if i > 0 {
                self.state.select(Some(i - 1));
            }
        }
    }

    fn move_down(&mut self) {
        if let Some(i) = self.state.selected() {
            if i < self.items.len().saturating_sub(1) {
                self.state.select(Some(i + 1));
            }
        }
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, title: &str) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|s| ListItem::new(Line::raw(format!("  {s}"))))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Theme::border())
                    .title(format!(" {title} "))
                    .title_style(Theme::accent()),
            )
            .highlight_style(Theme::selected())
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, area, &mut self.state);
    }
}
