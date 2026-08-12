// src/bin/dashboard_ui/render_by_file.rs

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Paragraph, Wrap},
    Frame,
};
use std::collections::HashMap;

use super::styles::{
    confidence_color, outer_block, status_color, status_emoji, ACCENT, MUTED, TEXT,
};

pub fn render_by_file(f: &mut Frame, area: Rect, analysis: &crate::DeadCodeAnalysis) {
    let mut file_groups: HashMap<String, Vec<&crate::DeadFunctionExtended>> = HashMap::new();
    for func in &analysis.functions {
        file_groups.entry(func.file.clone()).or_default().push(func);
    }

    let mut files: Vec<_> = file_groups.into_iter().collect();
    files.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

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
        .block(outer_block("Dead Functions by File"))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}
