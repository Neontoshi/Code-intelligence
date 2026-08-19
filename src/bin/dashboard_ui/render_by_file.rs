// src/bin/dashboard_ui/render_by_file.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Bar, BarChart, BarGroup, Paragraph, Wrap},
    Frame,
};
use std::collections::HashMap;

use super::styles::{
    confidence_color, outer_block, status_color, status_emoji, ACCENT, MUTED, TEXT, WARN,
};

pub fn render_by_file(f: &mut Frame, area: Rect, analysis: &crate::DeadCodeAnalysis) {
    let mut file_groups: HashMap<String, Vec<&crate::DeadFunctionExtended>> = HashMap::new();
    for func in &analysis.functions {
        file_groups.entry(func.file.clone()).or_default().push(func);
    }

    let mut files: Vec<_> = file_groups.into_iter().collect();
    files.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(9), Constraint::Min(6)])
        .split(area);

    // ---------- Top: function count per file, at a glance ----------
    let chart_files: Vec<_> = files.iter().take(10).collect();
    let file_bars: Vec<Bar> = chart_files
        .iter()
        .map(|(file, funcs)| {
            let short_file = file.split('/').last().unwrap_or(file);
            Bar::default()
                .label(Line::from(short_file.to_string()))
                .value(funcs.len() as u64)
                .style(Style::default().fg(WARN))
                .value_style(Style::default().fg(Color::Black).bg(WARN))
        })
        .collect();

    let by_file_chart = BarChart::default()
        .block(outer_block("Dead Functions per File (top 10)"))
        .direction(Direction::Horizontal)
        .data(BarGroup::default().bars(&file_bars))
        .bar_width(1)
        .bar_gap(1);
    f.render_widget(by_file_chart, rows[0]);

    // ---------- Bottom: drill-down detail, same info as before ----------
    let mut lines: Vec<Line> = Vec::new();
    for (file, funcs) in files.iter() {
        let short_file = file.split('/').last().unwrap_or(file);
        lines.push(Line::from(vec![
            Span::styled("▸ ", Style::default().fg(ACCENT)),
            Span::styled(
                short_file.to_string(),
                Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  ({} functions)", funcs.len()),
                Style::default().fg(MUTED),
            ),
        ]));
        let mut sorted_funcs = funcs.clone();
        sorted_funcs.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        for func in sorted_funcs.iter().take(5) {
            let color = confidence_color(func.confidence);
            let status_emoji = status_emoji(&func.status);
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled("● ", Style::default().fg(color)),
                Span::styled(func.function_name.clone(), Style::default().fg(TEXT)),
                Span::styled(
                    format!("  {:.1}%", func.confidence),
                    Style::default().fg(color),
                ),
                Span::styled(
                    format!(" {}", status_emoji),
                    Style::default().fg(status_color(&func.status)),
                ),
            ]));
        }
        if sorted_funcs.len() > 5 {
            lines.push(Line::from(Span::styled(
                format!("    … and {} more", sorted_funcs.len() - 5),
                Style::default().fg(MUTED),
            )));
        }
        lines.push(Line::from(""));
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .block(outer_block("Dead Functions by File — detail"))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, rows[1]);
}
