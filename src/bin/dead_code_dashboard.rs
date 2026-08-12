// src/bin/dead_code_dashboard.rs

mod dashboard_ui;

use code_intelligence::analysis::dead_code::DeadCodeDetector;
use code_intelligence::Pipeline;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dashboard_ui::styles::{ACCENT, ACCENT_DIM, BAD, MUTED, TEXT, WARN};
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::TableState;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    symbols,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Tabs},
    Frame, Terminal,
};
use serde::{Deserialize, Serialize};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

// ============================================================================
// Data Structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DecisionType {
    ConfirmedDead,
    ConfirmedAlive,
    FalsePositive,
    NeedsInvestigation,
    Deferred,
    Stale,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub enum CandidateStatus {
    Pending,
    ConfirmedDead,
    ConfirmedAlive,
    FalsePositive,
    Deferred,
    Stale,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardDecision {
    pub candidate_id: String,
    pub decision: DecisionType,
    pub reason: Option<String>,
    pub user: String,
    pub timestamp: i64,
    pub analysis_id: String,
    pub model_version: String,
    pub source_commit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    pub analysis_id: String,
    pub model_version: String,
    pub feature_schema_version: u32,
    pub source_commit: String,
    pub analysis_timestamp: i64,
    pub total_functions: usize,
    pub dead_candidates: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadFunctionExtended {
    pub id: String,
    pub analysis_id: String,
    pub function_name: String,
    pub file: String,
    pub line: usize,
    pub confidence: f64,
    pub level: String,
    pub impact: String,
    pub loc: usize,
    pub order: usize,
    pub model_version: String,
    pub source_commit: String,
    pub evidence: Vec<String>,
    pub counter_evidence: Vec<String>,
    pub status: CandidateStatus,
}

#[derive(Debug, Clone)]
pub enum Action {
    ConfirmDead(String),
    FalsePositive(String),
    Defer(String),
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
    functions: Vec<DeadFunctionExtended>,
}

// ============================================================================
// App State
// ============================================================================

struct App {
    analysis: Option<DeadCodeAnalysis>,
    table_state: TableState,
    selected_tab: usize,
    tabs: Vec<String>,
    loading: bool,
    error: Option<String>,
    project_path: PathBuf,
    decisions: Vec<DashboardDecision>,
    analysis_metadata: Option<AnalysisMetadata>,
    show_confirmation: bool,
    show_reason_dialog: bool,
    pending_action: Option<Action>,
    pending_id: Option<String>,
    reason_input: String,
    current_commit: String,
}

impl App {
    fn new(path: PathBuf) -> Self {
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
                "History".to_string(),
            ],
            loading: true,
            error: None,
            project_path: path,
            decisions: Vec::new(),
            analysis_metadata: None,
            show_confirmation: false,
            show_reason_dialog: false,
            pending_action: None,
            pending_id: None,
            reason_input: String::new(),
            current_commit: String::new(),
        }
    }

    fn get_current_commit(&self) -> String {
        use std::process::Command;
        let output = Command::new("git")
            .current_dir(&self.project_path)
            .args(["rev-parse", "HEAD"])
            .output();
        match output {
            Ok(out) if out.status.success() => {
                String::from_utf8_lossy(&out.stdout).trim().to_string()
            }
            _ => "unknown".to_string(),
        }
    }

    fn load_analysis_metadata(&self) -> Option<AnalysisMetadata> {
        let path = self.project_path.join(".code-intelligence-metadata.json");
        if path.exists() {
            let data = std::fs::read_to_string(&path).ok()?;
            serde_json::from_str(&data).ok()
        } else {
            None
        }
    }

    fn load_decisions(&self) -> Vec<DashboardDecision> {
        let path = self.project_path.join(".code-intelligence-decisions.json");
        if path.exists() {
            let data = std::fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    fn save_decision(&self, decision: &DashboardDecision) {
        let path = self.project_path.join(".code-intelligence-decisions.json");
        let mut decisions = self.load_decisions();
        decisions.push(decision.clone());
        let _ = std::fs::write(&path, serde_json::to_string_pretty(&decisions).unwrap());
    }

    fn get_analysis_id(&self) -> String {
        self.analysis_metadata
            .as_ref()
            .map(|m| m.analysis_id.clone())
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn get_model_version(&self) -> String {
        self.analysis_metadata
            .as_ref()
            .map(|m| m.model_version.clone())
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn get_source_commit(&self) -> String {
        self.analysis_metadata
            .as_ref()
            .map(|m| m.source_commit.clone())
            .unwrap_or_else(|| self.current_commit.clone())
    }

    fn record_decision(
        &mut self,
        candidate_id: &str,
        decision: DecisionType,
        reason: Option<String>,
    ) {
        let decision_record = DashboardDecision {
            candidate_id: candidate_id.to_string(),
            decision: decision.clone(),
            reason,
            user: std::env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
            timestamp: chrono::Utc::now().timestamp(),
            analysis_id: self.get_analysis_id(),
            model_version: self.get_model_version(),
            source_commit: self.get_source_commit(),
        };
        self.save_decision(&decision_record);
        self.decisions.push(decision_record);
        if let Some(analysis) = &mut self.analysis {
            if let Some(candidate) = analysis.functions.iter_mut().find(|c| c.id == candidate_id) {
                candidate.status = match decision {
                    DecisionType::ConfirmedDead => CandidateStatus::ConfirmedDead,
                    DecisionType::ConfirmedAlive => CandidateStatus::ConfirmedAlive,
                    DecisionType::FalsePositive => CandidateStatus::FalsePositive,
                    DecisionType::Deferred => CandidateStatus::Deferred,
                    _ => candidate.status.clone(),
                };
            }
        }
    }

    fn load_data(&mut self, path: PathBuf) {
        self.loading = true;
        self.current_commit = self.get_current_commit();
        self.analysis_metadata = self.load_analysis_metadata();
        self.decisions = self.load_decisions();

        // 🛑 REMOVED THE RAW PRINTLN! STATEMENTS SO THE TUI ISN'T CORRUPTED

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
                let is_stale = self
                    .analysis_metadata
                    .as_ref()
                    .map(|m| m.source_commit != self.current_commit)
                    .unwrap_or(false);

                let mut functions: Vec<DeadFunctionExtended> = analysis
                    .functions
                    .iter()
                    .map(|f| {
                        let candidate_id = format!(
                            "{}::{}::{}",
                            self.get_analysis_id(),
                            f.file.replace('/', "_"),
                            f.name
                        );
                        let status = if is_stale {
                            CandidateStatus::Stale
                        } else if let Some(decision) = self
                            .decisions
                            .iter()
                            .find(|d| d.candidate_id == candidate_id)
                        {
                            match decision.decision {
                                DecisionType::ConfirmedDead => CandidateStatus::ConfirmedDead,
                                DecisionType::ConfirmedAlive => CandidateStatus::ConfirmedAlive,
                                DecisionType::FalsePositive => CandidateStatus::FalsePositive,
                                DecisionType::Deferred => CandidateStatus::Deferred,
                                _ => CandidateStatus::Pending,
                            }
                        } else {
                            CandidateStatus::Pending
                        };

                        DeadFunctionExtended {
                            id: candidate_id,
                            analysis_id: self.get_analysis_id(),
                            function_name: f.name.clone(),
                            file: f.file.clone(),
                            line: f.line,
                            confidence: f.score.score * 100.0,
                            level: format!("{:?}", f.score.level),
                            impact: f.impact.estimated_removal_impact.clone(),
                            loc: f.impact.lines_of_code,
                            order: f.removal_order,
                            model_version: self.get_model_version(),
                            source_commit: self.get_source_commit(),
                            evidence: f
                                .score
                                .factors
                                .iter()
                                .filter(|s| s.contribution > 0.0)
                                .map(|s| s.explanation.clone())
                                .collect(),
                            counter_evidence: f
                                .score
                                .factors
                                .iter()
                                .filter(|s| s.contribution < 0.0)
                                .map(|s| s.explanation.clone())
                                .collect(),
                            status,
                        }
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

    // No println here!
    // No indicatif spinner here!

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(path.clone());
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

// ============================================================================
// UI Rendering (thin wrapper that calls dashboard_ui)
// ============================================================================

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if handle_dialogs(app, key.code) {
                    continue;
                }
                if handle_navigation(app, key.code)? {
                    continue;
                }
                if handle_actions(app, key.code) {
                    continue;
                }
            }
        }
    }
}

fn handle_dialogs(app: &mut App, key: KeyCode) -> bool {
    if app.show_confirmation {
        match key {
            KeyCode::Char('y') | KeyCode::Enter => {
                if let Some(action) = app.pending_action.take() {
                    match action {
                        Action::ConfirmDead(id) => {
                            app.record_decision(&id, DecisionType::ConfirmedDead, None)
                        }
                        Action::FalsePositive(id) => {
                            app.show_reason_dialog = true;
                            app.pending_id = Some(id);
                        }
                        Action::Defer(id) => app.record_decision(&id, DecisionType::Deferred, None),
                    }
                    app.show_confirmation = false;
                }
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                app.show_confirmation = false;
                app.pending_action = None;
            }
            _ => {}
        }
        return true;
    }

    if app.show_reason_dialog {
        match key {
            KeyCode::Char(c) => app.reason_input.push(c),
            KeyCode::Backspace => {
                app.reason_input.pop();
            }
            KeyCode::Enter => {
                if let Some(id) = app.pending_id.take() {
                    let reason = if app.reason_input.is_empty() {
                        None
                    } else {
                        Some(app.reason_input.clone())
                    };
                    app.record_decision(&id, DecisionType::FalsePositive, reason);
                    app.reason_input.clear();
                }
                app.show_reason_dialog = false;
            }
            KeyCode::Esc => {
                app.show_reason_dialog = false;
                app.pending_id = None;
                app.reason_input.clear();
            }
            _ => {}
        }
        return true;
    }
    false
}

fn handle_navigation(app: &mut App, key: KeyCode) -> io::Result<bool> {
    match key {
        KeyCode::Char('q') | KeyCode::Esc => return Ok(true), // Quit
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
        KeyCode::Char('g') => app.table_state.select(Some(0)),
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
        _ => return Ok(false),
    }
    Ok(false)
}

fn handle_actions(app: &mut App, key: KeyCode) -> bool {
    match key {
        KeyCode::Char('d') => {
            if let Some(selected) = app.table_state.selected() {
                if let Some(candidate) = app
                    .analysis
                    .as_ref()
                    .and_then(|a| a.functions.get(selected))
                {
                    if candidate.status != CandidateStatus::Stale {
                        app.show_confirmation = true;
                        app.pending_action = Some(Action::ConfirmDead(candidate.id.clone()));
                    }
                }
            }
        }
        KeyCode::Char('f') => {
            if let Some(selected) = app.table_state.selected() {
                if let Some(candidate) = app
                    .analysis
                    .as_ref()
                    .and_then(|a| a.functions.get(selected))
                {
                    if candidate.status != CandidateStatus::Stale {
                        app.show_confirmation = true;
                        app.pending_action = Some(Action::FalsePositive(candidate.id.clone()));
                    }
                }
            }
        }
        KeyCode::Char('s') => {
            if let Some(selected) = app.table_state.selected() {
                if let Some(candidate) = app
                    .analysis
                    .as_ref()
                    .and_then(|a| a.functions.get(selected))
                {
                    if candidate.status != CandidateStatus::Stale {
                        app.show_confirmation = true;
                        app.pending_action = Some(Action::Defer(candidate.id.clone()));
                    }
                }
            }
        }
        _ => return false,
    }
    true
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

    // Header
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(0)])
        .split(root[0]);

    let title = Paragraph::new(vec![Line::from(vec![
        Span::styled("⬢ ", Style::default().fg(ACCENT)),
        Span::styled(
            "Dead Code Dashboard",
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

    // Main content - delegate to dashboard_ui
    match &app.analysis {
        Some(analysis) => match app.selected_tab {
            0 => dashboard_ui::render_summary(f, root[1], analysis, app),
            1 => dashboard_ui::render_charts(f, root[1], analysis),
            2 => dashboard_ui::render_list(f, root[1], analysis, &mut app.table_state),
            3 => dashboard_ui::render_by_file(f, root[1], analysis),
            4 => dashboard_ui::render_priority(f, root[1], analysis),
            5 => dashboard_ui::render_history(f, root[1], &app.decisions),
            _ => {}
        },
        None => {
            if app.loading {
                f.render_widget(
                    Paragraph::new("⏳ Loading...")
                        .alignment(Alignment::Center)
                        .style(Style::default().fg(WARN)),
                    root[1],
                );
            } else if let Some(ref err) = app.error {
                f.render_widget(
                    Paragraph::new(format!("✖ Error: {}", err))
                        .alignment(Alignment::Center)
                        .style(Style::default().fg(BAD)),
                    root[1],
                );
            }
        }
    }

    // Dialogs
    if app.show_confirmation {
        dashboard_ui::render_confirmation_dialog(f, f.size(), &app.pending_action);
    }
    if app.show_reason_dialog {
        dashboard_ui::render_reason_dialog(f, f.size(), &app.reason_input);
    }

    // Footer
    dashboard_ui::render_help(f, root[2], app.show_confirmation || app.show_reason_dialog);
}
