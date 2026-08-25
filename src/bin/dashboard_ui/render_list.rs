// src/bin/dashboard_ui/render_list.rs

use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState, Wrap},
    Frame,
};

use super::styles::{
    confidence_color, impact_color, outer_block, ACCENT, ACCENT_DIM, BAD, GOOD, MUTED, TEXT, WARN,
};
use code_intelligence::analysis::explainability::RiskLevel;

pub fn render_list(
    f: &mut Frame,
    area: Rect,
    analysis: &code_intelligence::analysis::dead_code::DeadCodeAnalysis,
    state: &mut TableState,
    selected_evidence: Option<&code_intelligence::analysis::explainability::VerdictExplanation>,
) {
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    let header_text = Line::from(vec![
        Span::styled(
            format!(" {} dead functions ", analysis.functions.len()),
            Style::default()
                .fg(Color::Black)
                .bg(ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(" d ", Style::default().fg(Color::Black).bg(Color::Red)),
        Span::raw(" dead   "),
        Span::styled(" f ", Style::default().fg(Color::Black).bg(Color::Green)),
        Span::raw(" false-positive   "),
        Span::styled(" s ", Style::default().fg(Color::Black).bg(Color::Yellow)),
        Span::raw(" defer   "),
        Span::styled(" Enter ", Style::default().fg(Color::Black).bg(ACCENT)),
        Span::raw(" show evidence"),
    ]);
    let header = Paragraph::new(header_text);
    f.render_widget(header, chunks[0]);

    let rows: Vec<Row> = analysis
        .functions
        .iter()
        .enumerate()
        .map(|(i, func)| {
            let confidence_pct = func.score.score * 100.0;
            let conf_color = confidence_color(confidence_pct);

            let row_bg = if i % 2 == 0 {
                Color::Reset
            } else {
                Color::Rgb(20, 22, 30)
            };

            Row::new(vec![
                Cell::from(func.removal_order.to_string()).style(Style::default().fg(MUTED)),
                Cell::from(func.name.clone()).style(Style::default().fg(TEXT)),
                Cell::from(format!("{:.1}%", confidence_pct))
                    .style(Style::default().fg(conf_color).add_modifier(Modifier::BOLD)),
                Cell::from(format!("{:?}", func.score.level))
                    .style(Style::default().fg(conf_color)),
                Cell::from(func.impact.estimated_removal_impact.clone()).style(
                    Style::default().fg(impact_color(&func.impact.estimated_removal_impact)),
                ),
                Cell::from(func.impact.lines_of_code.to_string()).style(Style::default().fg(MUTED)),
            ])
            .style(Style::default().bg(row_bg))
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Percentage(25),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Percentage(20),
            Constraint::Length(5),
        ],
    )
    .header(
        Row::new(vec!["#", "Function", "Conf.", "Level", "Impact", "LOC"])
            .style(
                Style::default()
                    .fg(Color::Black)
                    .bg(ACCENT)
                    .add_modifier(Modifier::BOLD),
            )
            .height(1),
    )
    .block(outer_block("All Dead Functions"))
    .highlight_style(
        Style::default()
            .add_modifier(Modifier::REVERSED)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ");

    f.render_stateful_widget(table, chunks[1], state);

    if let Some(evidence) = selected_evidence {
        render_evidence_card(f, area, evidence);
    }
}

/// Render evidence card from VerdictExplanation
pub fn render_evidence_card(
    f: &mut Frame,
    area: Rect,
    evidence: &code_intelligence::analysis::explainability::VerdictExplanation,
) {
    use super::layout::centered_rect;
    use ratatui::layout::Alignment;

    let evidence_area = centered_rect(75, 65, area);

    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("🔍 Evidence for ", Style::default().fg(ACCENT)),
        Span::styled(
            &evidence.function_name,
            Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Verdict: ", Style::default().fg(ACCENT)),
        Span::styled(
            &evidence.verdict,
            Style::default().fg(if evidence.verdict.contains("Dead") {
                BAD
            } else {
                GOOD
            }),
        ),
        Span::styled(
            format!(" ({:.1}%)", evidence.confidence * 100.0),
            Style::default().fg(MUTED),
        ),
    ]));
    lines.push(Line::from(""));

    // Risk assessment
    lines.push(Line::from(vec![
        Span::styled("⚠️ Risk: ", Style::default().fg(WARN)),
        Span::styled(
            format!("{:?}", evidence.risk_assessment.overall_risk),
            Style::default().fg(
                if evidence.risk_assessment.overall_risk == RiskLevel::High {
                    BAD
                } else {
                    WARN
                },
            ),
        ),
        Span::raw(" | "),
        Span::styled(
            &evidence.risk_assessment.estimated_effort,
            Style::default().fg(MUTED),
        ),
    ]));
    lines.push(Line::from(""));

    // Evidence signals
    lines.push(Line::from(Span::styled(
        "📋 Evidence:",
        Style::default().fg(ACCENT),
    )));
    for e in &evidence.evidence {
        let emoji = if e.direction.contains("Dead") {
            "🔴"
        } else if e.direction.contains("Alive") {
            "🟢"
        } else {
            "⚪"
        };
        lines.push(Line::from(vec![
            Span::styled(emoji, Style::default().fg(ACCENT)),
            Span::raw(format!(" {}: ", e.signal_name)),
            Span::styled(&e.explanation, Style::default().fg(TEXT)),
            Span::styled(
                format!(" ({:.1}%)", e.value * 100.0),
                Style::default().fg(MUTED),
            ),
        ]));
    }
    lines.push(Line::from(""));

    // Timeline
    if evidence.timeline.commit_count > 0 {
        lines.push(Line::from(Span::styled(
            "📅 Timeline:",
            Style::default().fg(ACCENT),
        )));
        if let Some(author) = &evidence.timeline.author {
            lines.push(Line::from(vec![
                Span::raw("  Author: "),
                Span::styled(author, Style::default().fg(TEXT)),
            ]));
        }
        lines.push(Line::from(vec![
            Span::raw("  Commits: "),
            Span::styled(
                evidence.timeline.commit_count.to_string(),
                Style::default().fg(TEXT),
            ),
        ]));
        if let Some(modified) = &evidence.timeline.last_modified {
            lines.push(Line::from(vec![
                Span::raw("  Last modified: "),
                Span::styled(modified, Style::default().fg(MUTED)),
            ]));
        }
        lines.push(Line::from(""));
    }

    // Recommendation
    lines.push(Line::from(vec![
        Span::styled("💡 ", Style::default().fg(ACCENT)),
        Span::styled(&evidence.recommendation, Style::default().fg(TEXT)),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Press Enter or Esc to close",
        Style::default().fg(MUTED),
    )));

    let paragraph = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(ACCENT_DIM))
                .title(Span::styled(
                    " Evidence Details ",
                    Style::default().fg(ACCENT),
                )),
        )
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Left);

    f.render_widget(paragraph, evidence_area);
}
