// src/bin/dashboard_ui/render_dialogs.rs

use ratatui::{
    layout::{Alignment, Rect},
    style::Style,
    text::Span,
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use super::layout::centered_rect;
use super::styles::{ACCENT, WARN};
use crate::Action;

pub fn render_confirmation_dialog(f: &mut Frame, area: Rect, pending_action: &Option<Action>) {
    let dialog_area = centered_rect(50, 20, area);
    let dialog = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(WARN))
        .title(Span::styled(" Confirm Action ", Style::default().fg(WARN)));

    let text = match pending_action {
        Some(Action::ConfirmDead(_)) => "Confirm this function as DEAD?\n\n[y] Yes  [n] No",
        Some(Action::FalsePositive(_)) => {
            "Mark this function as FALSE POSITIVE?\n\n[y] Yes  [n] No"
        }
        Some(Action::Defer(_)) => "Defer this decision?\n\n[y] Yes  [n] No",
        None => "Confirm?\n\n[y] Yes  [n] No",
    };

    let paragraph = Paragraph::new(text)
        .block(dialog)
        .alignment(Alignment::Center);
    f.render_widget(paragraph, dialog_area);
}

pub fn render_reason_dialog(f: &mut Frame, area: Rect, reason_input: &str) {
    let dialog_area = centered_rect(50, 20, area);
    let dialog = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(ACCENT))
        .title(Span::styled(
            " Reason for False Positive ",
            Style::default().fg(ACCENT),
        ));

    let text = format!(
        "Enter reason (optional):\n\n{}\n\n[Enter] Save  [Esc] Cancel",
        reason_input
    );

    let paragraph = Paragraph::new(text)
        .block(dialog)
        .alignment(Alignment::Center);
    f.render_widget(paragraph, dialog_area);
}
