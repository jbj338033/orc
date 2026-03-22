use orc_core::provider::{ContentBlock, Message, Role};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
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
        let lines = self.build_lines(messages, streaming_text);
        let text = Text::from(lines);
        self.total_lines = text.lines.len() as u16;

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

    fn build_lines(&self, messages: &[Message], streaming_text: &str) -> Vec<Line<'static>> {
        let mut lines: Vec<Line<'static>> = Vec::new();

        for msg in messages {
            if matches!(msg.role, Role::System) {
                continue;
            }

            // tool_result 메시지는 별도 표시
            let has_tool_result = msg
                .content
                .iter()
                .any(|b| matches!(b, ContentBlock::ToolResult { .. }));
            if has_tool_result {
                for block in &msg.content {
                    if let ContentBlock::ToolResult {
                        content, is_error, ..
                    } = block
                    {
                        let style = if *is_error {
                            Theme::error()
                        } else {
                            Style::default().fg(Color::DarkGray)
                        };
                        let prefix = if *is_error { "  error: " } else { "  " };
                        for line in content.lines().take(30) {
                            lines.push(Line::from(Span::styled(
                                format!("{prefix}{line}"),
                                style,
                            )));
                        }
                        if content.lines().count() > 30 {
                            lines.push(Line::from(Span::styled(
                                "  ... (truncated)".to_string(),
                                Theme::dimmed(),
                            )));
                        }
                    }
                }
                lines.push(Line::from(""));
                continue;
            }

            let (label, style) = match msg.role {
                Role::User => ("You", Theme::user()),
                Role::Assistant => ("Assistant", Theme::assistant()),
                Role::System => unreachable!(),
            };

            lines.push(Line::from(Span::styled(format!("{label}:"), style)));

            for block in &msg.content {
                match block {
                    ContentBlock::Text { text } => {
                        lines.extend(render_markdown(&text.clone()));
                    }
                    ContentBlock::ToolUse { name, input, .. } => {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "  > ".to_string(),
                                Style::default().fg(Color::Yellow),
                            ),
                            Span::styled(
                                name.clone(),
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ]));
                        if let Some(obj) = input.as_object() {
                            for (k, v) in obj {
                                let val = match v {
                                    serde_json::Value::String(s) => {
                                        if s.len() > 80 {
                                            format!("{}...", &s[..80])
                                        } else {
                                            s.clone()
                                        }
                                    }
                                    other => {
                                        let s = other.to_string();
                                        if s.len() > 80 {
                                            format!("{}...", &s[..80])
                                        } else {
                                            s
                                        }
                                    }
                                };
                                lines.push(Line::from(Span::styled(
                                    format!("    {k}: {val}"),
                                    Theme::dimmed(),
                                )));
                            }
                        }
                    }
                    ContentBlock::ToolResult { .. } => {}
                }
            }

            lines.push(Line::from(""));
        }

        if !streaming_text.is_empty() {
            lines.push(Line::from(Span::styled(
                "Assistant:".to_string(),
                Theme::assistant(),
            )));
            lines.extend(render_markdown(&streaming_text.to_string()));
            lines.push(Line::from(Span::styled(
                "  ▋".to_string(),
                Theme::accent(),
            )));
        }

        lines
    }
}

fn render_markdown(text: &str) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut in_code_block = false;

    for line in text.lines() {
        if line.starts_with("```") {
            if in_code_block {
                in_code_block = false;
                lines.push(Line::from(Span::styled(
                    "  └───────────────────────────────────────┘".to_string(),
                    Style::default().fg(Color::DarkGray),
                )));
            } else {
                in_code_block = true;
                let lang = line.trim_start_matches('`');
                let header = if lang.is_empty() {
                    "  ┌─ code ─────────────────────────────────┐".to_string()
                } else {
                    format!("  ┌─ {lang} ─────────────────────────────────┐")
                };
                lines.push(Line::from(Span::styled(
                    header,
                    Style::default().fg(Color::DarkGray),
                )));
            }
            continue;
        }

        if in_code_block {
            lines.push(Line::from(Span::styled(
                format!("  │ {line}"),
                Style::default().fg(Color::Green),
            )));
        } else if line.starts_with("# ") {
            lines.push(Line::from(Span::styled(
                format!("  {line}"),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )));
        } else if line.starts_with("## ") || line.starts_with("### ") {
            lines.push(Line::from(Span::styled(
                format!("  {line}"),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
        } else if line.starts_with("- ") || line.starts_with("* ") {
            lines.push(Line::from(vec![
                Span::styled("  • ".to_string(), Style::default().fg(Color::Cyan)),
                Span::styled(line[2..].to_string(), Style::default().fg(Color::White)),
            ]));
        } else if line.starts_with("> ") {
            lines.push(Line::from(Span::styled(
                format!("  │ {}", &line[2..]),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )));
        } else {
            let spans = parse_inline_code(line);
            let mut result: Vec<Span<'static>> = vec![Span::raw("  ".to_string())];
            result.extend(spans);
            lines.push(Line::from(result));
        }
    }

    if in_code_block {
        lines.push(Line::from(Span::styled(
            "  └───────────────────────────────────────┘".to_string(),
            Style::default().fg(Color::DarkGray),
        )));
    }

    lines
}

fn parse_inline_code(text: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut rest = text;

    while let Some(start) = rest.find('`') {
        if start > 0 {
            spans.push(Span::styled(
                rest[..start].replace("**", "").replace("__", ""),
                Style::default().fg(Color::White),
            ));
        }
        let after_tick = &rest[start + 1..];
        if let Some(end) = after_tick.find('`') {
            spans.push(Span::styled(
                after_tick[..end].to_string(),
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::Rgb(40, 40, 40)),
            ));
            rest = &after_tick[end + 1..];
        } else {
            spans.push(Span::styled(
                rest[start..].to_string(),
                Style::default().fg(Color::White),
            ));
            return spans;
        }
    }

    if !rest.is_empty() {
        spans.push(Span::styled(
            rest.replace("**", "").replace("__", ""),
            Style::default().fg(Color::White),
        ));
    }

    spans
}
