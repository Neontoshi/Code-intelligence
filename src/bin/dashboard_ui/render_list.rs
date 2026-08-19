// src/bin/dashboard_ui/render_list.rs

use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use super::styles::{confidence_color, impact_color, outer_block, ACCENT, MUTED, TEXT};

pub fn render_list(
    f: &mut Frame,
    area: Rect,
    analysis: &code_intelligence::analysis::dead_code::DeadCodeAnalysis,
    state: &mut TableState,
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
        Span::raw(" defer"),
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
}
