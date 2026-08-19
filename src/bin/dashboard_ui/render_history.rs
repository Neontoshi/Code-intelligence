// src/bin/dashboard_ui/render_history.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Paragraph, Wrap},
    Frame,
};

use super::styles::{outer_block, ACCENT, BAD, GOOD, MUTED, TEXT, WARN};
use crate::{DashboardDecision, DecisionType};

pub fn render_history(f: &mut Frame, area: Rect, decisions: &[DashboardDecision]) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    let header_text = Line::from(vec![Span::styled(
        format!(" {} decisions recorded ", decisions.len()),
        Style::default()
            .fg(Color::Black)
            .bg(ACCENT)
            .add_modifier(Modifier::BOLD),
    )]);
    f.render_widget(Paragraph::new(header_text), rows[0]);

    let mut lines = Vec::new();

    if decisions.is_empty() {
        lines.push(Line::from("No decisions recorded yet."));
        lines.push(Line::from(
            "Use [d] to confirm dead, [f] for false positive, [s] to defer.",
        ));
    } else {
        for decision in decisions.iter().rev().take(100) {
            let status_emoji = match decision.decision {
                DecisionType::ConfirmedDead => "✅",
                DecisionType::ConfirmedAlive => "❌",
                DecisionType::FalsePositive => "🚫",
                DecisionType::Deferred => "⏳",
                DecisionType::Stale => "⚠️",
                DecisionType::NeedsInvestigation => "🔍",
            };

            let date_str = chrono::DateTime::from_timestamp(decision.timestamp, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or("unknown".to_string());

            lines.push(Line::from(vec![
                Span::styled(status_emoji, Style::default().fg(GOOD)),
                Span::raw(format!(
                    " {} ",
                    decision
                        .candidate_id
                        .split("::")
                        .last()
                        .unwrap_or("unknown")
                )),
                Span::styled(
                    format!("{:?}", decision.decision),
                    Style::default().fg(ACCENT),
                ),
                Span::raw(format!(" by {} ", decision.user)),
                Span::styled(date_str, Style::default().fg(MUTED)),
            ]));

            if let Some(reason) = &decision.reason {
                lines.push(Line::from(vec![
                    Span::raw("  └─ "),
                    Span::styled(reason, Style::default().fg(MUTED)),
                ]));
            }
            lines.push(Line::from(""));
        }

        // Add summary
        let total_decisions = decisions.len();
        let confirmed_dead = decisions
            .iter()
            .filter(|d| d.decision == DecisionType::ConfirmedDead)
            .count();
        let false_positives = decisions
            .iter()
            .filter(|d| d.decision == DecisionType::FalsePositive)
            .count();
        let deferred = decisions
            .iter()
            .filter(|d| d.decision == DecisionType::Deferred)
            .count();

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("📊 Summary: ", Style::default().fg(ACCENT)),
            Span::styled(
                format!("{} total decisions", total_decisions),
                Style::default().fg(TEXT),
            ),
            Span::raw(" | "),
            Span::styled(
                format!("✅ {} dead", confirmed_dead),
                Style::default().fg(BAD),
            ),
            Span::raw(" | "),
            Span::styled(
                format!("🚫 {} false positives", false_positives),
                Style::default().fg(GOOD),
            ),
            Span::raw(" | "),
            Span::styled(
                format!("⏳ {} deferred", deferred),
                Style::default().fg(WARN),
            ),
        ]));
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .block(outer_block("Decision History"))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, rows[1]);
}
