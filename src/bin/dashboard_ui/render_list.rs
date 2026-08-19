// src/bin/dashboard_ui/render_list.rs

use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use super::styles::{
    confidence_color, impact_color, outer_block, status_color, status_emoji, ACCENT, MUTED, TEXT,
};

pub fn render_list(
    f: &mut Frame,
    area: Rect,
    analysis: &crate::DeadCodeAnalysis,
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
            let conf_color = confidence_color(func.confidence);
            let status_color = status_color(&func.status);
            let status_emoji = status_emoji(&func.status);

            // Subtle row striping so long lists don't blur together
            let row_bg = if i % 2 == 0 {
                Color::Reset
            } else {
                Color::Rgb(20, 22, 30)
            };

            Row::new(vec![
                Cell::from(func.order.to_string()).style(Style::default().fg(MUTED)),
                Cell::from(func.function_name.clone()).style(Style::default().fg(TEXT)),
                Cell::from(format!("{:.1}%", func.confidence))
                    .style(Style::default().fg(conf_color).add_modifier(Modifier::BOLD)),
                Cell::from(func.level.clone()).style(Style::default().fg(conf_color)),
                Cell::from(format!("{} {:?}", status_emoji, func.status))
                    .style(Style::default().fg(status_color)),
                Cell::from(func.impact.clone())
                    .style(Style::default().fg(impact_color(&func.impact))),
                Cell::from(func.loc.to_string()).style(Style::default().fg(MUTED)),
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
            Constraint::Length(16),
            Constraint::Percentage(20),
            Constraint::Length(5),
        ],
    )
    .header(
        Row::new(vec![
            "#", "Function", "Conf.", "Level", "Status", "Impact", "LOC",
        ])
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
