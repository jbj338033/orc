use crate::provider::StreamEvent;
use crate::tool::ToolResult;

#[derive(Debug, Clone)]
pub enum Event {
    Quit,
    Navigate(Screen),
    ShowModal(ModalKind),
    CloseModal,
    Toast(String, ToastLevel),

    // provider
    SendMessage(String),
    Stream(StreamEvent),

    // tool
    ToolStart { id: String, name: String },
    ToolDone { id: String, result: ToolResult },

    // config
    ProviderChanged,
    ModelChanged(String),

    // oauth
    StartOAuth { provider_id: String },
    OAuthDone,
    OAuthError(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Setup,
    Main,
    Settings,
}

#[derive(Debug, Clone)]
pub enum ModalKind {
    ProviderSelect,
    ModelSelect,
    Confirm { message: String },
}

#[derive(Debug, Clone)]
pub enum ToastLevel {
    Info,
    Error,
}
