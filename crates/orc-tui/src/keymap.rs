use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use orc_core::event::{Event, ModalKind, Screen};

pub fn handle_global_key(key: KeyEvent) -> Option<Event> {
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => Some(Event::Quit),
        (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
            Some(Event::ShowModal(ModalKind::ProviderSelect))
        }
        (KeyModifiers::CONTROL, KeyCode::Char('m')) => {
            Some(Event::ShowModal(ModalKind::ModelSelect))
        }
        (KeyModifiers::CONTROL, KeyCode::Char(',')) => Some(Event::Navigate(Screen::Settings)),
        (_, KeyCode::Esc) => Some(Event::CloseModal),
        _ => None,
    }
}
