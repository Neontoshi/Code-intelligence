// src/bin/dashboard_ui/render_help.rs

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::styles::ACCENT;

pub fn render_help(f: &mut Frame, area: Rect, show_dialog: bool) {
    let help = if show_dialog {
        Line::from("")
    } else {
        Line::from(vec![
            Span::styled(" Tab/←→ ", Style::default().fg(Color::Black).bg(ACCENT)),
            Span::raw(" switch   "),
            Span::styled(" ↑↓/jk ", Style::default().fg(Color::Black).bg(ACCENT)),
            Span::raw(" move   "),
            Span::styled(" d ", Style::default().fg(Color::Black).bg(Color::Red)),
            Span::raw(" dead   "),
            Span::styled(" f ", Style::default().fg(Color::Black).bg(Color::Green)),
            Span::raw(" false-positive   "),
            Span::styled(" s ", Style::default().fg(Color::Black).bg(Color::Yellow)),
            Span::raw(" defer   "),
            Span::styled(" q ", Style::default().fg(Color::Black).bg(Color::Red)),
            Span::raw(" quit"),
        ])
    };

    let help_paragraph = Paragraph::new(help).style(Style::default().fg(Color::Gray));
    f.render_widget(help_paragraph, area);
}
