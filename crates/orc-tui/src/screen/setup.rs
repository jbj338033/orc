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
    auth_list: Option<ListSelect>,
    name_input: TextInput,
    key_input: TextInput,
    url_input: TextInput,
    selected_factory_id: String,
    instance_name: String,
    selected_auth: AuthKind,
    oauth_pending: bool,
}

#[derive(Clone)]
enum AuthKind {
    ApiKey,
    OAuth,
    None,
}

enum SetupStep {
    SelectProvider,
    EnterName,
    SelectAuth,
    EnterApiKey,
    EnterBaseUrl,
    OAuthWaiting,
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
            auth_list: None,
            name_input: TextInput::new(),
            key_input: TextInput::new(),
            url_input: TextInput::new(),
            selected_factory_id: String::new(),
            instance_name: String::new(),
            selected_auth: AuthKind::ApiKey,
            oauth_pending: false,
        }
    }

    fn auth_options_for(&self, factory_id: &str) -> Vec<(String, AuthKind)> {
        match factory_id {
            "anthropic" => vec![
                ("API Key".to_string(), AuthKind::ApiKey),
                ("OAuth (browser login)".to_string(), AuthKind::OAuth),
            ],
            "openai" | "gemini" => vec![("API Key".to_string(), AuthKind::ApiKey)],
            "ollama" => vec![("No auth needed".to_string(), AuthKind::None)],
            _ => vec![("API Key".to_string(), AuthKind::ApiKey)],
        }
    }

    fn finish_setup(&mut self, config: &mut AppConfig, entry: ProviderEntry) -> Vec<Event> {
        if config.default_provider.is_none() {
            config.default_provider = Some(entry.id.clone());
        }
        config.provider.push(entry);
        let _ = orc_core::config::save_config(config);
        vec![Event::Navigate(Screen::Main), Event::ProviderChanged]
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
                        self.step = SetupStep::EnterName;
                    }
                }
                vec![]
            }
            SetupStep::EnterName => {
                if key.code == KeyCode::Esc {
                    self.step = SetupStep::SelectProvider;
                    return vec![];
                }
                if let Some(name) = self.name_input.handle_key(key) {
                    self.instance_name = name;
                    let options = self.auth_options_for(&self.selected_factory_id);
                    if options.len() == 1 {
                        self.selected_auth = options[0].1.clone();
                        match &self.selected_auth {
                            AuthKind::ApiKey => self.step = SetupStep::EnterApiKey,
                            AuthKind::OAuth => self.step = SetupStep::OAuthWaiting,
                            AuthKind::None => {
                                let needs_url = self.selected_factory_id == "ollama";
                                if needs_url {
                                    self.step = SetupStep::EnterBaseUrl;
                                } else {
                                    let entry = ProviderEntry {
                                        id: self.instance_name.clone(),
                                        provider_type: self.selected_factory_id.clone(),
                                        base_url: None,
                                        auth: AuthConfig::default(),
                                    };
                                    return self.finish_setup(config, entry);
                                }
                            }
                        }
                    } else {
                        let items = options.iter().map(|(label, _)| label.clone()).collect();
                        self.auth_list = Some(ListSelect::new(items));
                        self.step = SetupStep::SelectAuth;
                    }
                }
                vec![]
            }
            SetupStep::SelectAuth => {
                if key.code == KeyCode::Esc {
                    self.step = SetupStep::EnterName;
                    return vec![];
                }
                if let Some(ref mut list) = self.auth_list {
                    if let Some(idx) = list.handle_key(key) {
                        let options = self.auth_options_for(&self.selected_factory_id);
                        if let Some((_, auth_kind)) = options.get(idx) {
                            self.selected_auth = auth_kind.clone();
                            match auth_kind {
                                AuthKind::ApiKey => self.step = SetupStep::EnterApiKey,
                                AuthKind::OAuth => self.step = SetupStep::OAuthWaiting,
                                AuthKind::None => {
                                    let entry = ProviderEntry {
                                        id: self.instance_name.clone(),
                                        provider_type: self.selected_factory_id.clone(),
                                        base_url: None,
                                        auth: AuthConfig::default(),
                                    };
                                    return self.finish_setup(config, entry);
                                }
                            }
                        }
                    }
                }
                vec![]
            }
            SetupStep::EnterApiKey => {
                if key.code == KeyCode::Esc {
                    self.step = SetupStep::SelectAuth;
                    return vec![];
                }
                if let Some(api_key) = self.key_input.handle_key(key) {
                    let needs_url = self.selected_factory_id == "ollama";
                    if needs_url {
                        // ollama는 api key 후 base_url 필요 (보통 안 오지만)
                        self.step = SetupStep::EnterBaseUrl;
                        return vec![];
                    }
                    let entry = ProviderEntry {
                        id: self.instance_name.clone(),
                        provider_type: self.selected_factory_id.clone(),
                        base_url: None,
                        auth: AuthConfig {
                            method: Some("api_key".to_string()),
                            api_key: Some(api_key),
                            api_key_env: None,
                        },
                    };
                    return self.finish_setup(config, entry);
                }
                vec![]
            }
            SetupStep::EnterBaseUrl => {
                if key.code == KeyCode::Esc {
                    self.step = SetupStep::EnterName;
                    return vec![];
                }
                if let Some(url) = self.url_input.handle_key(key) {
                    let base_url = if url.is_empty() {
                        "http://localhost:11434".to_string()
                    } else {
                        url
                    };
                    let entry = ProviderEntry {
                        id: self.instance_name.clone(),
                        provider_type: self.selected_factory_id.clone(),
                        base_url: Some(base_url),
                        auth: AuthConfig::default(),
                    };
                    return self.finish_setup(config, entry);
                }
                vec![]
            }
            SetupStep::OAuthWaiting => {
                if key.code == KeyCode::Esc {
                    self.step = SetupStep::SelectAuth;
                    self.oauth_pending = false;
                    return vec![];
                }
                if key.code == KeyCode::Enter && !self.oauth_pending {
                    self.oauth_pending = true;
                    // provider를 먼저 config에 저장 (oauth method로)
                    let entry = ProviderEntry {
                        id: self.instance_name.clone(),
                        provider_type: self.selected_factory_id.clone(),
                        base_url: None,
                        auth: AuthConfig {
                            method: Some("oauth".to_string()),
                            api_key: None,
                            api_key_env: None,
                        },
                    };
                    return self.finish_setup(config, entry);
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
                let layout =
                    Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(center);

                let title = Paragraph::new(Line::from(vec![
                    Span::styled("Welcome to ", Theme::base()),
                    Span::styled("orc", Theme::accent()),
                ]))
                .block(
                    Block::default()
                        .borders(Borders::BOTTOM)
                        .border_style(Theme::border()),
                );

                frame.render_widget(title, layout[0]);
                self.provider_list
                    .render(frame, layout[1], "Select a provider");
            }
            SetupStep::EnterName => {
                let layout = Layout::vertical([
                    Constraint::Length(2),
                    Constraint::Length(3),
                    Constraint::Fill(1),
                ])
                .split(center);

                let label = Paragraph::new(Line::from(vec![
                    Span::styled("Name this ", Theme::base()),
                    Span::styled(&self.selected_factory_id, Theme::accent()),
                    Span::styled(" instance:", Theme::base()),
                ]));

                frame.render_widget(label, layout[0]);
                self.name_input.render(frame, layout[1]);
            }
            SetupStep::SelectAuth => {
                let layout =
                    Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(center);

                let label = Paragraph::new(Line::from(vec![
                    Span::styled(&self.instance_name, Theme::accent()),
                    Span::styled(" — choose auth method:", Theme::base()),
                ]));

                frame.render_widget(label, layout[0]);
                if let Some(ref mut list) = self.auth_list {
                    list.render(frame, layout[1], "Auth Method");
                }
            }
            SetupStep::EnterApiKey => {
                let layout = Layout::vertical([
                    Constraint::Length(2),
                    Constraint::Length(3),
                    Constraint::Fill(1),
                ])
                .split(center);

                let label = Paragraph::new(Line::from(vec![
                    Span::styled(&self.instance_name, Theme::accent()),
                    Span::styled(" — enter API key:", Theme::base()),
                ]));

                frame.render_widget(label, layout[0]);
                self.key_input.render(frame, layout[1]);
            }
            SetupStep::EnterBaseUrl => {
                let layout = Layout::vertical([
                    Constraint::Length(2),
                    Constraint::Length(3),
                    Constraint::Length(2),
                    Constraint::Fill(1),
                ])
                .split(center);

                let label = Paragraph::new(Line::from(vec![
                    Span::styled(&self.instance_name, Theme::accent()),
                    Span::styled(" — enter base URL:", Theme::base()),
                ]));

                let hint = Paragraph::new(Span::styled(
                    "  (empty for http://localhost:11434)",
                    Theme::dimmed(),
                ));

                frame.render_widget(label, layout[0]);
                self.url_input.render(frame, layout[1]);
                frame.render_widget(hint, layout[2]);
            }
            SetupStep::OAuthWaiting => {
                let layout = Layout::vertical([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(2),
                    Constraint::Fill(1),
                ])
                .split(center);

                let label = Paragraph::new(Line::from(vec![
                    Span::styled(&self.instance_name, Theme::accent()),
                    Span::styled(" — OAuth", Theme::base()),
                ]));

                let (msg, style) = if self.oauth_pending {
                    ("  Saving... Browser will open on first message.", Theme::dimmed())
                } else {
                    ("  Press Enter to save. Browser opens on first use.", Theme::base())
                };

                let action = Paragraph::new(Span::styled(msg, style));

                let hint = Paragraph::new(Span::styled("  [Esc] Cancel", Theme::dimmed()));

                frame.render_widget(label, layout[0]);
                frame.render_widget(action, layout[1]);
                frame.render_widget(hint, layout[2]);
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
