// src/bin/dashboard_ui/render_charts.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Bar, BarChart, BarGroup},
    Frame,
};
use std::collections::HashMap;

use super::styles::{impact_color, outer_block, BAD, GOOD, WARN};

pub fn render_charts(f: &mut Frame, area: Rect, analysis: &crate::DeadCodeAnalysis) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Confidence distribution
    let conf_order = ["Guaranteed", "VeryLikely", "Probably", "Uncertain", "Other"];
    let mut conf_counts: HashMap<&str, u64> = HashMap::new();
    for func in &analysis.functions {
        let level = if func.confidence >= 95.0 {
            "Guaranteed"
        } else if func.confidence >= 80.0 {
            "VeryLikely"
        } else if func.confidence >= 60.0 {
            "Probably"
        } else if func.confidence >= 40.0 {
            "Uncertain"
        } else {
            "Other"
        };
        *conf_counts.entry(level).or_insert(0) += 1;
    }

    let conf_bars: Vec<Bar> = conf_order
        .iter()
        .map(|level| {
            let count = *conf_counts.get(level).unwrap_or(&0);
            let color = match *level {
                "Guaranteed" | "VeryLikely" => BAD,
                "Probably" => WARN,
                _ => GOOD,
            };
            Bar::default()
                .label(Line::from(*level))
                .value(count)
                .style(Style::default().fg(color))
                .value_style(Style::default().fg(Color::Black).bg(color))
        })
        .collect();

    let confidence_chart = BarChart::default()
        .block(outer_block("Confidence Distribution"))
        .data(BarGroup::default().bars(&conf_bars))
        .bar_width(9)
        .bar_gap(2);
    f.render_widget(confidence_chart, chunks[0]);

    // Impact distribution
    let impact_order = ["High", "Medium", "Low"];
    let mut impact_counts: HashMap<&str, u64> = HashMap::new();
    for func in &analysis.functions {
        let impact = if func.impact.contains("High") {
            "High"
        } else if func.impact.contains("Medium") {
            "Medium"
        } else {
            "Low"
        };
        *impact_counts.entry(impact).or_insert(0) += 1;
    }

    let impact_bars: Vec<Bar> = impact_order
        .iter()
        .map(|level| {
            let count = *impact_counts.get(level).unwrap_or(&0);
            let color = impact_color(level);
            Bar::default()
                .label(Line::from(*level))
                .value(count)
                .style(Style::default().fg(color))
                .value_style(Style::default().fg(Color::Black).bg(color))
        })
        .collect();

    let impact_chart = BarChart::default()
        .block(outer_block("Impact Distribution"))
        .data(BarGroup::default().bars(&impact_bars))
        .bar_width(9)
        .bar_gap(2);
    f.render_widget(impact_chart, chunks[1]);
}
