// src/bin/ci.rs

use clap::{Parser, Subcommand};
use code_intelligence::{
    analysis::{
        outcomes::{OutcomeTracker, VerdictOutcome},
        service::{AnalysisService, AnalysisServiceConfig},
    },
    engine::Pipeline,
    error::{err, Result},
    graph::GraphMetrics,
    ml::{classifier::DeadCodeClassifier, duplicate_classifier::DuplicateClassifier},
    output::{InteractiveGraph, OverviewGraph},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// Types & Configuration

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub projects: std::collections::HashMap<String, ProjectConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defaults {
    pub model: Option<String>,
    pub duplicate_model: Option<String>,
    pub threshold: Option<f64>,
    pub verbose: bool,
    pub llm_provider: Option<String>,
    pub llm_model: Option<String>,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            model: None,
            duplicate_model: None,
            threshold: None,
            verbose: false,
            llm_provider: Some("ollama".to_string()),
            llm_model: Some("phi:2.7b".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub path: String,
    pub project_type: Option<String>,
    pub threshold: Option<f64>,
    pub last_analyzed: Option<String>,
    pub dead_count: Option<usize>,
}

// CLI Arguments

#[derive(Parser, Debug)]
#[command(
    name = "ci",
    author = "Code Intelligence Team",
    version = "0.2.0",
    about = "Code Intelligence - Complete dead code detection toolkit"
)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Analyze a project for dead code
    Analyze {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Use specific threshold (overrides config)
        #[arg(long)]
        threshold: Option<f64>,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
        /// Enable LLM analysis
        #[arg(long)]
        llm: bool,
        /// Enable Git analysis
        #[arg(long)]
        git: bool,
        /// Enable disk cache for faster repeat runs
        #[arg(long)]
        cache: bool,
        /// Cache directory
        #[arg(long)]
        cache_dir: Option<PathBuf>,
        /// Model file path
        #[arg(long)]
        model: Option<PathBuf>,
    },

    /// Find duplicate code in a project
    Dedup {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Similarity threshold (0.0 - 1.0)
        #[arg(long, default_value = "0.85")]
        threshold: f64,
        /// Use ML model for duplicate detection
        #[arg(long)]
        ml: bool,
        /// Duplicate model file path
        #[arg(long)]
        duplicate_model: Option<PathBuf>,
    },

    /// Generate call graph visualization (HTML)
    Graph {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Output file
        #[arg(long)]
        output: Option<PathBuf>,
        /// Mode: interactive or overview
        #[arg(long, default_value = "interactive")]
        mode: String,
    },

    /// List dead functions found in a project
    List {
        /// Path to list (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Show all (including removed and kept)
        #[arg(long)]
        all: bool,
    },

    /// Mark a function as removed (by name)
    Remove {
        /// Function name (partial match supported)
        name: String,
        /// Git commit hash (optional)
        #[arg(long)]
        commit: Option<String>,
        /// Path (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Mark a function as false positive (kept)
    Keep {
        /// Function name (partial match supported)
        name: String,
        /// Reason for keeping
        reason: String,
        /// Path (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Update outcome by verdict ID
    Update {
        /// Project directory
        #[arg(default_value = ".")]
        path: PathBuf,
        /// The verdict ID
        id: String,
        #[command(subcommand)]
        action: UpdateAction,
    },

    /// Show outcome statistics
    Stats {
        /// Path (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Show detailed breakdown
        #[arg(long)]
        detailed: bool,
    },

    /// Generate report for a project
    Report {
        /// Path (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Output format: markdown, json, html, full
        #[arg(long, default_value = "markdown")]
        format: String,
        /// Output file
        #[arg(long)]
        output: Option<PathBuf>,
        /// Include LLM analysis
        #[arg(long)]
        llm: bool,
    },

    /// Train the ML model
    Train {
        /// Training data path
        #[arg(long, default_value = "data/train.json")]
        data: PathBuf,
        /// Validation data path
        #[arg(long, default_value = "data/val.json")]
        val_data: Option<PathBuf>,
        /// Output model path
        #[arg(long, default_value = "model.bin")]
        output: PathBuf,
        /// Target precision (0.0 - 1.0)
        #[arg(long, default_value = "0.95")]
        precision: f64,
    },

    /// Train duplicate detection model
    TrainDuplicate {
        /// Project path or JSON file
        input: PathBuf,
        /// Output model path
        #[arg(long, default_value = "duplicate_model.bin")]
        output: PathBuf,
    },

    /// Calibrate a trained model
    Calibrate {
        /// Model file path
        #[arg(long, default_value = "model.bin")]
        model: PathBuf,
        /// Validation data
        #[arg(long, default_value = "data/val.json")]
        data: PathBuf,
        /// Output model path
        #[arg(long, default_value = "model_calibrated.bin")]
        output: PathBuf,
        /// Calibration method: temperature, histogram, none
        #[arg(long, default_value = "temperature")]
        method: String,
    },

    /// Tune confidence threshold
    Tune {
        /// Model file path
        #[arg(long, default_value = "model.bin")]
        model: PathBuf,
        /// Validation data
        #[arg(long, default_value = "data/val.json")]
        data: PathBuf,
        /// Target precision (0.0 - 1.0)
        #[arg(long, default_value = "0.99")]
        precision: f64,
    },

    /// Export training data from a project
    Export {
        /// Path to analyze
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Output file
        #[arg(long, default_value = "training_data.json")]
        output: PathBuf,
    },

    /// Merge training data files
    Merge {
        /// Input files (glob pattern)
        #[arg(default_value = "training_data/*.json")]
        input: String,
        /// Output file
        #[arg(long, default_value = "combined_training.json")]
        output: PathBuf,
        /// Deduplicate examples
        #[arg(long)]
        dedup: bool,
    },

    /// Collect training data from repositories
    Collect {
        /// Repository URLs (space-separated)
        repos: Vec<String>,
        /// Output directory
        #[arg(long, default_value = "training_data")]
        output: PathBuf,
        /// Max repos to process
        #[arg(long, default_value = "50")]
        max_repos: usize,
    },

    /// Open interactive dashboard
    Dashboard {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Model file (optional)
        #[arg(long)]
        model: Option<PathBuf>,
    },

    /// Analyze code-intelligence itself (self-analysis)
    SelfAnalyze {
        /// Output format: markdown, json, full
        #[arg(long, default_value = "markdown")]
        format: String,
        /// Output file
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// CI mode - run analysis with exit code
    Ci {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Fail if dead code count exceeds this
        #[arg(long)]
        max_dead: Option<usize>,
        /// Fail if dead code ratio exceeds this (0.0-1.0)
        #[arg(long)]
        max_ratio: Option<f64>,
        /// Output format: json, markdown, summary
        #[arg(long, default_value = "json")]
        format: String,
        /// Output file
        #[arg(long)]
        output: Option<PathBuf>,
        /// Fail on any dead code
        #[arg(long, default_value = "true")]
        fail_on_dead: bool,
        /// Threshold for dead code confidence
        #[arg(long, default_value = "0.80")]
        threshold: f64,
        /// Conservative mode (higher threshold)
        #[arg(long)]
        conservative: bool,
    },

    /// Configure global settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Export dashboard decisions as training data
    ExportFeedback {
        /// Path to the project
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Output file for training data
        #[arg(short, long, default_value = "feedback_training.json")]
        output: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigAction {
    Set { key: String, value: String },
    Get { key: String },
    List,
}

#[derive(Subcommand, Debug)]
enum UpdateAction {
    Removed {
        #[arg(long)]
        commit: Option<String>,
    },
    FalsePositive {
        reason: String,
    },
}

// Main Entry Point

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        // ====================================================================
        // Core Analysis Commands
        // ====================================================================
        Commands::Analyze {
            path,
            threshold,
            verbose,
            llm,
            git,
            cache,
            cache_dir,
            model,
        } => {
            let project_path = resolve_path(&path)?;
            run_analyze(
                project_path,
                threshold,
                verbose,
                llm,
                git,
                cache,
                cache_dir,
                model,
            )
            .await?;
        }

        Commands::Dedup {
            path,
            threshold,
            ml,
            duplicate_model,
        } => {
            let project_path = resolve_path(&path)?;
            run_dedup(&project_path, threshold, ml, duplicate_model).await?;
        }

        Commands::Graph { path, output, mode } => {
            let project_path = resolve_path(&path)?;
            run_graph(&project_path, output, &mode).await?;
        }

        // ====================================================================
        // Outcome Management
        // ====================================================================
        Commands::List { path, all } => {
            let project_path = resolve_path(&path)?;
            run_list(&project_path, all)?;
        }

        Commands::Remove { name, commit, path } => {
            let project_path = resolve_path(&path)?;
            run_remove(&project_path, &name, commit)?;
        }

        Commands::Keep { name, reason, path } => {
            let project_path = resolve_path(&path)?;
            run_keep(&project_path, &name, &reason)?;
        }

        Commands::Update { path, id, action } => {
            let project_path = resolve_path(&path)?;
            run_update(&project_path, &id, action)?;
        }

        Commands::Stats { path, detailed } => {
            let project_path = resolve_path(&path)?;
            run_stats(&project_path, detailed)?;
        }

        Commands::Report {
            path,
            format,
            output,
            llm,
        } => {
            let project_path = resolve_path(&path)?;
            run_report(project_path, &format, output, llm).await?;
        }

        // ====================================================================
        // Training & Model Management
        // ====================================================================
        Commands::Train {
            data,
            val_data,
            output,
            precision,
        } => {
            run_train(&data, val_data.as_deref(), &output, precision)?;
        }

        Commands::TrainDuplicate { input, output } => {
            run_train_duplicate(&input, &output)?;
        }

        Commands::Calibrate {
            model,
            data,
            output,
            method,
        } => {
            run_calibrate(&model, &data, &output, &method)?;
        }

        Commands::Tune {
            model,
            data,
            precision,
        } => {
            run_tune(&model, &data, precision)?;
        }

        // ====================================================================
        // Data Management
        // ====================================================================
        Commands::Export { path, output } => {
            let project_path = resolve_path(&path)?;
            run_export(&project_path, &output).await?;
        }

        Commands::Merge {
            input,
            output,
            dedup,
        } => {
            run_merge(&input, &output, dedup)?;
        }

        Commands::Collect {
            repos,
            output,
            max_repos,
        } => {
            run_collect(&repos, &output, max_repos).await?;
        }

        // ====================================================================
        // Special Commands
        // ====================================================================
        Commands::Dashboard { path, model } => {
            let project_path = resolve_path(&path)?;
            run_dashboard(&project_path, model).await?;
        }

        Commands::SelfAnalyze { format, output } => {
            run_self_analyze(&format, output).await?;
        }

        Commands::Ci {
            path,
            max_dead,
            max_ratio,
            format,
            output,
            fail_on_dead,
            threshold,
            conservative,
            ..
        } => {
            let project_path = resolve_path(&path)?;
            run_ci(
                project_path,
                max_dead,
                max_ratio,
                &format,
                output,
                fail_on_dead,
                threshold,
                conservative,
            )
            .await?;
        }

        Commands::Config { action } => {
            run_config(action)?;
        }

        Commands::ExportFeedback { path, output } => {
            let project_path = resolve_path(&path)?;
            run_export_feedback(&project_path, &output)?;
        }
    }

    Ok(())
}

// Command Implementations - Using Library Directly

/// Run analysis on a project
async fn run_analyze(
    path: PathBuf,
    threshold: Option<f64>,
    verbose: bool,
    llm: bool,
    git: bool,
    cache: bool,
    cache_dir: Option<PathBuf>,
    model_path: Option<PathBuf>,
) -> Result<()> {
    println!("🔍 Analyzing project: {:?}", path);

    let project_type = detect_project_type(&path);
    if let Some(pt) = &project_type {
        println!("📊 Detected project type: {}", pt);
    }

    // Get model path
    let model_path = model_path
        .or_else(|| get_default_model().map(PathBuf::from))
        .ok_or_else(|| err::config("No model configured. Run: ci config set model <path>"))?;

    if !model_path.exists() {
        return Err(err::model(format!(
            "Model file not found: {:?}",
            model_path
        )));
    }

    // Build service config
    let config = AnalysisServiceConfig {
        model_path: Some(model_path),
        threshold,
        verbose,
        debug: verbose,
        cache,
        cache_dir,
        llm,
        git,
    };

    let mut service = AnalysisService::new(config);
    service.load_model()?;

    let result = service.analyze(&path).await?;

    // Print results
    println!("\n📊 Analysis Results:");
    println!("   Total functions: {}", result.call_graph.node_count());
    println!("   Dead functions: {}", result.dead_verdicts.len());
    println!("   Alive functions: {}", result.alive_verdicts.len());
    println!("   Unknown: {}", result.unknown_verdicts.len());
    println!("   Effective threshold: {:.2}", result.effective_threshold);

    if verbose && !result.dead_verdicts.is_empty() {
        println!("\n🔍 Top Dead Functions:");
        for verdict in result.dead_verdicts.iter().take(10) {
            println!(
                "   - {} ({:.1}%)",
                verdict.function_name,
                verdict.confidence * 100.0
            );
        }
        if result.dead_verdicts.len() > 10 {
            println!("   ... and {} more", result.dead_verdicts.len() - 10);
        }
    }

    // Save project config
    let project_config = ProjectConfig {
        path: path.to_string_lossy().to_string(),
        project_type,
        threshold,
        last_analyzed: Some(chrono::Local::now().to_string()),
        dead_count: Some(result.dead_verdicts.len()),
    };
    let _ = save_project_config(&path, project_config);

    Ok(())
}

/// Run duplicate detection
async fn run_dedup(
    path: &Path,
    threshold: f64,
    ml: bool,
    duplicate_model: Option<PathBuf>,
) -> Result<()> {
    println!("🔍 Finding duplicates in: {:?}", path);
    println!("📊 Similarity threshold: {:.2}", threshold);

    // Get duplicate model
    let model = if ml {
        let model_path = duplicate_model
            .or_else(get_default_duplicate_model)
            .ok_or_else(|| err::config("No duplicate model configured"))?;

        if !model_path.exists() {
            return Err(err::model(format!(
                "Duplicate model not found: {:?}",
                model_path
            )));
        }

        Some(DuplicateClassifier::load(&*model_path.to_string_lossy())?)
    } else {
        None
    };

    // Run pipeline
    let mut pipeline = Pipeline::new();
    let analysis = pipeline.process_project(path).await?;

    // Find duplicates
    use code_intelligence::optimize::dedup::Deduplicator;
    let mut dedup = if let Some(model) = model {
        Deduplicator::new_with_ml(Some(model))
    } else {
        Deduplicator::new()
    };
    dedup = dedup.with_threshold(threshold);

    let result = dedup.find_duplicates(&analysis.call_graph, &analysis.files);

    println!("\n📊 Deduplication Results:");
    println!("   Duplicate groups: {}", result.duplicate_groups.len());
    println!("   Total token savings: ~{}", result.total_saved_tokens);
    println!(
        "   Confidence: {:.1}%",
        result.accuracy_metrics.confidence_score * 100.0
    );

    if !result.duplicate_groups.is_empty() {
        println!("\n🔍 Duplicate Groups:");
        for (i, group) in result.duplicate_groups.iter().enumerate() {
            println!(
                "   {}: {} functions, similarity: {:.1}%, type: {:?}",
                i + 1,
                group.functions.len(),
                group.similarity_score * 100.0,
                group.duplicate_type
            );
        }
    }

    Ok(())
}

/// Generate call graph visualization
async fn run_graph(path: &Path, output: Option<PathBuf>, mode: &str) -> Result<()> {
    let output_file = output.unwrap_or_else(|| {
        if mode == "overview" {
            PathBuf::from("call_graph_overview.html")
        } else {
            PathBuf::from("call_graph.html")
        }
    });

    println!("📊 Generating {} call graph for: {:?}", mode, path);

    let mut pipeline = Pipeline::new();
    let analysis = pipeline.process_project(path).await?;

    let project_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let html = if mode == "overview" {
        OverviewGraph::generate(&analysis.call_graph, &project_name)
    } else {
        InteractiveGraph::generate(&analysis.call_graph, &analysis.files, &project_name)
    };

    std::fs::write(&output_file, html)?;

    println!("✅ HTML saved to: {:?}", output_file);
    println!("   Functions: {}", analysis.call_graph.node_count());
    println!("   Edges: {}", analysis.call_graph.edge_count());

    Ok(())
}

// Outcome Management Commands

fn run_list(path: &Path, all: bool) -> Result<()> {
    let tracker = OutcomeTracker::new(path);
    let verdicts = tracker.get_verdicts();

    if verdicts.is_empty() {
        println!("No tracked verdicts. Run `ci analyze` first.");
        return Ok(());
    }

    let filtered: Vec<_> = if all {
        verdicts.iter().collect()
    } else {
        verdicts
            .iter()
            .filter(|v| matches!(v.outcome, VerdictOutcome::Pending))
            .collect()
    };

    if filtered.is_empty() {
        if all {
            println!("No tracked verdicts.");
        } else {
            println!("✅ No pending verdicts!");
        }
        return Ok(());
    }

    println!(
        "\n📋 {} Dead Functions:",
        if all { "All" } else { "Pending" }
    );
    println!("");
    println!("| # | Function | Confidence | File | Status |");
    println!("|---|----------|------------|------|--------|");

    for (i, v) in filtered.iter().enumerate() {
        let file = v.file.split('/').last().unwrap_or(&v.file);
        println!(
            "| {} | {} | {:.1}% | {} | {:?} |",
            i + 1,
            v.function_name,
            v.confidence * 100.0,
            file,
            v.outcome
        );
    }

    Ok(())
}

fn run_remove(path: &Path, name: &str, commit: Option<String>) -> Result<()> {
    let mut tracker = OutcomeTracker::new(path);
    // Clone the needed data before mutable borrow
    let target_id = tracker
        .get_verdicts()
        .iter()
        .find(|v| v.function_name.contains(name) && matches!(v.outcome, VerdictOutcome::Pending))
        .map(|v| (v.id.clone(), v.function_name.clone()));

    if let Some((id, func_name)) = target_id {
        tracker
            .mark_removed(&id, commit.as_deref())
            .map_err(|e| err::internal(e))?;
        println!("✅ Marked '{}' as removed", func_name);
    } else {
        println!("⚠️ No pending function found matching '{}'", name);
    }

    Ok(())
}

fn run_keep(path: &Path, name: &str, reason: &str) -> Result<()> {
    let mut tracker = OutcomeTracker::new(path);

    // Extract data before mutable borrow
    let target_id = tracker
        .get_verdicts()
        .iter()
        .find(|v| v.function_name.contains(name) && matches!(v.outcome, VerdictOutcome::Pending))
        .map(|v| (v.id.clone(), v.function_name.clone()));

    if let Some((id, func_name)) = target_id {
        tracker
            .mark_false_positive(&id, reason)
            .map_err(|e| err::internal(e))?;
        println!("✅ Marked '{}' as false positive", func_name);
    } else {
        println!("⚠️ No pending function found matching '{}'", name);
    }

    Ok(())
}

fn run_update(path: &Path, id: &str, action: UpdateAction) -> Result<()> {
    let mut tracker = OutcomeTracker::new(path);

    match action {
        UpdateAction::Removed { commit } => {
            tracker
                .mark_removed(&id, commit.as_deref())
                .map_err(|e| err::internal(e))?;
            println!("✅ Marked {} as removed", id);
        }
        UpdateAction::FalsePositive { reason } => {
            tracker
                .mark_false_positive(id, &reason)
                .map_err(|e| err::internal(e))?;
            println!("✅ Marked {} as false positive: {}", id, reason);
        }
    }

    Ok(())
}

fn run_stats(path: &Path, detailed: bool) -> Result<()> {
    let tracker = OutcomeTracker::new(path);
    let stats = tracker.get_stats();

    println!("\n📊 Outcome Statistics for: {:?}", path);
    println!("");
    println!("   Total flagged: {}", stats.total_flagged);
    println!(
        "   Removed: {} ({:.1}%)",
        stats.removed_count,
        stats.removal_rate * 100.0
    );
    println!("   Kept (false positives): {}", stats.kept_count);
    println!("   Pending: {}", stats.pending_count);

    if detailed {
        let feedback_stats = tracker.get_feedback_stats();
        println!("\n📈 Detailed Feedback Stats:");
        println!("   Total decisions: {}", feedback_stats.total_decisions);
        println!(
            "   Feedback ratio: {:.1}%",
            feedback_stats.feedback_ratio * 100.0
        );
        println!(
            "   False positive rate: {:.1}%",
            feedback_stats.false_positive_rate * 100.0
        );
    }

    Ok(())
}

// Report Commands

async fn run_report(path: PathBuf, format: &str, output: Option<PathBuf>, llm: bool) -> Result<()> {
    println!("📄 Generating report for: {:?}", path);
    println!("   Format: {}", format);

    let output_file = output.unwrap_or_else(|| {
        let ext = match format {
            "json" => "json",
            "html" => "html",
            "full" => "md",
            _ => "md",
        };
        PathBuf::from(format!("code_analysis.{}", ext))
    });

    // Run analysis
    let config = AnalysisServiceConfig {
        model_path: get_default_model().map(PathBuf::from),
        threshold: None,
        verbose: false,
        debug: false,
        cache: false,
        cache_dir: None,
        llm,
        git: false,
    };

    let mut service = AnalysisService::new(config);
    service.load_model()?;
    let result = service.analyze(&path).await?;

    // Generate output based on format
    let content = match format {
        "json" => result.project_analysis.to_json(),
        "full" => result.project_analysis.to_full_report(),
        _ => result.project_analysis.to_markdown(),
    };

    std::fs::write(&output_file, content)?;
    println!("✅ Report saved to: {:?}", output_file);

    Ok(())
}

// Training & Model Management

fn run_train(data: &Path, val_data: Option<&Path>, output: &Path, precision: f64) -> Result<()> {
    println!("🧠 Training model...");
    println!("   Training data: {:?}", data);
    if let Some(vd) = val_data {
        println!("   Validation data: {:?}", vd);
    }
    println!("   Target precision: {:.2}", precision);

    // Load training data
    let data_str = std::fs::read_to_string(data)?;
    let train_examples: Vec<code_intelligence::analysis::training_data::TrainingExample> =
        serde_json::from_str(&data_str)?;

    let mut classifier = DeadCodeClassifier::new();
    classifier
        .train(&train_examples)
        .map_err(|e| err::training(e))?;

    // Print feature importance
    classifier.print_feature_importance();

    // Save model
    classifier
        .save(output)
        .map_err(|e| err::model(e.to_string()))?;

    println!("\n✅ Model saved to: {:?}", output);
    println!("   Run `ci calibrate` to calibrate the model.");

    Ok(())
}

fn run_train_duplicate(input: &Path, output: &Path) -> Result<()> {
    println!("🧠 Training duplicate detection model...");
    println!("   Input: {:?}", input);
    println!("   Output: {:?}", output);

    use code_intelligence::analysis::training_data::TrainingExample;
    use code_intelligence::ml::duplicate_classifier::{
        DuplicateClassifier, DuplicateExample, DuplicateLabel,
    };

    let data_str = std::fs::read_to_string(input)?;
    let examples: Vec<TrainingExample> = serde_json::from_str(&data_str)?;

    // Generate duplicate pairs from examples
    let mut duplicate_examples = Vec::new();
    let mut processed = std::collections::HashSet::new();

    for i in 0..examples.len().min(200) {
        for j in (i + 1)..examples.len().min(200) {
            let a = &examples[i];
            let b = &examples[j];

            let key = (a.full_path.clone(), b.full_path.clone());
            if processed.contains(&key) {
                continue;
            }
            processed.insert(key);

            let similarity = a
                .features
                .to_feature_vector()
                .iter()
                .zip(b.features.to_feature_vector().iter())
                .map(|(x, y)| if x == y { 1.0 } else { 0.0 })
                .sum::<f64>()
                / a.features.to_feature_vector().len() as f64;

            let label = if similarity > 0.85 {
                DuplicateLabel::Duplicate
            } else if similarity < 0.3 {
                DuplicateLabel::NotDuplicate
            } else {
                continue;
            };

            duplicate_examples.push(DuplicateExample {
                func_a: a.features.clone(),
                func_b: b.features.clone(),
                label,
                confidence: similarity,
            });
        }
    }

    if duplicate_examples.is_empty() {
        return Err(err::training("No duplicate examples generated"));
    }

    let mut classifier = DuplicateClassifier::default();
    let accuracy = classifier.train(&duplicate_examples);
    println!("   Training accuracy: {:.1}%", accuracy * 100.0);

    classifier.save(output)?;
    println!("✅ Model saved to: {:?}", output);

    Ok(())
}

fn run_calibrate(model: &Path, data: &Path, output: &Path, method: &str) -> Result<()> {
    println!("🔬 Calibrating model...");
    println!("   Model: {:?}", model);
    println!("   Data: {:?}", data);
    println!("   Method: {}", method);

    let mut classifier = DeadCodeClassifier::load(&*model.to_string_lossy())?;

    let data_str = std::fs::read_to_string(data)?;
    let val_examples: Vec<code_intelligence::analysis::training_data::TrainingExample> =
        serde_json::from_str(&data_str)?;

    use code_intelligence::ml::calibration::CalibrationMethod;
    let cal_method = match method {
        "temperature" => CalibrationMethod::TemperatureScaling,
        "histogram" => CalibrationMethod::HistogramBinning,
        _ => CalibrationMethod::None,
    };

    if let Some(model) = classifier.get_model_mut() {
        use code_intelligence::ml::calibration::CalibratedModel;
        let calibrated = CalibratedModel::calibrate(model, &val_examples, cal_method);

        // Clone before moving
        let cal_classifier = calibrated.classifier.clone();
        let cal_params = calibrated.calibration.clone();

        classifier.model = Some(cal_classifier);
        classifier.calibration = Some(cal_params);

        let stats = calibrated.calibration_stats(&val_examples);
        println!("\n📊 Calibration Statistics:");
        stats.print();

        classifier.save(output)?;
        println!("\n✅ Calibrated model saved to: {:?}", output);
    } else {
        return Err(err::model("No model found"));
    }

    Ok(())
}

fn run_tune(model: &Path, data: &Path, precision: f64) -> Result<()> {
    println!("🎯 Tuning threshold...");
    println!("   Model: {:?}", model);
    println!("   Target precision: {:.2}", precision);

    let classifier = DeadCodeClassifier::load(&*model.to_string_lossy())?;

    let data_str = std::fs::read_to_string(data)?;
    let val_examples: Vec<code_intelligence::analysis::training_data::TrainingExample> =
        serde_json::from_str(&data_str)?;

    // Find optimal threshold
    let mut best_threshold = 0.92;
    let mut best_f1 = 0.0;

    for threshold in (50..=95).step_by(5).map(|t| t as f64 / 100.0) {
        let mut tp = 0;
        let mut fp = 0;
        let mut fn_ = 0;

        for example in &val_examples {
            let dead_prob = classifier.predict_dead_probability(example);
            let pred = if dead_prob >= threshold {
                code_intelligence::analysis::training_data::TrainingLabel::Dead
            } else {
                code_intelligence::analysis::training_data::TrainingLabel::Alive
            };
            let actual = &example.label;

            match (pred, actual) {
                (
                    code_intelligence::analysis::training_data::TrainingLabel::Dead,
                    code_intelligence::analysis::training_data::TrainingLabel::Dead,
                ) => tp += 1,
                (
                    code_intelligence::analysis::training_data::TrainingLabel::Alive,
                    code_intelligence::analysis::training_data::TrainingLabel::Dead,
                ) => fn_ += 1,
                (
                    code_intelligence::analysis::training_data::TrainingLabel::Dead,
                    code_intelligence::analysis::training_data::TrainingLabel::Alive,
                ) => fp += 1,
                _ => {}
            }
        }

        let p = if tp + fp > 0 {
            tp as f64 / (tp + fp) as f64
        } else {
            0.0
        };
        let r = if tp + fn_ > 0 {
            tp as f64 / (tp + fn_) as f64
        } else {
            0.0
        };
        let f1 = if p + r > 0.0 {
            2.0 * p * r / (p + r)
        } else {
            0.0
        };

        if f1 > best_f1 && p >= precision {
            best_f1 = f1;
            best_threshold = threshold;
        }
    }

    println!("\n📊 Optimal threshold: {:.2}", best_threshold);
    println!("   Best F1: {:.1}%", best_f1 * 100.0);

    Ok(())
}

// Data Management Commands

async fn run_export(path: &Path, output: &Path) -> Result<()> {
    println!("📊 Exporting training data from: {:?}", path);

    use code_intelligence::analysis::roots::ReachabilityAnalyzer;
    use code_intelligence::analysis::roots::{RootDetectionConfig, RootDetector};
    use code_intelligence::analysis::TrainingDataCollector;

    let mut pipeline = Pipeline::new();
    let analysis = pipeline.process_project(path).await?;

    let root_config = RootDetectionConfig::default();
    let root_set = RootDetector::detect_roots(&analysis.call_graph, &analysis.files, &root_config);
    let reachability = ReachabilityAnalyzer::compute_reachability(&analysis.call_graph, &root_set);

    let mut collector = TrainingDataCollector::new();

    // Label functions
    for idx in analysis.call_graph.node_indices() {
        let func = &analysis.call_graph[idx];
        let full_path = &func.full_path;

        let is_reachable = reachability.is_reachable(full_path);
        let has_callers = func.fan_in > 0;

        use code_intelligence::analysis::training_data::TrainingLabel;

        if is_reachable || has_callers {
            collector.add_high_confidence_example(
                func,
                &analysis.call_graph,
                TrainingLabel::Alive,
                0.90,
                "reachable",
            );
        } else if !func.is_public && func.fan_in == 0 {
            collector.add_high_confidence_example(
                func,
                &analysis.call_graph,
                TrainingLabel::Dead,
                0.85,
                "unreachable",
            );
        }
    }

    let json = collector.to_json()?;
    std::fs::write(output, json)?;

    println!("✅ Training data exported to: {:?}", output);
    println!("   Examples: {}", collector.examples.len());
    println!(
        "   Alive: {}, Dead: {}, Unknown: {}",
        collector.stats.alive_count, collector.stats.dead_count, collector.stats.unknown_count
    );

    Ok(())
}

fn run_merge(input: &str, output: &Path, dedup: bool) -> Result<()> {
    println!("📊 Merging training data...");
    println!("   Input pattern: {}", input);
    println!("   Output: {:?}", output);
    if dedup {
        println!("   Deduplication: enabled");
    }

    use code_intelligence::analysis::training_data::TrainingExample;
    use std::fs;

    let mut all_examples = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for entry in fs::read_dir(input)? {
        let path = entry?.path();
        let data = std::fs::read_to_string(&path)?;

        if let Ok(examples) = serde_json::from_str::<Vec<TrainingExample>>(&data) {
            for example in examples {
                let key = format!("{}:{}", example.full_path, example.language);
                if dedup && seen.contains(&key) {
                    continue;
                }
                seen.insert(key);
                all_examples.push(example);
            }
        }
    }

    if all_examples.is_empty() {
        return Err(err::dataset("No examples found"));
    }

    let json = serde_json::to_string_pretty(&all_examples)?;
    std::fs::write(output, json)?;

    println!("✅ Merged {} examples to: {:?}", all_examples.len(), output);

    Ok(())
}

async fn run_collect(repos: &[String], output: &Path, max_repos: usize) -> Result<()> {
    println!("📊 Collecting training data from repositories...");

    let default_repos = vec![
        "https://github.com/rust-lang/rust.git",
        "https://github.com/rust-lang/cargo.git",
        "https://github.com/rust-lang/rust-clippy.git",
        "https://github.com/tokio-rs/tokio.git",
        "https://github.com/serde-rs/serde.git",
    ];

    let repo_list: Vec<String> = if repos.is_empty() {
        default_repos.iter().map(|s| s.to_string()).collect()
    } else {
        repos.to_vec()
    };

    let mut count = 0;
    for repo_url in repo_list.iter().take(max_repos) {
        let repo_name = repo_url
            .split('/')
            .last()
            .unwrap_or("unknown")
            .trim_end_matches(".git");
        let repo_dir = output.join(repo_name);

        println!("   Processing: {}", repo_name);

        // Clone repository
        if !repo_dir.exists() {
            let status = std::process::Command::new("git")
                .args([
                    "clone",
                    "--depth",
                    "1",
                    repo_url,
                    &repo_dir.to_string_lossy(),
                ])
                .status()?;

            if !status.success() {
                eprintln!("      ⚠️ Failed to clone {}", repo_name);
                continue;
            }
        }

        // Export training data
        let output_file = output.join(format!("{}.json", repo_name));
        let _ = run_export(&repo_dir, &output_file).await;

        count += 1;
    }

    println!("✅ Processed {} repositories", count);
    Ok(())
}

// Special Commands

async fn run_dashboard(path: &Path, model: Option<PathBuf>) -> Result<()> {
    println!("📊 Opening dashboard for: {:?}", path);

    let mut cmd = std::process::Command::new("dead_code_dashboard");
    cmd.arg(path);
    if let Some(m) = model {
        cmd.args(["--model", &m.to_string_lossy()]);
    }
    let status = cmd.status()?;

    if !status.success() {
        return Err(err::internal("Dashboard failed"));
    }

    Ok(())
}

async fn run_self_analyze(format: &str, output: Option<PathBuf>) -> Result<()> {
    println!("🔍 Analyzing code-intelligence itself...");

    let current_dir = std::env::current_dir()?;
    let output_file = output.unwrap_or_else(|| {
        let ext = match format {
            "json" => "json",
            "full" => "md",
            _ => "md",
        };
        PathBuf::from(format!("self_analysis.{}", ext))
    });

    let config = AnalysisServiceConfig {
        model_path: get_default_model().map(PathBuf::from),
        threshold: None,
        verbose: true,
        debug: false,
        cache: false,
        cache_dir: None,
        llm: false,
        git: false,
    };

    let mut service = AnalysisService::new(config);
    service.load_model()?;
    let result = service.analyze(&current_dir).await?;

    let content = match format {
        "json" => result.project_analysis.to_json(),
        "full" => result.project_analysis.to_full_report(),
        _ => result.project_analysis.to_markdown(),
    };

    std::fs::write(&output_file, content)?;
    println!("✅ Self-analysis saved to: {:?}", output_file);

    Ok(())
}

// CI Mode

async fn run_ci(
    path: PathBuf,
    max_dead: Option<usize>,
    max_ratio: Option<f64>,
    format: &str,
    output: Option<PathBuf>,
    fail_on_dead: bool,
    threshold: f64,
    conservative: bool,
) -> Result<()> {
    println!("🤖 Running in CI mode for: {:?}", path);
    println!("   Threshold: {:.2}", threshold);
    if conservative {
        println!("   Conservative mode: ON");
    }

    // Run analysis with CI-optimized settings
    let config = AnalysisServiceConfig {
        model_path: get_default_model().map(PathBuf::from),
        threshold: Some(threshold),
        verbose: false,
        debug: false,
        cache: false,
        cache_dir: None,
        llm: false,
        git: false,
    };

    let mut service = AnalysisService::new(config);
    service.load_model()?;
    let result = service.analyze(&path).await?;

    // Generate report
    let report = if format == "json" {
        serde_json::json!({
            "project": path.to_string_lossy(),
            "threshold": threshold,
            "total_functions": result.call_graph.node_count(),
            "dead_functions": result.dead_verdicts.len(),
            "alive_functions": result.alive_verdicts.len(),
            "dead_ratio": if result.call_graph.node_count() > 0 {
                result.dead_verdicts.len() as f64 / result.call_graph.node_count() as f64
            } else { 0.0 },
            "status": if result.dead_verdicts.is_empty() { "PASS" } else { "FAIL" },
        })
        .to_string()
    } else {
        format!(
            "📊 CI Report\n===========\n\
             Project: {}\n\
             Threshold: {:.2}\n\
             Total Functions: {}\n\
             Dead Functions: {}\n\
             Dead Ratio: {:.1}%\n\
             Status: {}\n",
            path.to_string_lossy(),
            threshold,
            result.call_graph.node_count(),
            result.dead_verdicts.len(),
            if result.call_graph.node_count() > 0 {
                result.dead_verdicts.len() as f64 / result.call_graph.node_count() as f64 * 100.0
            } else {
                0.0
            },
            if result.dead_verdicts.is_empty() {
                "✅ PASS"
            } else {
                "❌ FAIL"
            }
        )
    };

    if let Some(output_path) = output {
        std::fs::write(output_path, &report)?;
    } else {
        println!("{}", report);
    }

    // Check conditions
    if let Some(max) = max_dead {
        if result.dead_verdicts.len() > max {
            eprintln!(
                "❌ Dead code count {} exceeds limit {}",
                result.dead_verdicts.len(),
                max
            );
            std::process::exit(1);
        }
    }

    if let Some(max) = max_ratio {
        let ratio = if result.call_graph.node_count() > 0 {
            result.dead_verdicts.len() as f64 / result.call_graph.node_count() as f64
        } else {
            0.0
        };
        if ratio > max {
            eprintln!(
                "❌ Dead ratio {:.1}% exceeds limit {:.1}%",
                ratio * 100.0,
                max * 100.0
            );
            std::process::exit(1);
        }
    }

    if fail_on_dead && !result.dead_verdicts.is_empty() {
        eprintln!("❌ Found {} dead functions", result.dead_verdicts.len());
        std::process::exit(1);
    }

    if result.dead_verdicts.is_empty() {
        println!("✅ No dead code found!");
    }

    Ok(())
}

// Feedback Export

fn run_export_feedback(path: &Path, output: &Path) -> Result<()> {
    println!("📊 Exporting feedback from: {:?}", path);

    let tracker = OutcomeTracker::new(path);
    let stats = tracker.get_feedback_stats();

    println!("   Total decisions: {}", stats.total_decisions);
    println!("   Feedback ratio: {:.1}%", stats.feedback_ratio * 100.0);

    if stats.total_decisions == 0 {
        println!("   ⚠️ No decisions to export.");
        return Ok(());
    }

    tracker
        .save_feedback_as_training_data(output)
        .map_err(|e| err::internal(e))?;

    println!("✅ Feedback exported to: {:?}", output);

    Ok(())
}

// Configuration

fn run_config(action: ConfigAction) -> Result<()> {
    let mut config = load_config();

    match action {
        ConfigAction::Set { key, value } => match key.as_str() {
            "model" => {
                config.defaults.model = Some(value.clone());
                save_config(&config)?;
                println!("✅ Model set to: {}", value);
            }
            "duplicate_model" => {
                config.defaults.duplicate_model = Some(value.clone());
                save_config(&config)?;
                println!("✅ Duplicate model set to: {}", value);
            }
            "threshold" => {
                let threshold = value.parse::<f64>()?;
                config.defaults.threshold = Some(threshold);
                save_config(&config)?;
                println!("✅ Threshold set to: {:.2}", threshold);
            }
            "verbose" => {
                let verbose = value.parse::<bool>()?;
                config.defaults.verbose = verbose;
                save_config(&config)?;
                println!("✅ Verbose set to: {}", verbose);
            }
            "llm_provider" => {
                config.defaults.llm_provider = Some(value.clone());
                save_config(&config)?;
                println!("✅ LLM provider set to: {}", value);
            }
            "llm_model" => {
                config.defaults.llm_model = Some(value.clone());
                save_config(&config)?;
                println!("✅ LLM model set to: {}", value);
            }
            _ => {
                println!("⚠️ Unknown config key: {}", key);
                println!("   Available: model, duplicate_model, threshold, verbose, llm_provider, llm_model");
            }
        },
        ConfigAction::Get { key } => match key.as_str() {
            "model" => println!(
                "{}",
                config.defaults.model.as_deref().unwrap_or("(not set)")
            ),
            "duplicate_model" => println!(
                "{}",
                config
                    .defaults
                    .duplicate_model
                    .as_deref()
                    .unwrap_or("(not set)")
            ),
            "threshold" => println!("{:.2}", config.defaults.threshold.unwrap_or(0.92)),
            "verbose" => println!("{}", config.defaults.verbose),
            "llm_provider" => println!(
                "{}",
                config.defaults.llm_provider.as_deref().unwrap_or("ollama")
            ),
            "llm_model" => println!(
                "{}",
                config.defaults.llm_model.as_deref().unwrap_or("phi:2.7b")
            ),
            _ => println!("⚠️ Unknown config key: {}", key),
        },
        ConfigAction::List => {
            println!("📋 Current Configuration:");
            println!("");
            println!("[defaults]");
            println!("  model = {:?}", config.defaults.model);
            println!("  duplicate_model = {:?}", config.defaults.duplicate_model);
            println!("  threshold = {:?}", config.defaults.threshold);
            println!("  verbose = {}", config.defaults.verbose);
            println!("  llm_provider = {:?}", config.defaults.llm_provider);
            println!("  llm_model = {:?}", config.defaults.llm_model);
        }
    }

    Ok(())
}

// Helper Functions

fn get_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config/code-intelligence/config.toml")
}

fn load_config() -> GlobalConfig {
    let path = get_config_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        toml::from_str(&content).unwrap_or_else(|_| GlobalConfig {
            defaults: Defaults::default(),
            projects: std::collections::HashMap::new(),
        })
    } else {
        GlobalConfig {
            defaults: Defaults::default(),
            projects: std::collections::HashMap::new(),
        }
    }
}

fn save_config(config: &GlobalConfig) -> Result<()> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| err::config(format!("Failed to create config dir: {}", e)))?;
    }
    let content = toml::to_string_pretty(config)
        .map_err(|e| err::config(format!("Failed to serialize: {}", e)))?;
    std::fs::write(&path, content)
        .map_err(|e| err::config(format!("Failed to write config: {}", e)))?;
    Ok(())
}

fn save_project_config(path: &Path, project_config: ProjectConfig) -> Result<()> {
    let mut config = load_config();
    let key = path
        .canonicalize()
        .unwrap_or(path.to_path_buf())
        .to_string_lossy()
        .to_string();
    config.projects.insert(key, project_config);
    save_config(&config)
}

fn get_default_model() -> Option<String> {
    load_config().defaults.model
}

fn get_default_duplicate_model() -> Option<PathBuf> {
    load_config().defaults.duplicate_model.map(PathBuf::from)
}

fn _get_default_threshold() -> f64 {
    load_config().defaults.threshold.unwrap_or(0.55)
}

fn detect_project_type(path: &Path) -> Option<String> {
    let path = path.canonicalize().unwrap_or(path.to_path_buf());

    let has_rust = path.join("Cargo.toml").exists();
    let has_typescript = path.join("package.json").exists() && path.join("tsconfig.json").exists();
    let has_javascript = path.join("package.json").exists();
    let has_go = path.join("go.mod").exists();
    let has_java = path.join("pom.xml").exists() || path.join("build.gradle").exists();
    let has_python = path.join("requirements.txt").exists() || path.join("pyproject.toml").exists();

    let lang_count = [has_rust, has_typescript, has_go, has_java, has_python]
        .iter()
        .filter(|&&x| x)
        .count();

    if lang_count > 1 {
        Some("mixed".to_string())
    } else if has_rust {
        Some("rust".to_string())
    } else if has_typescript {
        Some("typescript".to_string())
    } else if has_javascript {
        Some("javascript".to_string())
    } else if has_go {
        Some("go".to_string())
    } else if has_java {
        Some("java".to_string())
    } else if has_python {
        Some("python".to_string())
    } else {
        None
    }
}

fn resolve_path(path: &Path) -> Result<PathBuf> {
    let resolved = if path.to_string_lossy() == "." {
        std::env::current_dir().map_err(|e| err::io(path.to_path_buf(), e))?
    } else if path.is_relative() {
        std::env::current_dir()
            .map_err(|e| err::io(path.to_path_buf(), e))?
            .join(path)
    } else {
        path.to_path_buf()
    };

    if !resolved.exists() {
        return Err(err::analysis(format!(
            "Path does not exist: {:?}",
            resolved
        )));
    }

    Ok(resolved)
}
