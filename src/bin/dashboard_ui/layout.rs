// src/bin/dashboard_ui/layout.rs

use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((area.height * (100 - percent_y) / 100) / 2),
            Constraint::Length(area.height * percent_y / 100),
            Constraint::Length((area.height * (100 - percent_y) / 100) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((area.width * (100 - percent_x) / 100) / 2),
            Constraint::Length(area.width * percent_x / 100),
            Constraint::Length((area.width * (100 - percent_x) / 100) / 2),
        ])
        .split(popup_layout[1])[1]
}
