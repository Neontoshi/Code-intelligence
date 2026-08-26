// src/bin/dead_code_dashboard.rs
mod dashboard_ui;

use code_intelligence::analysis::dead_code::DeadCodeAnalysis;
use code_intelligence::analysis::explainability::ExplainabilityEngine;
use code_intelligence::analysis::git_analysis::GitAnalyzer;
use code_intelligence::config::{get_default_model, get_default_threshold};
use code_intelligence::error::{err, Result};
use code_intelligence::graph::GraphMetrics;
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

// Data Structures
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

// App State
struct App {
    analysis: Option<DeadCodeAnalysis>,
    table_state: TableState,
    selected_tab: usize,
    tabs: Vec<String>,
    loading: bool,
    loading_message: String,
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
    selected_evidence: Option<code_intelligence::analysis::explainability::VerdictExplanation>,
}

#[derive(Debug, Clone)]
pub enum Action {
    ConfirmDead(String),
    FalsePositive(String),
    Defer(String),
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
            loading_message: String::new(),
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
            selected_evidence: None,
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

    fn load_decisions(&self) -> Vec<DashboardDecision> {
        let path = self.project_path.join(".code-intelligence-decisions.json");
        if path.exists() {
            let data = std::fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    fn save_decision(&self, decision: &DashboardDecision) -> Result<()> {
        let path = self.project_path.join(".code-intelligence-decisions.json");
        let mut decisions = self.load_decisions();
        decisions.push(decision.clone());

        let json = serde_json::to_string_pretty(&decisions)
            .map_err(|e| err::internal(format!("Failed to serialize decisions: {}", e)))?;
        std::fs::write(&path, json)
            .map_err(|e| err::internal(format!("Failed to write decisions file: {}", e)))?;
        Ok(())
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
        if let Err(e) = self.save_decision(&decision_record) {
            self.error = Some(format!("Decision not saved: {}", e));
        }
        self.decisions.push(decision_record);
    }

    fn load_data(&mut self, path: PathBuf) {
        self.loading = true;
        self.loading_message = "Analyzing project...".to_string();
        self.current_commit = self.get_current_commit();
        self.decisions = self.load_decisions();

        use indicatif::{ProgressBar, ProgressStyle};
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::with_template("  {spinner:.cyan} {msg}")
                .expect("Invalid progress bar template"),
        );
        pb.enable_steady_tick(std::time::Duration::from_millis(80));

        let path_clone = path.clone();

        let result = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(async {
                // Use the shared AnalysisService
                use code_intelligence::analysis::service::{
                    AnalysisService, AnalysisServiceConfig,
                };

                let config = AnalysisServiceConfig {
                    model_path: get_default_model(),
                    threshold: get_default_threshold().or(Some(0.92)),
                    verbose: false,
                    debug: false,
                    cache: false,
                    cache_dir: None,
                    llm: false,
                    git: false,
                };

                let mut service = AnalysisService::new(config);

                // Run the analysis using the shared service
                let result = service
                    .analyze(&path_clone)
                    .await
                    .map_err(|e| e.to_string())?;

                // Extract the dead code analysis from the result
                Ok::<_, String>(result)
            })
        });

        let outcome = result.join().expect("Thread panicked during analysis");
        pb.finish_and_clear();

        match outcome {
            Ok(result) => {
                let provenance = result.verdicts.first().map(|v| v.provenance.clone());

                self.analysis_metadata = Some(AnalysisMetadata {
                    analysis_id: format!("run_{}", chrono::Utc::now().timestamp()),
                    model_version: provenance
                        .as_ref()
                        .and_then(|p| p.model_path.clone())
                        .unwrap_or_else(|| "none".to_string()),
                    feature_schema_version: provenance
                        .as_ref()
                        .map(|p| p.feature_schema_version)
                        .unwrap_or(1),
                    source_commit: self.current_commit.clone(),
                    analysis_timestamp: provenance
                        .as_ref()
                        .map(|p| p.analysis_timestamp)
                        .unwrap_or_else(|| chrono::Utc::now().timestamp()),
                    total_functions: result.call_graph.node_count(),
                    dead_candidates: result.dead_verdicts.len(),
                });

                self.analysis = Some(result.dead_code_analysis);
                self.table_state.select(Some(0));
                self.loading = false;
                self.loading_message = "Ready".to_string();
            }
            Err(e) => {
                self.error = Some(format!("Failed to analyze: {}", e));
                self.loading = false;
                self.loading_message = "Error".to_string();
            }
        }
    }

    fn show_evidence_for_function(
        &self,
        function_name: &str,
    ) -> Option<code_intelligence::analysis::explainability::VerdictExplanation> {
        if let Some(analysis) = &self.analysis {
            for func in &analysis.functions {
                if func.name == function_name {
                    let git_info = GitAnalyzer::analyze(&self.project_path).ok();

                    let func_node = code_intelligence::graph::call_graph::FunctionNode {
                        name: func.name.clone(),
                        full_path: func.full_path.clone(),
                        file: func.file.clone(),
                        line: func.line,
                        body_start_line: func.line,
                        body_end_line: func.line + func.impact.lines_of_code,
                        is_public: func.is_binary_only || false,
                        is_async: false,
                        params: Vec::new(),
                        returns: Vec::new(),
                        complexity: func.impact.complexity,
                        importance_score: func.score.score,
                        doc_comment: None,
                        writes_to: Vec::new(),
                        reads_from: Vec::new(),
                        errors: Vec::new(),
                        fan_in: 0,
                        fan_out: 0,
                        is_cycle: false,
                        depth: 0,
                        layer: String::new(),
                        trait_impl: None,
                        decorators: Vec::new(),
                        is_test: false,
                        is_trait_method: false,
                        is_trait_default: false,
                    };

                    use code_intelligence::analysis::training_data::TrainingLabel;
                    use code_intelligence::analysis::verdict_source::label_source::VerdictState;
                    use code_intelligence::analysis::verdict_source::state::DeletionRecommendation;
                    use code_intelligence::analysis::verdict_source::{
                        Signal, SignalDirection, Verdict,
                    };

                    let verdict = Verdict {
                        function_name: func.name.clone(),
                        full_path: func.full_path.clone(),
                        label: TrainingLabel::Dead,
                        state: VerdictState::DefinitelyDead,
                        confidence: func.score.score,
                        dead_probability: Some(func.score.score),
                        signals: func
                            .score
                            .factors
                            .iter()
                            .map(|f| Signal {
                                name: f.name.clone(),
                                value: f.contribution.abs(),
                                direction: if f.contribution > 0.0 {
                                    SignalDirection::SupportsDead
                                } else {
                                    SignalDirection::SupportsAlive
                                },
                                weight: f.weight,
                                explanation: f.explanation.clone(),
                            })
                            .collect(),
                        ml_probability: Some(func.score.score),
                        static_score: Some(func.score.score),
                        explanation: format!(
                            "Dead function with {:.1}% confidence",
                            func.score.score * 100.0
                        ),
                        evidence_sources: Vec::new(),
                        verified: false,
                        verified_by: None,
                        provenance:
                            code_intelligence::analysis::verdict_source::state::VerdictProvenance {
                                analysis_version: env!("CARGO_PKG_VERSION").to_string(),
                                model_version: None,
                                commit_sha: Some(self.current_commit.clone()),
                                feature_schema_version: 1,
                                analysis_timestamp: chrono::Utc::now().timestamp(),
                                analysis_duration_secs: None,
                                stages_used: vec!["dashboard_analysis".to_string()],
                                ml_enabled: false,
                                static_enabled: true,
                                model_path: None,
                            },
                        evidence_conflicts: Vec::new(),
                        deletion_recommendation: DeletionRecommendation::NeedsReview,
                        label_provenance: None,
                    };

                    let git_info_ref = git_info
                        .as_ref()
                        .and_then(|g| g.files.get(&std::path::PathBuf::from(&func.file)));
                    return Some(ExplainabilityEngine::generate_explanation(
                        &verdict,
                        &func_node,
                        git_info_ref,
                    ));
                }
            }
        }
        None
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let path = if args.len() >= 2 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from(".")
    };

    enable_raw_mode().map_err(|e| err::internal(format!("Failed to enable raw mode: {}", e)))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| err::internal(format!("Failed to setup terminal: {}", e)))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| err::internal(format!("Failed to create terminal: {}", e)))?;

    let mut app = App::new(path.clone());
    app.load_data(path);
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode().map_err(|e| err::internal(format!("Failed to disable raw mode: {}", e)))?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .map_err(|e| err::internal(format!("Failed to restore terminal: {}", e)))?;
    terminal
        .show_cursor()
        .map_err(|e| err::internal(format!("Failed to show cursor: {}", e)))?;

    if let Err(err) = res {
        println!("{:?}", err);
    }
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    terminal.clear()?;
    loop {
        terminal.draw(|f| ui(f, app))?;
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Check for quit first
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
                if handle_dialogs(app, key.code) {
                    continue;
                }
                handle_navigation(app, key.code)?;
                handle_actions(app, key.code);
            }
        }
    }
    Ok(())
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

fn handle_navigation(app: &mut App, key: KeyCode) -> io::Result<()> {
    if matches!(
        key,
        KeyCode::Tab | KeyCode::Right | KeyCode::Left | KeyCode::Char('l') | KeyCode::Char('h')
    ) {
        app.selected_evidence = None;
    }

    match key {
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
        _ => {}
    }
    Ok(())
}

fn handle_actions(app: &mut App, key: KeyCode) -> bool {
    match key {
        KeyCode::Enter => {
            if let Some(selected) = app.table_state.selected() {
                if let Some(analysis) = &app.analysis {
                    if let Some(func) = analysis.functions.get(selected) {
                        // ⭐ Show evidence for the selected function
                        if let Some(evidence) = app.show_evidence_for_function(&func.name) {
                            app.selected_evidence = Some(evidence);
                        }
                        return true;
                    }
                }
            }
            if app.selected_evidence.is_some() {
                app.selected_evidence = None;
                return true;
            }
        }
        KeyCode::Esc => {
            if app.selected_evidence.is_some() {
                app.selected_evidence = None;
                return true;
            }
        }
        _ => {}
    }
    false
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

    match &app.analysis {
        Some(analysis) => match app.selected_tab {
            0 => dashboard_ui::render_summary(f, root[1], analysis, app),
            1 => dashboard_ui::render_charts(f, root[1], analysis),
            2 => dashboard_ui::render_list(
                f,
                root[1],
                analysis,
                &mut app.table_state,
                app.selected_evidence.as_ref(),
            ),
            3 => dashboard_ui::render_by_file(f, root[1], analysis),
            4 => dashboard_ui::render_priority(f, root[1], analysis),
            5 => dashboard_ui::render_history(f, root[1], &app.decisions),
            _ => {}
        },
        None => {
            if app.loading {
                f.render_widget(
                    Paragraph::new(format!("⏳ {}", app.loading_message))
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
