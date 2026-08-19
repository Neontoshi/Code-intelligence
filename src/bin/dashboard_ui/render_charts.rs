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

pub fn render_charts(
    f: &mut Frame,
    area: Rect,
    analysis: &code_intelligence::analysis::dead_code::DeadCodeAnalysis,
) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(11), Constraint::Min(6)])
        .split(area);

    let top_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[0]);

    // Confidence distribution
    let conf_order = ["Guaranteed", "VeryLikely", "Probably", "Uncertain", "Other"];
    let mut conf_counts: HashMap<&str, u64> = HashMap::new();
    for func in &analysis.functions {
        let confidence_pct = func.score.score * 100.0;
        let level = if confidence_pct >= 95.0 {
            "Guaranteed"
        } else if confidence_pct >= 80.0 {
            "VeryLikely"
        } else if confidence_pct >= 60.0 {
            "Probably"
        } else if confidence_pct >= 40.0 {
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
    f.render_widget(confidence_chart, top_cols[0]);

    // Impact distribution
    let impact_order = ["High", "Medium", "Low"];
    let mut impact_counts: HashMap<&str, u64> = HashMap::new();
    for func in &analysis.functions {
        let impact_str = func.impact.estimated_removal_impact.as_str();
        let impact = if impact_str.contains("High") {
            "High"
        } else if impact_str.contains("Medium") {
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
    f.render_widget(impact_chart, top_cols[1]);

    //  Dead LOC by file
    let mut loc_by_file: HashMap<&str, u64> = HashMap::new();
    for func in &analysis.functions {
        let basename = func.file.rsplit('/').next().unwrap_or(&func.file);
        *loc_by_file.entry(basename).or_insert(0) += func.impact.lines_of_code as u64;
    }

    let mut loc_list: Vec<(&str, u64)> = loc_by_file.into_iter().collect();
    loc_list.sort_by(|a, b| b.1.cmp(&a.1));
    loc_list.truncate(8);

    let loc_bars: Vec<Bar> = loc_list
        .iter()
        .map(|(file, loc)| {
            Bar::default()
                .label(Line::from(*file))
                .value(*loc)
                .style(Style::default().fg(BAD))
                .value_style(Style::default().fg(Color::Black).bg(BAD))
        })
        .collect();

    let loc_chart = BarChart::default()
        .block(outer_block("Dead LOC by File (top 8)"))
        .direction(Direction::Horizontal)
        .data(BarGroup::default().bars(&loc_bars))
        .bar_width(1)
        .bar_gap(1);
    f.render_widget(loc_chart, rows[1]);
}
