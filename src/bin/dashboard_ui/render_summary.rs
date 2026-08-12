// src/bin/dashboard_ui/render_summary.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::Span,
    widgets::{Gauge, Paragraph},
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
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    // Row 1: stat cards
    let cards = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ])
        .split(rows[0]);

    f.render_widget(
        stat_card("Total Functions", summary.total_functions.to_string(), TEXT),
        cards[0],
    );
    f.render_widget(
        stat_card(
            "Dead Functions",
            summary.dead_functions.to_string(),
            confidence_color(dead_pct),
        ),
        cards[1],
    );
    f.render_widget(
        stat_card(
            "Avg Confidence",
            format!("{:.1}%", summary.avg_confidence * 100.0),
            confidence_color(summary.avg_confidence * 100.0),
        ),
        cards[2],
    );
    f.render_widget(
        stat_card(
            "LOC Removable",
            summary.estimated_loc_removable.to_string(),
            ACCENT,
        ),
        cards[3],
    );

    // Row 2: secondary stat cards
    let cards2 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(rows[1]);

    f.render_widget(
        stat_card("Dead Types", summary.dead_types.to_string(), MUTED),
        cards2[0],
    );
    f.render_widget(
        stat_card("Dead Modules", summary.dead_modules.to_string(), MUTED),
        cards2[1],
    );
    f.render_widget(
        stat_card("Dead Files", summary.dead_files.to_string(), MUTED),
        cards2[2],
    );

    // Row 3: gauge
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

    // Row 4: status breakdown
    let mut status_counts: HashMap<CandidateStatus, usize> = HashMap::new();
    for func in &analysis.functions {
        *status_counts.entry(func.status.clone()).or_insert(0) += 1;
    }

    let status_line = vec![
        Span::styled("⏳ Pending: ", Style::default().fg(WARN)),
        Span::styled(
            status_counts
                .get(&CandidateStatus::Pending)
                .unwrap_or(&0)
                .to_string(),
            Style::default().fg(WARN),
        ),
        Span::raw(" | "),
        Span::styled("✅ Dead: ", Style::default().fg(BAD)),
        Span::styled(
            status_counts
                .get(&CandidateStatus::ConfirmedDead)
                .unwrap_or(&0)
                .to_string(),
            Style::default().fg(BAD),
        ),
        Span::raw(" | "),
        Span::styled("🚫 FP: ", Style::default().fg(GOOD)),
        Span::styled(
            status_counts
                .get(&CandidateStatus::FalsePositive)
                .unwrap_or(&0)
                .to_string(),
            Style::default().fg(GOOD),
        ),
        Span::raw(" | "),
        Span::styled(
            "⚠️ Stale: ",
            Style::default().fg(ratatui::style::Color::Magenta),
        ),
        Span::styled(
            status_counts
                .get(&CandidateStatus::Stale)
                .unwrap_or(&0)
                .to_string(),
            Style::default().fg(ratatui::style::Color::Magenta),
        ),
    ];

    let status_paragraph = Paragraph::new(ratatui::text::Line::from(status_line))
        .block(outer_block("Decision Status"))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(status_paragraph, rows[3]);

    // Row 5: metadata
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
    f.render_widget(metadata_paragraph, rows[4]);
}
