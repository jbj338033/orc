use orc_core::provider::{ContentBlock, Message, Role};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap};

use crate::theme::Theme;

pub struct Chat {
    scroll: u16,
    total_lines: u16,
}

impl Chat {
    pub fn new() -> Self {
        Self {
            scroll: 0,
            total_lines: 0,
        }
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(3);
    }

    pub fn scroll_down(&mut self) {
        self.scroll = self
            .scroll
            .saturating_add(3)
            .min(self.total_lines.saturating_sub(1));
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll = self.total_lines.saturating_sub(1);
    }

    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        messages: &[Message],
        streaming_text: &str,
    ) {
        let mut lines: Vec<Line> = Vec::new();

        for msg in messages {
            let (label, style) = match msg.role {
                Role::User => ("You", Theme::user()),
                Role::Assistant => ("Assistant", Theme::assistant()),
                Role::System => ("System", Theme::dimmed()),
            };

            lines.push(Line::from(Span::styled(
                format!("{label}:"),
                style,
            )));

            for block in &msg.content {
                match block {
                    ContentBlock::Text { text } => {
                        for line in text.lines() {
                            lines.push(Line::from(format!("  {line}")));
                        }
                    }
                    ContentBlock::ToolUse { name, .. } => {
                        lines.push(Line::from(Span::styled(
                            format!("  [tool: {name}]"),
                            Theme::dimmed(),
                        )));
                    }
                    ContentBlock::ToolResult { content, is_error, .. } => {
                        let style = if *is_error {
                            Theme::error()
                        } else {
                            Theme::dimmed()
                        };
                        for line in content.lines().take(20) {
                            lines.push(Line::from(Span::styled(
                                format!("  {line}"),
                                style,
                            )));
                        }
                    }
                }
            }

            lines.push(Line::from(""));
        }

        // 스트리밍 중인 텍스트
        if !streaming_text.is_empty() {
            lines.push(Line::from(Span::styled(
                "Assistant:",
                Theme::assistant(),
            )));
            for line in streaming_text.lines() {
                lines.push(Line::from(format!("  {line}")));
            }
            // 커서
            lines.push(Line::from(Span::styled("  ▋", Theme::accent())));
        }

        let text = Text::from(lines.clone());
        self.total_lines = text.lines.len() as u16;

        // 자동 스크롤: 맨 아래에 있으면 새 내용이 오면 따라감
        if self.total_lines > area.height {
            let max_scroll = self.total_lines.saturating_sub(area.height);
            if self.scroll >= max_scroll.saturating_sub(5) {
                self.scroll = max_scroll;
            }
        }

        let paragraph = Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));

        frame.render_widget(paragraph, area);

        // 스크롤바
        if self.total_lines > area.height {
            let mut scrollbar_state = ScrollbarState::new(self.total_lines as usize)
                .position(self.scroll as usize)
                .viewport_content_length(area.height as usize);
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight),
                area,
                &mut scrollbar_state,
            );
        }
    }
}
