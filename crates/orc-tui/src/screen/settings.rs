use crossterm::event::{KeyCode, KeyEvent};
use orc_core::config::AppConfig;
use orc_core::event::{Event, Screen};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::theme::Theme;
use crate::widget::ListSelect;

pub struct SettingsScreen {
    list: Option<ListSelect>,
}

impl SettingsScreen {
    pub fn new() -> Self {
        Self { list: None }
    }

    pub fn refresh(&mut self, config: &AppConfig) {
        let items: Vec<String> = config
            .provider
            .iter()
            .map(|p| {
                let default_marker = if config.default_provider.as_deref() == Some(&p.id) {
                    " (default)"
                } else {
                    ""
                };
                let auth_desc = p
                    .auth
                    .method
                    .as_deref()
                    .or(p.auth.api_key_env.as_deref())
                    .unwrap_or("none");
                format!(
                    "{:<20} {:<12} {}{}",
                    p.id, p.provider_type, auth_desc, default_marker
                )
            })
            .collect();

        self.list = Some(ListSelect::new(items));
    }

    pub fn handle_key(&mut self, key: KeyEvent, config: &mut AppConfig) -> Vec<Event> {
        match key.code {
            KeyCode::Esc => return vec![Event::Navigate(Screen::Main)],
            KeyCode::Char('d') | KeyCode::Delete => {
                if let Some(ref list) = self.list {
                    if let Some(idx) = list.selected_index() {
                        if idx < config.provider.len() {
                            let removed = config.provider.remove(idx);
                            if config.default_provider.as_deref() == Some(&removed.id) {
                                config.default_provider =
                                    config.provider.first().map(|p| p.id.clone());
                            }
                            let _ = orc_core::config::save_config(config);
                            self.refresh(config);
                            return vec![Event::ProviderChanged];
                        }
                    }
                }
            }
            KeyCode::Enter => {
                if let Some(ref list) = self.list {
                    if let Some(idx) = list.selected_index() {
                        if idx < config.provider.len() {
                            config.default_provider = Some(config.provider[idx].id.clone());
                            let _ = orc_core::config::save_config(config);
                            self.refresh(config);
                            return vec![Event::ProviderChanged];
                        }
                    }
                }
            }
            _ => {
                if let Some(ref mut list) = self.list {
                    list.handle_key(key);
                }
            }
        }
        vec![]
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::vertical([
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .split(area);

        let title = Paragraph::new(Line::from(Span::styled(
            " Settings > Providers",
            Theme::accent(),
        )))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Theme::border()),
        );

        frame.render_widget(title, layout[0]);

        if let Some(ref mut list) = self.list {
            list.render(frame, layout[1], "Providers");
        }

        let help = Paragraph::new(Line::from(vec![
            Span::styled("[Enter] ", Theme::accent()),
            Span::styled("Set default  ", Theme::dimmed()),
            Span::styled("[D] ", Theme::accent()),
            Span::styled("Delete  ", Theme::dimmed()),
            Span::styled("[Esc] ", Theme::accent()),
            Span::styled("Back", Theme::dimmed()),
        ]));

        frame.render_widget(help, layout[2]);
    }
}
