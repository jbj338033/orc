use ratatui::style::{Color, Modifier, Style};

pub struct Theme;

impl Theme {
    pub fn base() -> Style {
        Style::default().fg(Color::White)
    }

    pub fn dimmed() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    pub fn accent() -> Style {
        Style::default().fg(Color::Cyan)
    }

    pub fn user() -> Style {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    }

    pub fn assistant() -> Style {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    }

    pub fn error() -> Style {
        Style::default().fg(Color::Red)
    }

    pub fn status_bar() -> Style {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    }

    pub fn selected() -> Style {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    }

    pub fn border() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    pub fn input() -> Style {
        Style::default().fg(Color::White)
    }
}
