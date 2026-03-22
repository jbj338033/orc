use crossterm::event::{KeyCode, KeyEvent};
use orc_core::config::{AppConfig, AuthConfig, ProviderEntry};
use orc_core::event::{Event, Screen};
use orc_core::provider::ProviderRegistry;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::theme::Theme;
use crate::widget::{ListSelect, TextInput};

pub struct SetupScreen {
    step: SetupStep,
    provider_list: ListSelect,
    name_input: TextInput,
    key_input: TextInput,
    selected_factory_id: String,
}

enum SetupStep {
    SelectProvider,
    ConfigureName,
    ConfigureKey,
}

impl SetupScreen {
    pub fn new(registry: &ProviderRegistry) -> Self {
        let items: Vec<String> = registry
            .factories()
            .iter()
            .map(|f| format!("{} ({})", f.display_name(), f.id()))
            .collect();

        Self {
            step: SetupStep::SelectProvider,
            provider_list: ListSelect::new(items),
            name_input: TextInput::new(),
            key_input: TextInput::new(),
            selected_factory_id: String::new(),
        }
    }

    pub fn handle_key(
        &mut self,
        key: KeyEvent,
        registry: &ProviderRegistry,
        config: &mut AppConfig,
    ) -> Vec<Event> {
        match self.step {
            SetupStep::SelectProvider => {
                if let Some(idx) = self.provider_list.handle_key(key) {
                    let factories = registry.factories();
                    if let Some(factory) = factories.get(idx) {
                        self.selected_factory_id = factory.id().to_string();
                        self.step = SetupStep::ConfigureName;
                    }
                }
                vec![]
            }
            SetupStep::ConfigureName => {
                if let Some(name) = self.name_input.handle_key(key) {
                    // ollama는 API key 불필요
                    if self.selected_factory_id == "ollama" {
                        let entry = ProviderEntry {
                            id: name,
                            provider_type: self.selected_factory_id.clone(),
                            base_url: Some("http://localhost:11434".to_string()),
                            auth: AuthConfig::default(),
                        };
                        config.provider.push(entry);
                        if config.default_provider.is_none() {
                            config.default_provider =
                                Some(config.provider.last().unwrap().id.clone());
                        }
                        let _ = orc_core::config::save_config(config);
                        return vec![Event::Navigate(Screen::Main), Event::ProviderChanged];
                    }
                    self.step = SetupStep::ConfigureKey;
                }
                if key.code == KeyCode::Esc {
                    self.step = SetupStep::SelectProvider;
                }
                vec![]
            }
            SetupStep::ConfigureKey => {
                if let Some(api_key) = self.key_input.handle_key(key) {
                    let entry = ProviderEntry {
                        id: config
                            .provider
                            .iter()
                            .find(|p| p.provider_type == self.selected_factory_id)
                            .map(|_| {
                                format!(
                                    "{}-{}",
                                    self.selected_factory_id,
                                    config.provider.len()
                                )
                            })
                            .unwrap_or_else(|| self.selected_factory_id.clone()),
                        provider_type: self.selected_factory_id.clone(),
                        base_url: None,
                        auth: AuthConfig {
                            method: Some("api_key".to_string()),
                            api_key: Some(api_key),
                            api_key_env: None,
                        },
                    };
                    if config.default_provider.is_none() {
                        config.default_provider = Some(entry.id.clone());
                    }
                    config.provider.push(entry);
                    let _ = orc_core::config::save_config(config);
                    return vec![Event::Navigate(Screen::Main), Event::ProviderChanged];
                }
                if key.code == KeyCode::Esc {
                    self.step = SetupStep::ConfigureName;
                }
                vec![]
            }
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let center = centered_rect(50, 60, area);
        frame.render_widget(Clear, center);

        match self.step {
            SetupStep::SelectProvider => {
                let layout = Layout::vertical([
                    Constraint::Length(3),
                    Constraint::Fill(1),
                ])
                .split(center);

                let title = Paragraph::new(Line::from(vec![
                    Span::styled("Welcome to ", Theme::base()),
                    Span::styled("orc", Theme::accent()),
                ]))
                .block(Block::default().borders(Borders::BOTTOM).border_style(Theme::border()));

                frame.render_widget(title, layout[0]);
                self.provider_list
                    .render(frame, layout[1], "Select a provider");
            }
            SetupStep::ConfigureName => {
                let layout = Layout::vertical([
                    Constraint::Length(2),
                    Constraint::Length(3),
                    Constraint::Fill(1),
                ])
                .split(center);

                let label = Paragraph::new(format!(
                    "Configure: {} — Enter a name for this instance",
                    self.selected_factory_id
                ))
                .style(Theme::base());

                frame.render_widget(label, layout[0]);
                self.name_input.render(frame, layout[1]);
            }
            SetupStep::ConfigureKey => {
                let layout = Layout::vertical([
                    Constraint::Length(2),
                    Constraint::Length(3),
                    Constraint::Fill(1),
                ])
                .split(center);

                let label = Paragraph::new("Enter your API key:")
                    .style(Theme::base());

                frame.render_widget(label, layout[0]);
                self.key_input.render(frame, layout[1]);
            }
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);
    let horizontal = Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1]);
    horizontal[1]
}
