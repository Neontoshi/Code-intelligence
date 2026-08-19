// src/bin/dashboard_ui/render_summary.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Gauge, Paragraph},
    Frame,
};
use std::collections::HashMap;

use super::styles::*;
use crate::{App, CandidateStatus, DeadCodeAnalysis};

pub fn render_summary(f: &mut Frame, area: Rect, analysis: &DeadCodeAnalysis, app: &App) {
    let summary = &analysis.summary;
    let dead_pct = if summary.total_functions > 0 {
        summary.dead_functions as f64 / summary.total_functions as f64 * 100.0
    } else {
        0.0
    };

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // hero row
            Constraint::Length(3), // slim stat strip
            Constraint::Length(4), // gauge (taller than before)
            Constraint::Length(4), // decision status bar
            Constraint::Min(6),    // dead functions by file (fills old dead space)
            Constraint::Length(3), // metadata
        ])
        .split(area);

    // ---------- Row 1: hero cards ----------
    let hero_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[0]);

    f.render_widget(
        hero_stat(
            "DEAD FUNCTIONS",
            summary.dead_functions.to_string(),
            confidence_color(dead_pct),
        ),
        hero_cols[0],
    );
    f.render_widget(
        hero_stat(
            "LOC REMOVABLE",
            summary.estimated_loc_removable.to_string(),
            ACCENT,
        ),
        hero_cols[1],
    );

    // ---------- Row 2: slim stat strip ----------
    let strip_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .split(rows[1]);

    f.render_widget(
        stat_card("Total Functions", summary.total_functions.to_string(), TEXT),
        strip_cols[0],
    );
    f.render_widget(
        stat_card(
            "Avg Confidence",
            format!("{:.1}%", summary.avg_confidence * 100.0),
            confidence_color(summary.avg_confidence * 100.0),
        ),
        strip_cols[1],
    );
    f.render_widget(
        stat_card("Dead Types", summary.dead_types.to_string(), MUTED),
        strip_cols[2],
    );
    f.render_widget(
        stat_card("Dead Modules", summary.dead_modules.to_string(), MUTED),
        strip_cols[3],
    );
    f.render_widget(
        stat_card("Dead Files", summary.dead_files.to_string(), MUTED),
        strip_cols[4],
    );

    // ---------- Row 3: gauge (now taller, label rendered inside) ----------
    let gauge = Gauge::default()
        .block(outer_block("Dead Code Share"))
        .gauge_style(
            Style::default()
                .fg(confidence_color(dead_pct))
                .bg(ratatui::style::Color::Black),
        )
        .ratio((dead_pct / 100.0).clamp(0.0, 1.0))
        .label(format!("{:.1}% of functions are dead", dead_pct));
    f.render_widget(gauge, rows[2]);

    // ---------- Row 4: decision status as a horizontal bar ----------
    let mut status_counts: HashMap<CandidateStatus, usize> = HashMap::new();
    for func in &analysis.functions {
        *status_counts.entry(func.status.clone()).or_insert(0) += 1;
    }

    let status_order = [
        CandidateStatus::Pending,
        CandidateStatus::ConfirmedDead,
        CandidateStatus::FalsePositive,
        CandidateStatus::Stale,
    ];
    let status_labels = ["Pending", "Dead", "FP", "Stale"];

    let status_bars: Vec<Bar> = status_order
        .iter()
        .zip(status_labels.iter())
        .map(|(status, label)| {
            let count = *status_counts.get(status).unwrap_or(&0) as u64;
            let color = status_color(status);
            Bar::default()
                .label(Line::from(*label))
                .value(count)
                .style(Style::default().fg(color))
                .value_style(Style::default().fg(ratatui::style::Color::Black).bg(color))
        })
        .collect();

    let status_chart = BarChart::default()
        .block(outer_block("Decision Status"))
        .direction(Direction::Horizontal)
        .data(BarGroup::default().bars(&status_bars))
        .bar_width(1)
        .bar_gap(1);
    f.render_widget(status_chart, rows[3]);

    // ---------- Row 5: dead functions by file (fills the old empty space) ----------
    let mut file_counts: HashMap<&str, u64> = HashMap::new();
    for func in &analysis.functions {
        if func.status != CandidateStatus::FalsePositive
            && func.status != CandidateStatus::ConfirmedAlive
        {
            let basename = func.file.rsplit('/').next().unwrap_or(&func.file);
            *file_counts.entry(basename).or_insert(0) += 1;
        }
    }

    let mut file_list: Vec<(&str, u64)> = file_counts.into_iter().collect();
    file_list.sort_by(|a, b| b.1.cmp(&a.1));
    file_list.truncate(8);

    let file_bars: Vec<Bar> = file_list
        .iter()
        .map(|(file, count)| {
            Bar::default()
                .label(Line::from(*file))
                .value(*count)
                .style(Style::default().fg(WARN))
                .value_style(Style::default().fg(ratatui::style::Color::Black).bg(WARN))
        })
        .collect();

    let by_file_chart = BarChart::default()
        .block(outer_block("Dead Functions by File (top 8)"))
        .direction(Direction::Horizontal)
        .data(BarGroup::default().bars(&file_bars))
        .bar_width(1)
        .bar_gap(1);
    f.render_widget(by_file_chart, rows[4]);

    // ---------- Row 6: metadata ----------
    let metadata = if let Some(meta) = &app.analysis_metadata {
        format!(
            "ID: {} | Model: {} | Commit: {} | Analyzed: {}",
            &meta.analysis_id[..8],
            meta.model_version
                .split('/')
                .last()
                .unwrap_or(&meta.model_version),
            &meta.source_commit[..8],
            chrono::DateTime::from_timestamp(meta.analysis_timestamp, 0)
                .map(|dt| dt.format("%m-%d %H:%M").to_string())
                .unwrap_or("unknown".to_string())
        )
    } else {
        "No analysis metadata available".to_string()
    };

    let metadata_paragraph = Paragraph::new(metadata)
        .block(outer_block("Metadata"))
        .style(Style::default().fg(MUTED));
    f.render_widget(metadata_paragraph, rows[5]);
}

/// A large, high-contrast stat block for the two numbers that matter most.
/// No big-text crate needed — weight comes from color, bold, borders, and
/// generous vertical padding rather than literal font size.
fn hero_stat(title: &str, value: String, color: ratatui::style::Color) -> Paragraph<'static> {
    Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            value,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            title.to_string(),
            Style::default().fg(MUTED).add_modifier(Modifier::BOLD),
        )),
    ])
    .alignment(ratatui::layout::Alignment::Center)
    .block(outer_block(""))
}
