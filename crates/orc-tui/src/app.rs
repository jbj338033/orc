use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode};
use futures::StreamExt;
use orc_core::config::{AppConfig, load_config};
use orc_core::event::{Event, ModalKind, Screen};
use orc_core::provider::{Message, Provider, ProviderRegistry, StreamEvent};
use orc_core::session::Session;
use orc_core::tool::ToolRegistry;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::Clear;
use tokio::sync::mpsc;

use crate::keymap::handle_global_key;
use crate::screen::main_screen::MainScreen;
use crate::screen::settings::SettingsScreen;
use crate::screen::setup::SetupScreen;
use crate::terminal::Tui;
use crate::widget::ListSelect;

pub struct App {
    config: AppConfig,
    provider_registry: ProviderRegistry,
    tool_registry: ToolRegistry,
    active_provider: Option<Box<dyn Provider>>,
    current_model: String,
    session: Option<Session>,
    screen: Screen,
    setup_screen: Option<SetupScreen>,
    main_screen: MainScreen,
    settings_screen: SettingsScreen,
    modal: Option<ActiveModal>,
    should_quit: bool,
    is_streaming: bool,
    event_tx: mpsc::UnboundedSender<Event>,
    event_rx: mpsc::UnboundedReceiver<Event>,
}

enum ActiveModal {
    ProviderSelect(ListSelect),
    ModelSelect(ListSelect),
}

impl App {
    pub fn new() -> Result<Self> {
        let config = load_config()?;
        let provider_registry = ProviderRegistry::new();
        let tool_registry = ToolRegistry::new();
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let screen = if config.provider.is_empty() {
            Screen::Setup
        } else {
            Screen::Main
        };

        let setup_screen = if screen == Screen::Setup {
            Some(SetupScreen::new(&provider_registry))
        } else {
            None
        };

        let mut app = Self {
            config,
            provider_registry,
            tool_registry,
            active_provider: None,
            current_model: String::new(),
            session: None,
            screen,
            setup_screen,
            main_screen: MainScreen::new(),
            settings_screen: SettingsScreen::new(),
            modal: None,
            should_quit: false,
            is_streaming: false,
            event_tx,
            event_rx,
        };

        app.init_provider();
        Ok(app)
    }

    fn init_provider(&mut self) {
        if let Some(provider_id) = &self.config.default_provider {
            if let Some(entry) = self
                .config
                .provider
                .iter()
                .find(|p| &p.id == provider_id)
            {
                if let Ok(provider) = self.provider_registry.create_provider(entry) {
                    let models = provider.models();
                    self.current_model = self
                        .config
                        .default_model
                        .clone()
                        .or_else(|| models.first().map(|m| m.id.clone()))
                        .unwrap_or_default();
                    self.session = Some(Session::new(
                        provider_id.clone(),
                        self.current_model.clone(),
                    ));
                    self.active_provider = Some(provider);
                }
            }
        }
    }

    pub async fn run(&mut self, terminal: &mut Tui) -> Result<()> {
        let (key_tx, mut key_rx) = mpsc::unbounded_channel::<CrosstermEvent>();

        // 키 이벤트를 별도 스레드에서 읽기
        std::thread::spawn(move || loop {
            if event::poll(std::time::Duration::from_millis(50)).unwrap_or(false) {
                if let Ok(evt) = event::read() {
                    if key_tx.send(evt).is_err() {
                        break;
                    }
                }
            }
        });

        loop {
            terminal.draw(|f| self.render(f))?;

            tokio::select! {
                Some(crossterm_event) = key_rx.recv() => {
                    if let CrosstermEvent::Key(key) = crossterm_event {
                        self.handle_key(key);
                    }
                }
                Some(event) = self.event_rx.recv() => {
                    self.handle_event(event).await;
                }
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        // 모달이 열려있으면 모달에서 처리
        if let Some(modal) = &mut self.modal {
            match key.code {
                KeyCode::Esc => {
                    self.modal = None;
                    return;
                }
                _ => {
                    match modal {
                        ActiveModal::ProviderSelect(list) => {
                            if let Some(idx) = list.handle_key(key) {
                                let providers = &self.config.provider;
                                if let Some(entry) = providers.get(idx) {
                                    self.config.default_provider = Some(entry.id.clone());
                                    let _ = orc_core::config::save_config(&self.config);
                                    self.init_provider();
                                }
                                self.modal = None;
                            }
                        }
                        ActiveModal::ModelSelect(list) => {
                            if let Some(idx) = list.handle_key(key) {
                                if let Some(provider) = &self.active_provider {
                                    let models = provider.models();
                                    if let Some(model) = models.get(idx) {
                                        self.current_model = model.id.clone();
                                        self.config.default_model =
                                            Some(self.current_model.clone());
                                        let _ = orc_core::config::save_config(&self.config);
                                        if let Some(session) = &mut self.session {
                                            session.model = self.current_model.clone();
                                        }
                                    }
                                }
                                self.modal = None;
                            }
                        }
                    }
                    return;
                }
            }
        }

        // 글로벌 키 처리
        if let Some(event) = handle_global_key(key) {
            let tx = self.event_tx.clone();
            let _ = tx.send(event);
            return;
        }

        // 현재 화면에 키 전달
        let events = match self.screen {
            Screen::Setup => {
                if let Some(ref mut setup) = self.setup_screen {
                    setup.handle_key(key, &self.provider_registry, &mut self.config)
                } else {
                    vec![]
                }
            }
            Screen::Main => self.main_screen.handle_key(key),
            Screen::Settings => self.settings_screen.handle_key(key, &mut self.config),
        };

        for event in events {
            let _ = self.event_tx.send(event);
        }
    }

    async fn handle_event(&mut self, event: Event) {
        match event {
            Event::Quit => self.should_quit = true,
            Event::Navigate(screen) => {
                if screen == Screen::Settings {
                    self.settings_screen.refresh(&self.config);
                }
                self.screen = screen;
            }
            Event::ShowModal(kind) => match kind {
                ModalKind::ProviderSelect => {
                    let items: Vec<String> = self
                        .config
                        .provider
                        .iter()
                        .map(|p| format!("{} ({})", p.id, p.provider_type))
                        .collect();
                    self.modal = Some(ActiveModal::ProviderSelect(ListSelect::new(items)));
                }
                ModalKind::ModelSelect => {
                    if let Some(provider) = &self.active_provider {
                        let items: Vec<String> = provider
                            .models()
                            .iter()
                            .map(|m| {
                                let ctx = m
                                    .context_window
                                    .map(|c| format!("  {}k ctx", c / 1000))
                                    .unwrap_or_default();
                                format!("{}{}", m.display_name, ctx)
                            })
                            .collect();
                        self.modal = Some(ActiveModal::ModelSelect(ListSelect::new(items)));
                    }
                }
                _ => {}
            },
            Event::CloseModal => self.modal = None,
            Event::ProviderChanged => {
                self.init_provider();
            }
            Event::ModelChanged(model) => {
                self.current_model = model;
            }
            Event::SendMessage(text) => {
                self.send_message(text).await;
            }
            Event::Stream(stream_event) => {
                self.handle_stream_event(stream_event).await;
            }
            Event::ToolStart { .. } => {
                // tool 실행 시작 표시는 향후 구현
            }
            Event::ToolDone { id, result } => {
                self.handle_tool_result(&id, result).await;
            }
            _ => {}
        }
    }

    async fn send_message(&mut self, text: String) {
        let session = match &mut self.session {
            Some(s) => s,
            None => return,
        };

        session.push(Message::user(&text));
        self.is_streaming = true;

        let provider = match &self.active_provider {
            Some(p) => p,
            None => return,
        };

        let tool_defs = self.tool_registry.definitions();
        let model = self.current_model.clone();
        let messages = session.messages.clone();
        let tx = self.event_tx.clone();

        match provider.stream(&model, &messages, &tool_defs).await {
            Ok(mut stream) => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    while let Some(event) = stream.next().await {
                        if tx.send(Event::Stream(event)).is_err() {
                            break;
                        }
                    }
                });
            }
            Err(e) => {
                let _ = tx.send(Event::Stream(StreamEvent::Error(e.to_string())));
            }
        }
    }

    async fn handle_stream_event(&mut self, event: StreamEvent) {
        let session = match &mut self.session {
            Some(s) => s,
            None => return,
        };

        match event {
            StreamEvent::Delta(text) => {
                session.append_delta(&text);
                self.main_screen.on_stream_delta();
            }
            StreamEvent::ToolUseStart { id, name } => {
                // 현재 스트리밍 텍스트 확정
                let text = session.finish_streaming();
                if !text.is_empty() {
                    session.push(Message::assistant(&text));
                }
                let _ = self.event_tx.send(Event::ToolStart {
                    id: id.clone(),
                    name: name.clone(),
                });
            }
            StreamEvent::ToolUseInput(chunk) => {
                session.append_tool_input(&chunk);
            }
            StreamEvent::ToolUseEnd => {
                let _input = session.take_tool_input();
                // tool 실행은 향후 여기서 처리
            }
            StreamEvent::Done => {
                let text = session.finish_streaming();
                if !text.is_empty() {
                    session.push(Message::assistant(&text));
                }
                self.is_streaming = false;
            }
            StreamEvent::Error(err) => {
                session.push(Message::assistant(format!("Error: {err}")));
                self.is_streaming = false;
            }
        }
    }

    async fn handle_tool_result(&mut self, _id: &str, _result: orc_core::tool::ToolResult) {
        // tool result를 메시지에 추가하고 재요청하는 로직은 향후 구현
    }

    fn render(&mut self, frame: &mut ratatui::Frame) {
        let area = frame.area();

        match self.screen {
            Screen::Setup => {
                if let Some(ref mut setup) = self.setup_screen {
                    setup.render(frame, area);
                }
            }
            Screen::Main => {
                let provider_name = self
                    .active_provider
                    .as_ref()
                    .map(|p| p.name())
                    .unwrap_or("none");
                let streaming = self
                    .session
                    .as_ref()
                    .map(|s| s.streaming_text())
                    .unwrap_or("");
                let messages = self
                    .session
                    .as_ref()
                    .map(|s| s.messages.as_slice())
                    .unwrap_or(&[]);

                self.main_screen.render(
                    frame,
                    area,
                    provider_name,
                    &self.current_model,
                    messages,
                    streaming,
                );
            }
            Screen::Settings => {
                self.settings_screen.render(frame, area);
            }
        }

        // 모달 렌더
        if let Some(ref mut modal) = self.modal {
            let modal_area = centered_rect(40, 40, area);
            frame.render_widget(Clear, modal_area);
            match modal {
                ActiveModal::ProviderSelect(list) => {
                    list.render(frame, modal_area, "Provider");
                }
                ActiveModal::ModelSelect(list) => {
                    list.render(frame, modal_area, "Model");
                }
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
