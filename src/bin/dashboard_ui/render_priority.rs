// src/bin/dashboard_ui/render_priority.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Paragraph, Wrap},
    Frame,
};

use super::styles::{confidence_color, outer_block, ACCENT, GOOD, MUTED, TEXT};

pub fn render_priority(
    f: &mut Frame,
    area: Rect,
    analysis: &code_intelligence::analysis::dead_code::DeadCodeAnalysis,
) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    let header_text = Line::from(vec![Span::styled(
        format!(
            " {} of {} functions, ranked by removal priority ",
            analysis.functions.len().min(20),
            analysis.functions.len()
        ),
        Style::default()
            .fg(Color::Black)
            .bg(ACCENT)
            .add_modifier(Modifier::BOLD),
    )]);
    f.render_widget(Paragraph::new(header_text), rows[0]);

    let mut lines: Vec<Line> = Vec::new();

    for (i, func) in analysis.functions.iter().take(20).enumerate() {
        let confidence_pct = func.score.score * 100.0;
        let color = confidence_color(confidence_pct);

        let rank_color = match i {
            0 => Color::Rgb(255, 215, 0),
            1 => Color::Rgb(192, 192, 192),
            2 => Color::Rgb(205, 127, 50),
            _ => MUTED,
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("{:>3}. ", func.removal_order),
                Style::default().fg(rank_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled("● ", Style::default().fg(color)),
            Span::styled(
                func.name.clone(),
                Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "  ({} · {:.1}%)",
                    func.impact.estimated_removal_impact, confidence_pct
                ),
                Style::default().fg(color),
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
            Span::styled(
                format!("  ·  {} LOC", func.impact.lines_of_code),
                Style::default().fg(MUTED),
            ),
        ]));

        // Show evidence if available
        if !func.score.factors.is_empty() {
            let evidence_text = func
                .score
                .factors
                .iter()
                .filter(|f| f.contribution > 0.0)
                .next()
                .map(|f| f.explanation.as_str())
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

    f.render_widget(paragraph, rows[1]);
}
