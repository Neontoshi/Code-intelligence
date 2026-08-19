// src/bin/dashboard_ui/styles.rs

use crate::CandidateStatus;
use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
    widgets::Block,
};

// ---------- Palette ----------
pub const ACCENT: Color = Color::Cyan;
pub const ACCENT_DIM: Color = Color::DarkGray;
pub const GOOD: Color = Color::Green;
pub const WARN: Color = Color::Yellow;
pub const BAD: Color = Color::Red;
pub const TEXT: Color = Color::White;
pub const MUTED: Color = Color::Gray;

pub fn confidence_color(confidence: f64) -> Color {
    if confidence >= 80.0 {
        BAD
    } else if confidence >= 60.0 {
        WARN
    } else {
        GOOD
    }
}

pub fn _status_color(status: &CandidateStatus) -> Color {
    match status {
        CandidateStatus::Pending => WARN,
        CandidateStatus::ConfirmedDead => BAD,
        CandidateStatus::ConfirmedAlive => GOOD,
        CandidateStatus::FalsePositive => GOOD,
        CandidateStatus::Deferred => MUTED,
        CandidateStatus::Stale => Color::Magenta,
    }
}

pub fn _status_emoji(status: &CandidateStatus) -> &'static str {
    match status {
        CandidateStatus::Pending => "⏳",
        CandidateStatus::ConfirmedDead => "✅",
        CandidateStatus::ConfirmedAlive => "❌",
        CandidateStatus::FalsePositive => "🚫",
        CandidateStatus::Deferred => "⏰",
        CandidateStatus::Stale => "⚠️",
    }
}

pub fn impact_color(impact: &str) -> Color {
    if impact.contains("High") {
        BAD
    } else if impact.contains("Medium") {
        WARN
    } else {
        GOOD
    }
}

pub fn outer_block(title: &str) -> Block<'static> {
    Block::default()
        .borders(ratatui::widgets::Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(ACCENT_DIM))
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
}

pub fn stat_card(title: &str, value: String, color: Color) -> ratatui::widgets::Paragraph<'static> {
    ratatui::widgets::Paragraph::new(vec![
        ratatui::text::Line::from(Span::styled(
            value,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        ratatui::text::Line::from(Span::styled(title.to_string(), Style::default().fg(MUTED))),
    ])
    .alignment(ratatui::layout::Alignment::Center)
    .block(outer_block(""))
}
