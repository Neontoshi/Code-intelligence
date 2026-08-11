// src/bin/dead_code_dashboard.rs

use code_intelligence::analysis::dead_code::DeadCodeDetector;
use code_intelligence::Pipeline;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Bar, BarChart, BarGroup, Block, BorderType, Borders, Cell, Gauge, Paragraph, Row, Table,
        TableState, Tabs,
    },
    Frame, Terminal,
};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

// ---------- Palette ----------
// Keeping a small, consistent palette makes the whole dashboard read as one
// coherent UI instead of a pile of ad-hoc colors.
const ACCENT: Color = Color::Cyan;
const ACCENT_DIM: Color = Color::DarkGray;
const GOOD: Color = Color::Green;
const WARN: Color = Color::Yellow;
const BAD: Color = Color::Red;
const TEXT: Color = Color::White;
const MUTED: Color = Color::Gray;

#[derive(Debug, Clone)]
struct DeadFunction {
    name: String,
    file: String,
    confidence: f64,
    level: String,
    impact: String,
    loc: usize,
    order: usize,
}

#[derive(Debug, Clone)]
struct AnalysisSummary {
    total_functions: usize,
    dead_functions: usize,
    dead_types: usize,
    dead_modules: usize,
    dead_files: usize,
    avg_confidence: f64,
    estimated_loc_removable: usize,
}

#[derive(Debug, Clone)]
struct DeadCodeAnalysis {
    summary: AnalysisSummary,
    functions: Vec<DeadFunction>,
}

struct App {
    analysis: Option<DeadCodeAnalysis>,
    table_state: TableState,
    selected_tab: usize,
    tabs: Vec<String>,
    loading: bool,
    error: Option<String>,
}

impl App {
    fn new() -> Self {
        Self {
            analysis: None,
            table_state: TableState::default(),
            selected_tab: 0,
            tabs: vec![
                "Summary".to_string(),
                "Charts".to_string(),
                "List".to_string(),
                "By File".to_string(),
                "Priority".to_string(),
            ],
            loading: true,
            error: None,
        }
    }

    fn load_data(&mut self, path: PathBuf) {
        self.loading = true;

        let result = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut pipeline = Pipeline::new();
                let analysis = pipeline
                    .process_project(&path)
                    .await
                    .map_err(|e| e.to_string())?;

                let dead_analysis = DeadCodeDetector::analyze(
                    &analysis.call_graph,
                    &analysis.type_graph,
                    &analysis.import_graph,
                    &analysis.dependency_graph,
                    &analysis.files,
                    None,
                );

                Ok::<_, String>(dead_analysis)
            })
        });

        match result.join().unwrap() {
            Ok(analysis) => {
                let mut functions: Vec<DeadFunction> = analysis
                    .functions
                    .iter()
                    .map(|f| DeadFunction {
                        name: f.name.clone(),
                        file: f.file.clone(),
                        confidence: f.score.score * 100.0,
                        level: format!("{:?}", f.score.level),
                        impact: f.impact.estimated_removal_impact.clone(),
                        loc: f.impact.lines_of_code,
                        order: f.removal_order,
                    })
                    .collect();
                functions.sort_by_key(|f| f.order);

                self.analysis = Some(DeadCodeAnalysis {
                    summary: AnalysisSummary {
                        total_functions: analysis.summary.total_functions,
                        dead_functions: analysis.summary.dead_functions,
                        dead_types: analysis.summary.dead_types,
                        dead_modules: analysis.summary.dead_modules,
                        dead_files: analysis.summary.dead_files,
                        avg_confidence: analysis.summary.avg_confidence,
                        estimated_loc_removable: analysis.summary.estimated_loc_removable,
                    },
                    functions,
                });
                self.table_state.select(Some(0));
                self.loading = false;
            }
            Err(e) => {
                self.error = Some(format!("Failed to analyze: {}", e));
                self.loading = false;
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let path = if args.len() >= 2 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from(".")
    };

    println!("Analyzing dead code in: {:?}", path);
    println!("Loading...");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    app.load_data(path);

    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                        app.selected_tab = (app.selected_tab + 1) % app.tabs.len();
                    }
                    KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
                        app.selected_tab = if app.selected_tab == 0 {
                            app.tabs.len() - 1
                        } else {
                            app.selected_tab - 1
                        };
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let count = app
                            .analysis
                            .as_ref()
                            .map(|a| a.functions.len())
                            .unwrap_or(0);
                        let selected = app.table_state.selected().unwrap_or(0);
                        if count > 0 && selected < count - 1 {
                            app.table_state.select(Some(selected + 1));
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let selected = app.table_state.selected().unwrap_or(0);
                        if selected > 0 {
                            app.table_state.select(Some(selected - 1));
                        }
                    }
                    KeyCode::Char('g') => {
                        app.table_state.select(Some(0));
                    }
                    KeyCode::Char('G') => {
                        let count = app
                            .analysis
                            .as_ref()
                            .map(|a| a.functions.len())
                            .unwrap_or(0);
                        if count > 0 {
                            app.table_state.select(Some(count - 1));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

// ---------- Shared helpers ----------

fn confidence_color(confidence: f64) -> Color {
    if confidence >= 80.0 {
        BAD
    } else if confidence >= 60.0 {
        WARN
    } else {
        GOOD
    }
}

fn impact_color(impact: &str) -> Color {
    if impact.contains("High") {
        BAD
    } else if impact.contains("Medium") {
        WARN
    } else {
        GOOD
    }
}

fn outer_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ACCENT_DIM))
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
}

fn ui(f: &mut Frame, app: &mut App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.size());

    // Header: title + tabs combined into one strip for a tighter, more
    // polished top bar.
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(0)])
        .split(root[0]);

    let title = Paragraph::new(vec![Line::from(vec![
        Span::styled("⬢ ", Style::default().fg(ACCENT)),
        Span::styled(
            "Dead Code",
            Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
        ),
    ])])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(ACCENT_DIM)),
    )
    .alignment(Alignment::Center);
    f.render_widget(title, header_chunks[0]);

    let tab_titles: Vec<Line> = app
        .tabs
        .iter()
        .map(|t| Line::from(Span::styled(t.clone(), Style::default().fg(MUTED))))
        .collect();
    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(ACCENT_DIM)),
        )
        .select(app.selected_tab)
        .style(Style::default().fg(MUTED))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(ACCENT)
                .add_modifier(Modifier::BOLD),
        )
        .divider(symbols::DOT);
    f.render_widget(tabs, header_chunks[1]);

    // Main content
    match &app.analysis {
        Some(analysis) => match app.selected_tab {
            0 => render_summary(f, root[1], analysis),
            1 => render_charts(f, root[1], analysis),
            2 => render_list(f, root[1], analysis, &mut app.table_state),
            3 => render_by_file(f, root[1], analysis),
            4 => render_priority(f, root[1], analysis),
            _ => {}
        },
        None => {
            if app.loading {
                let loading = Paragraph::new("⏳ Loading...")
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(WARN));
                f.render_widget(loading, root[1]);
            } else if let Some(ref err) = app.error {
                let error = Paragraph::new(format!("✖ Error: {}", err))
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(BAD));
                f.render_widget(error, root[1]);
            }
        }
    }

    // Footer help bar
    let help = Paragraph::new(Line::from(vec![
        Span::styled(" Tab/←→ ", Style::default().fg(Color::Black).bg(ACCENT)),
        Span::raw(" switch tab   "),
        Span::styled(" ↑↓/jk ", Style::default().fg(Color::Black).bg(ACCENT)),
        Span::raw(" move   "),
        Span::styled(" g/G ", Style::default().fg(Color::Black).bg(ACCENT)),
        Span::raw(" top/bottom   "),
        Span::styled(" q ", Style::default().fg(Color::Black).bg(BAD)),
        Span::raw(" quit"),
    ]))
    .style(Style::default().fg(MUTED));
    f.render_widget(help, root[2]);
}

fn stat_card(title: &str, value: String, color: Color) -> Paragraph<'static> {
    Paragraph::new(vec![
        Line::from(Span::styled(
            value,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(title.to_string(), Style::default().fg(MUTED))),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(ACCENT_DIM)),
    )
}

fn render_summary(f: &mut Frame, area: Rect, analysis: &DeadCodeAnalysis) {
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
            Constraint::Min(0),
        ])
        .split(area);

    // Row 1: headline stat cards
    let cards = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
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

    // Row 3: dead-code proportion gauge, colored by severity
    let gauge = Gauge::default()
        .block(outer_block("Dead Code Share"))
        .gauge_style(
            Style::default()
                .fg(confidence_color(dead_pct))
                .bg(Color::Black),
        )
        .ratio((dead_pct / 100.0).clamp(0.0, 1.0))
        .label(format!("{:.1}% of functions are dead", dead_pct));
    f.render_widget(gauge, rows[2]);

    // Row 4: quick read-out / legend
    let legend = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("● ", Style::default().fg(BAD)),
            Span::raw("High confidence (≥80%) — safe to remove first"),
        ]),
        Line::from(vec![
            Span::styled("● ", Style::default().fg(WARN)),
            Span::raw("Medium confidence (60–79%) — review before removing"),
        ]),
        Line::from(vec![
            Span::styled("● ", Style::default().fg(GOOD)),
            Span::raw("Lower confidence (<60%) — double-check usage first"),
        ]),
    ])
    .block(outer_block("How to read this"))
    .style(Style::default().fg(TEXT));
    f.render_widget(legend, rows[3]);
}

fn render_charts(f: &mut Frame, area: Rect, analysis: &DeadCodeAnalysis) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // ---- Confidence distribution as a real BarChart ----
    let conf_order = ["Guaranteed", "VeryLikely", "Probably", "Uncertain", "Other"];
    let mut conf_counts: std::collections::HashMap<&str, u64> = std::collections::HashMap::new();
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

    // ---- Impact distribution as a real BarChart ----
    let impact_order = ["High", "Medium", "Low"];
    let mut impact_counts: std::collections::HashMap<&str, u64> = std::collections::HashMap::new();
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

fn render_list(f: &mut Frame, area: Rect, analysis: &DeadCodeAnalysis, state: &mut TableState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    let header_text = Paragraph::new(format!(
        "{} dead functions — sorted by removal order",
        analysis.functions.len()
    ))
    .style(Style::default().fg(MUTED));
    f.render_widget(header_text, chunks[0]);

    let rows: Vec<Row> = analysis
        .functions
        .iter()
        .map(|func| {
            let conf_color = confidence_color(func.confidence);
            let impact_color = impact_color(&func.impact);
            Row::new(vec![
                Cell::from(func.order.to_string()).style(Style::default().fg(MUTED)),
                Cell::from(func.name.clone()).style(Style::default().fg(TEXT)),
                Cell::from(format!("{:.1}%", func.confidence))
                    .style(Style::default().fg(conf_color).add_modifier(Modifier::BOLD)),
                Cell::from(func.level.clone()).style(Style::default().fg(conf_color)),
                Cell::from(func.impact.clone()).style(Style::default().fg(impact_color)),
                Cell::from(func.loc.to_string()).style(Style::default().fg(MUTED)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Percentage(38),
            Constraint::Length(11),
            Constraint::Length(12),
            Constraint::Length(24),
            Constraint::Length(6),
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

fn render_by_file(f: &mut Frame, area: Rect, analysis: &DeadCodeAnalysis) {
    let mut file_groups: std::collections::HashMap<String, Vec<&DeadFunction>> =
        std::collections::HashMap::new();
    for func in &analysis.functions {
        file_groups.entry(func.file.clone()).or_default().push(func);
    }

    let mut files: Vec<_> = file_groups.into_iter().collect();
    // Worst files (most dead functions) first.
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
        sorted_funcs.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));
        for func in sorted_funcs.iter().take(5) {
            let color = confidence_color(func.confidence);
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled("● ", Style::default().fg(color)),
                Span::styled(func.name.clone(), Style::default().fg(TEXT)),
                Span::styled(
                    format!("  {:.1}%", func.confidence),
                    Style::default().fg(color),
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
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_priority(f: &mut Frame, area: Rect, analysis: &DeadCodeAnalysis) {
    let mut lines: Vec<Line> = Vec::new();

    for func in analysis.functions.iter().take(20) {
        let color = confidence_color(func.confidence);
        lines.push(Line::from(vec![
            Span::styled(format!("{:>3}. ", func.order), Style::default().fg(MUTED)),
            Span::styled("● ", Style::default().fg(color)),
            Span::styled(
                func.name.clone(),
                Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  ({} · {:.1}%)", func.impact, func.confidence),
                Style::default().fg(color),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::raw("     "),
            Span::styled(
                func.file
                    .split('/')
                    .last()
                    .unwrap_or(&func.file)
                    .to_string(),
                Style::default().fg(MUTED),
            ),
            Span::styled(format!("  ·  {} LOC", func.loc), Style::default().fg(MUTED)),
        ]));
        lines.push(Line::from(""));
    }

    if analysis.functions.len() > 20 {
        lines.push(Line::from(Span::styled(
            format!("… and {} more functions", analysis.functions.len() - 20),
            Style::default().fg(MUTED),
        )));
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .block(outer_block("Priority Removal Order"))
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(paragraph, area);
}
