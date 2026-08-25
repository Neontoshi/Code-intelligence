// src/bin/ci/commands.rs

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "ci",
    author = "Code Intelligence Team",
    version = "0.1.2",
    about = "Semantic codebase intelligence: high-precision dead code detection, structural deduplication, and architectural mapping"
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    // CORE COMMANDS (Visible in help)
    /// Full project analysis (dead code + duplicates + important functions)
    Analyze {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Confidence threshold (0.0 - 1.0)
        #[arg(long)]
        threshold: Option<f64>,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
        /// Model file path
        #[arg(long)]
        model: Option<PathBuf>,
        /// Enable LLM analysis
        #[arg(long)]
        llm: bool,
        /// Enable Git analysis
        #[arg(long)]
        git: bool,
        /// Enable disk cache
        #[arg(long)]
        cache: bool,
        /// Cache directory
        #[arg(long)]
        cache_dir: Option<PathBuf>,
    },

    /// Quick list of dead functions
    List {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Show all (including removed and kept)
        #[arg(long)]
        all: bool,
    },

    /// Detailed dead code report with priority removal order
    DeadCode {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Confidence threshold (0.0 - 1.0)
        #[arg(long, default_value = "0.92")]
        threshold: f64,
        /// Output markdown file (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Model file path
        #[arg(long)]
        model: Option<PathBuf>,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Detailed deduplication report
    Dedup {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Similarity threshold (0.0 - 1.0)
        #[arg(long, default_value = "0.85")]
        threshold: f64,
        /// Output markdown file (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Use ML model for duplicate detection
        #[arg(long)]
        ml: bool,
        /// Duplicate model file path
        #[arg(long)]
        duplicate_model: Option<PathBuf>,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Mark function as removed
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

    /// Mark function as false positive (kept)
    Keep {
        /// Function name (partial match supported)
        name: String,
        /// Reason for keeping
        reason: String,
        /// Path (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
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

    /// Generate overview report
    Report {
        /// Path (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Output format: markdown, json, html, full
        #[arg(long, default_value = "markdown")]
        format: String,
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Include LLM analysis
        #[arg(long)]
        llm: bool,
    },

    /// Generate call graph visualization (HTML)
    Graph {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Mode: interactive or overview
        #[arg(long, default_value = "interactive")]
        mode: String,
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

    /// CI/CD mode with exit code
    Check {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Fail if dead code count exceeds this
        #[arg(long)]
        max_dead: Option<usize>,
        /// Fail if dead code ratio exceeds this (0.0-1.0)
        #[arg(long)]
        max_ratio: Option<f64>,
        /// Threshold for dead code confidence
        #[arg(long, default_value = "0.80")]
        threshold: f64,
        /// Output format: json, markdown, summary
        #[arg(long, default_value = "json")]
        format: String,
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Fail on any dead code
        #[arg(long, default_value = "true")]
        fail_on_dead: bool,
        /// Conservative mode (higher threshold)
        #[arg(long)]
        conservative: bool,
    },

    /// Configure settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    // ADVANCED COMMANDS (Hidden)
    /// Train the ML model (advanced)
    #[cfg(feature = "advanced")]
    #[command(hide = true)]
    Train {
        #[arg(long, default_value = "data/train.json")]
        data: PathBuf,
        #[arg(long, default_value = "data/val.json")]
        val_data: Option<PathBuf>,
        #[arg(long, default_value = "model.bin")]
        output: PathBuf,
        #[arg(long, default_value = "0.95")]
        precision: f64,
    },

    /// Train duplicate detection model (advanced)
    #[cfg(feature = "advanced")]
    #[command(hide = true)]
    TrainDuplicate {
        input: PathBuf,
        #[arg(long, default_value = "duplicate_model.bin")]
        output: PathBuf,
    },

    /// Calibrate a trained model (advanced)
    #[cfg(feature = "advanced")]
    #[command(hide = true)]
    Calibrate {
        #[arg(long, default_value = "model.bin")]
        model: PathBuf,
        #[arg(long, default_value = "data/val.json")]
        data: PathBuf,
        #[arg(long, default_value = "model_calibrated.bin")]
        output: PathBuf,
        #[arg(long, default_value = "temperature")]
        method: String,
    },

    /// Tune confidence threshold (advanced)
    #[cfg(feature = "advanced")]
    #[command(hide = true)]
    Tune {
        #[arg(long, default_value = "model.bin")]
        model: PathBuf,
        #[arg(long, default_value = "data/val.json")]
        data: PathBuf,
        #[arg(long, default_value = "0.99")]
        precision: f64,
    },

    /// Export training data (advanced)
    #[cfg(feature = "advanced")]
    #[command(hide = true)]
    Export {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, default_value = "training_data.json")]
        output: PathBuf,
    },

    /// Merge training data files (advanced)
    #[cfg(feature = "advanced")]
    #[command(hide = true)]
    Merge {
        #[arg(default_value = "training_data/*.json")]
        input: String,
        #[arg(long, default_value = "combined_training.json")]
        output: PathBuf,
        #[arg(long)]
        dedup: bool,
    },

    /// Collect training data from repositories (advanced)
    #[cfg(feature = "advanced")]
    #[command(hide = true)]
    Collect {
        repos: Vec<String>,
        #[arg(long, default_value = "training_data")]
        output: PathBuf,
        #[arg(long, default_value = "50")]
        max_repos: usize,
    },

    /// Export dashboard decisions as training data (advanced)
    #[cfg(feature = "advanced")]
    #[command(hide = true)]
    ExportFeedback {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(short, long, default_value = "feedback_training.json")]
        output: PathBuf,
    },

    /// Update outcome by verdict ID (advanced)
    #[cfg(feature = "advanced")]
    #[command(hide = true)]
    Update {
        #[arg(default_value = ".")]
        path: PathBuf,
        id: String,
        #[command(subcommand)]
        action: UpdateAction,
    },

    /// Analyze code-intelligence itself (advanced)
    #[cfg(feature = "advanced")]
    #[command(hide = true)]
    SelfAnalyze {
        #[arg(long, default_value = "markdown")]
        format: String,
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Set a configuration value
    Set { key: String, value: String },
    /// Get a configuration value
    Get { key: String },
    /// List all configuration values
    List,
}

#[derive(Subcommand, Debug)]
#[cfg(feature = "advanced")]
pub enum UpdateAction {
    Removed {
        #[arg(long)]
        commit: Option<String>,
    },
    FalsePositive {
        reason: String,
    },
}
