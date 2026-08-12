// src/bin/dashboard_ui/render_priority.rs

use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span, Text},
    widgets::{Paragraph, Wrap},
    Frame,
};

use super::styles::{confidence_color, outer_block, status_color, status_emoji, GOOD, MUTED, TEXT};

pub fn render_priority(f: &mut Frame, area: Rect, analysis: &crate::DeadCodeAnalysis) {
    let mut lines: Vec<Line> = Vec::new();

    for func in analysis.functions.iter().take(20) {
        let color = confidence_color(func.confidence);
        let status_emoji = status_emoji(&func.status);

        lines.push(Line::from(vec![
            Span::styled(format!("{:>3}. ", func.order), Style::default().fg(MUTED)),
            Span::styled("● ", Style::default().fg(color)),
            Span::styled(
                func.function_name.clone(),
                Style::default()
                    .fg(TEXT)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            ),
            Span::styled(
                format!("  ({} · {:.1}%)", func.impact, func.confidence),
                Style::default().fg(color),
            ),
            Span::styled(
                format!(" {}", status_emoji),
                Style::default().fg(status_color(&func.status)),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::raw("     "),
            Span::styled(
                func.file
                    .split('/')
                    .last()
                    .unwrap_or(&func.file)
                    .to_string(),
                Style::default().fg(MUTED),
            ),
            Span::styled(format!("  ·  {} LOC", func.loc), Style::default().fg(MUTED)),
        ]));

        // Show evidence if available
        if !func.evidence.is_empty() {
            let evidence_text = func
                .evidence
                .first()
                .map(|s| s.as_str())
                .unwrap_or("No evidence");
            lines.push(Line::from(vec![
                Span::raw("     "),
                Span::styled("✓ ", Style::default().fg(GOOD)),
                Span::styled(evidence_text, Style::default().fg(MUTED)),
            ]));
        }
        lines.push(Line::from(""));
    }

    if analysis.functions.len() > 20 {
        lines.push(Line::from(Span::styled(
            format!("… and {} more functions", analysis.functions.len() - 20),
            Style::default().fg(MUTED),
        )));
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .block(outer_block("Priority Removal Order"))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}
