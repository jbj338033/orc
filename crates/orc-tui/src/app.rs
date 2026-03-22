use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode};
use futures::StreamExt;
use orc_core::config::{AppConfig, load_config};
use orc_core::event::{Event, ModalKind, Screen};
use orc_core::provider::{
    ContentBlock, Message, Provider, ProviderRegistry, Role, StreamEvent, oauth,
};
use orc_core::session::Session;
use orc_core::tool::{ToolContext, ToolRegistry};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::Clear;
use tokio::sync::mpsc;

use crate::keymap::handle_global_key;
use crate::screen::main_screen::MainScreen;
use crate::screen::settings::SettingsScreen;
use crate::screen::setup::SetupScreen;
use crate::terminal::Tui;
use crate::widget::ListSelect;

const SYSTEM_PROMPT: &str = r#"You are an AI coding assistant running inside a terminal. You have access to tools for interacting with the user's filesystem and running commands.

When the user asks you to perform a task:
1. Use the available tools to read, write, and edit files
2. Use bash to run commands when needed
3. Use grep and glob to search the codebase
4. Explain what you're doing briefly

Be concise and direct. Focus on getting the task done."#;

pub struct App {
    config: AppConfig,
    provider_registry: ProviderRegistry,
    tool_registry: ToolRegistry,
    tool_context: ToolContext,
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
        let tool_context = ToolContext::default();
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
            tool_context,
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

                    match &mut self.session {
                        Some(session) => {
                            // 기존 대화 유지, provider/model만 전환
                            session.provider_id = provider_id.clone();
                            session.model = self.current_model.clone();
                        }
                        None => {
                            let mut session =
                                Session::new(provider_id.clone(), self.current_model.clone());
                            session.push(Message {
                                role: Role::System,
                                content: vec![ContentBlock::Text {
                                    text: SYSTEM_PROMPT.to_string(),
                                }],
                            });
                            self.session = Some(session);
                        }
                    }

                    self.active_provider = Some(provider);
                }
            }
        }
    }

    pub async fn run(&mut self, terminal: &mut Tui) -> Result<()> {
        let (key_tx, mut key_rx) = mpsc::unbounded_channel::<CrosstermEvent>();

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
        // 스트리밍 중에는 Ctrl+C로 중단만 허용
        if self.is_streaming {
            if key.code == KeyCode::Char('c')
                && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
            {
                self.is_streaming = false;
                if let Some(session) = &mut self.session {
                    let text = session.finish_streaming();
                    if !text.is_empty() {
                        session.push(Message::assistant(&text));
                    }
                }
            }
            return;
        }

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

        if let Some(event) = handle_global_key(key) {
            let _ = self.event_tx.send(event);
            return;
        }

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
            Event::ToolDone { id, result } => {
                self.handle_tool_done(&id, result).await;
            }
            Event::StartOAuth { provider_id } => {
                let tx = self.event_tx.clone();
                tokio::spawn(async move {
                    match oauth::run_oauth_flow(&provider_id).await {
                        Ok(_) => {
                            let _ = tx.send(Event::OAuthDone);
                        }
                        Err(e) => {
                            let _ = tx.send(Event::OAuthError(e.to_string()));
                        }
                    }
                });
            }
            Event::OAuthDone => {
                self.init_provider();
                self.screen = Screen::Main;
            }
            Event::OAuthError(_) => {
                // 실패하면 config에서 제거
                if self.setup_screen.is_some() {
                    // setup_screen의 instance_name에 접근 불가하므로 마지막 provider 제거
                    if let Some(last) = self.config.provider.last() {
                        if last.auth.method.as_deref() == Some("oauth") {
                            self.config.provider.pop();
                            if self.config.default_provider.as_deref()
                                == self.config.provider.last().map(|p| p.id.as_str())
                            {
                                self.config.default_provider = None;
                            }
                            let _ = orc_core::config::save_config(&self.config);
                        }
                    }
                }
                // setup으로 돌아가기
                self.setup_screen = Some(SetupScreen::new(&self.provider_registry));
                self.screen = Screen::Setup;
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
        self.start_streaming().await;
    }

    async fn start_streaming(&mut self) {
        self.is_streaming = true;

        let provider = match &self.active_provider {
            Some(p) => p,
            None => return,
        };

        let session = match &self.session {
            Some(s) => s,
            None => return,
        };

        let tool_defs = self.tool_registry.definitions();
        let model = self.current_model.clone();
        let messages = session.messages.clone();
        let tx = self.event_tx.clone();

        match provider.stream(&model, &messages, &tool_defs).await {
            Ok(mut stream) => {
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
                let text = session.finish_streaming();
                if !text.is_empty() {
                    session.push(Message::assistant(&text));
                }
                session.set_pending_tool(id, name);
            }
            StreamEvent::ToolUseInput(chunk) => {
                session.append_tool_input(&chunk);
            }
            StreamEvent::ToolUseEnd => {
                let input_json = session.take_tool_input();
                if let Some(pending) = session.take_pending_tool() {
                    let input: serde_json::Value =
                        serde_json::from_str(&input_json).unwrap_or(serde_json::Value::Null);

                    // assistant 메시지에 tool_use 블록 추가
                    session.push(Message {
                        role: Role::Assistant,
                        content: vec![ContentBlock::ToolUse {
                            id: pending.id.clone(),
                            name: pending.name.clone(),
                            input: input.clone(),
                        }],
                    });

                    // tool 실행
                    let tool_id = pending.id.clone();
                    let tool_name = pending.name.clone();
                    let tx = self.event_tx.clone();

                    if let Some(tool) = self.tool_registry.get(&tool_name) {
                        let result = tool.execute(input, &self.tool_context).await;
                        match result {
                            Ok(tool_result) => {
                                let _ = tx.send(Event::ToolDone {
                                    id: tool_id,
                                    result: tool_result,
                                });
                            }
                            Err(e) => {
                                let _ = tx.send(Event::ToolDone {
                                    id: tool_id,
                                    result: orc_core::tool::ToolResult::err(e.to_string()),
                                });
                            }
                        }
                    } else {
                        let _ = tx.send(Event::ToolDone {
                            id: tool_id,
                            result: orc_core::tool::ToolResult::err(format!(
                                "unknown tool: {tool_name}"
                            )),
                        });
                    }
                }
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

    async fn handle_tool_done(&mut self, tool_use_id: &str, result: orc_core::tool::ToolResult) {
        let session = match &mut self.session {
            Some(s) => s,
            None => return,
        };

        // tool result를 메시지에 추가
        session.push(Message {
            role: Role::User,
            content: vec![ContentBlock::ToolResult {
                tool_use_id: tool_use_id.to_string(),
                content: result.content,
                is_error: result.is_error,
            }],
        });

        // 자동으로 재요청 — provider가 다음 응답 생성
        self.start_streaming().await;
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
