use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::theme::Theme;

pub struct StatusBar;

impl StatusBar {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        provider_name: &str,
        model: &str,
    ) {
        let line = Line::from(vec![
            Span::styled(" orc ", Theme::accent()),
            Span::styled("│ ", Theme::dimmed()),
            Span::styled(provider_name, Theme::base()),
            Span::styled(" (", Theme::dimmed()),
            Span::styled(model, Theme::base()),
            Span::styled(") ", Theme::dimmed()),
        ]);

        let bar = Paragraph::new(line).style(Theme::status_bar());
        frame.render_widget(bar, area);
    }
}
