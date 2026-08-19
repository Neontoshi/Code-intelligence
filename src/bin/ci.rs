//! Code Intelligence CLI - Complete Dead Code Detection Tool
//!
//! Usage:
//!   ci analyze [path]              - Analyze project for dead code
//!   ci dedup [path]                - Find duplicate code
//!   ci list [path]                 - List dead functions found
//!   ci remove <name>               - Mark function as removed
//!   ci keep <name> "reason"        - Mark as false positive
//!   ci stats [path]                - Show outcome statistics
//!   ci report [path] [--format]    - Generate report
//!   ci graph [path] [--format]     - Generate call graph
//!   ci train [--data] [--output]   - Train ML model
//!   ci calibrate [--model] [--data] - Calibrate model
//!   ci tune [--model] [--data]     - Tune threshold
//!   ci config set <key> <value>    - Set config
//!   ci config get <key>            - Get config
//!   ci config list                 - List config
//!   ci dashboard [path]            - Open interactive dashboard
//!   ci llm [path]                  - Analyze with LLM
//!   ci compare [path]              - Compare ML models
//!   ci features [path]             - Analyze features per language
//!   ci export [path] [--output]    - Export training data
//!   ci merge [--input] [--output]  - Merge training data
//!   ci self                         - Analyze code-intelligence itself

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

// Config

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
    pub threshold: Option<f64>,
    pub verbose: bool,
    pub llm_provider: Option<String>,
    pub llm_model: Option<String>,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            model: None,
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

#[derive(Parser, Debug)]
#[command(
    name = "ci",
    author = "Code Intelligence Team",
    version = "0.1.0",
    about = "Code Intelligence - Complete dead code detection toolkit",
    long_about = "A comprehensive CLI tool for detecting, managing, and removing dead code across any project."
)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    // Core Analysis
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
        /// Cache directory (default: <project>/.code-intelligence-cache)
        #[arg(long)]
        cache_dir: Option<PathBuf>,
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
    },

    /// Generate call graph visualization
    Graph {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Output format: dot, svg, png
        #[arg(long, default_value = "dot")]
        format: String,
        /// Output file
        #[arg(long)]
        output: Option<PathBuf>,
        /// Max nodes to include
        #[arg(long, default_value = "60")]
        max_nodes: usize,
        /// Entry point for focused graph
        #[arg(long)]
        entry: Option<String>,
        /// Depth for focused graph
        #[arg(long, default_value = "2")]
        depth: usize,
    },

    /// Analyze with LLM integration
    Llm {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// LLM provider: ollama, openai, anthropic
        #[arg(long, default_value = "ollama")]
        provider: String,
        /// LLM model name
        #[arg(long)]
        model: Option<String>,
        /// API key (for cloud providers)
        #[arg(long)]
        api_key: Option<String>,
        /// Temperature (0.0 - 1.0)
        #[arg(long, default_value = "0.3")]
        temperature: f32,
        /// Max tokens
        #[arg(long, default_value = "1000")]
        max_tokens: usize,
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

    /// Show outcome statistics for a project
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
        /// Calibration method: temperature, histogram, isotonic
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

    /// Compare different ML models
    Compare {
        /// Training data
        #[arg(long, default_value = "data/train.json")]
        train_data: PathBuf,
        /// Validation data
        #[arg(long, default_value = "data/val.json")]
        val_data: PathBuf,
        /// Test data
        #[arg(long, default_value = "data/test.json")]
        test_data: PathBuf,
        /// Output directory
        #[arg(long, default_value = "model_comparison")]
        output: PathBuf,
    },

    /// Analyze feature importance per language
    Features {
        /// Training data file
        #[arg(long, default_value = "combined_training.json")]
        data: PathBuf,
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
        /// Input files (glob pattern or list)
        #[arg(default_value = "training_data/*.json")]
        input: String,
        /// Output file
        #[arg(long, default_value = "combined_training.json")]
        output: PathBuf,
        /// Deduplicate examples
        #[arg(long)]
        dedup: bool,
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

    /// Analyze features per language
    AnalyzeFeatures {
        /// Training data file
        #[arg(long, default_value = "combined_training.json")]
        data: PathBuf,
    },

    /// Evaluate model per language
    Evaluate {
        /// Model file path
        #[arg(long, default_value = "model.bin")]
        model: PathBuf,
        /// Test data file
        #[arg(long, default_value = "data/test.json")]
        test_data: PathBuf,
        /// Validation data file (optional)
        #[arg(long)]
        val_data: Option<PathBuf>,
        /// Show detailed metrics
        #[arg(long)]
        detailed: bool,
    },

    /// Configure global settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigAction {
    /// Set a config value
    Set { key: String, value: String },
    /// Get a config value
    Get { key: String },
    /// List all config values
    List,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        // Core Analysis
        Commands::Analyze {
            path,
            threshold,
            verbose,
            llm,
            git,
            cache,
            cache_dir,
        } => {
            let project_path = resolve_path(&path)?;
            run_analyze(
                &project_path,
                threshold,
                verbose,
                llm,
                git,
                cache,
                cache_dir,
            )?;
        }
        Commands::Dedup {
            path,
            threshold,
            ml,
        } => {
            let project_path = resolve_path(&path)?;
            run_dedup(&project_path, threshold, ml)?;
        }
        Commands::Graph {
            path,
            format,
            output,
            max_nodes,
            entry,
            depth,
        } => {
            let project_path = resolve_path(&path)?;
            run_graph(&project_path, &format, output, max_nodes, entry, depth)?;
        }
        Commands::Llm {
            path,
            provider,
            model,
            api_key,
            temperature,
            max_tokens,
        } => {
            let project_path = resolve_path(&path)?;
            run_llm(
                &project_path,
                &provider,
                model,
                api_key,
                temperature,
                max_tokens,
            )?;
        }

        // Outcome Management
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
            run_report(&project_path, &format, output, llm)?;
        }

        // Training & Model Management
        Commands::Train {
            data,
            val_data,
            output,
            precision,
        } => {
            run_train(&data, val_data.as_deref(), &output, precision)?;
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
        Commands::Compare {
            train_data,
            val_data,
            test_data,
            output,
        } => {
            run_compare(&train_data, &val_data, &test_data, &output)?;
        }
        Commands::Features { data } => {
            run_features(&data)?;
        }
        Commands::Export { path, output } => {
            let project_path = resolve_path(&path)?;
            run_export(&project_path, &output)?;
        }
        Commands::Merge {
            input,
            output,
            dedup,
        } => {
            run_merge(&input, &output, dedup)?;
        }

        // Special Commands
        Commands::Dashboard { path, model } => {
            let project_path = resolve_path(&path)?;
            run_dashboard(&project_path, model)?;
        }
        Commands::SelfAnalyze { format, output } => {
            run_self_analyze(&format, output)?;
        }
        Commands::AnalyzeFeatures { data } => {
            run_analyze_features(&data)?;
        }
        Commands::Evaluate {
            model,
            test_data,
            val_data,
            detailed,
        } => {
            run_evaluate(&model, &test_data, val_data.as_deref(), detailed)?;
        }

        // Configuration
        Commands::Config { action } => {
            run_config(action)?;
        }
    }

    Ok(())
}

// Config Helper Functions

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

fn save_config(config: &GlobalConfig) -> Result<(), String> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    let content =
        toml::to_string_pretty(config).map_err(|e| format!("Failed to serialize config: {}", e))?;
    std::fs::write(&path, content).map_err(|e| format!("Failed to write config: {}", e))?;
    Ok(())
}

fn get_default_model() -> Option<String> {
    let config = load_config();
    config.defaults.model.clone()
}

fn get_default_threshold() -> f64 {
    let config = load_config();
    config.defaults.threshold.unwrap_or(0.55)
}

fn get_project_config(path: &Path) -> Option<ProjectConfig> {
    let config = load_config();
    let key = path
        .canonicalize()
        .unwrap_or(path.to_path_buf())
        .to_string_lossy()
        .to_string();
    config.projects.get(&key).cloned()
}

fn save_project_config(path: &Path, project_config: ProjectConfig) -> Result<(), String> {
    let mut config = load_config();
    let key = path
        .canonicalize()
        .unwrap_or(path.to_path_buf())
        .to_string_lossy()
        .to_string();
    config.projects.insert(key, project_config);
    save_config(&config)
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

fn resolve_path(path: &Path) -> Result<PathBuf, String> {
    let resolved = if path.to_string_lossy() == "." {
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?
    } else if path.is_relative() {
        std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?
            .join(path)
    } else {
        path.to_path_buf()
    };

    if !resolved.exists() {
        return Err(format!("Path does not exist: {:?}", resolved));
    }

    Ok(resolved)
}

fn run_analyze(
    path: &Path,
    threshold: Option<f64>,
    verbose: bool,
    llm: bool,
    git: bool,
    cache: bool,
    cache_dir: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Analyzing project: {:?}", path);

    let project_type = detect_project_type(path);
    if let Some(pt) = &project_type {
        println!("📊 Detected project type: {}", pt);
    }

    let model = get_default_model();
    if model.is_none() {
        println!("⚠️ No model configured. Run: ci config set model <path>");
        return Ok(());
    }

    let threshold = threshold
        .or(get_project_config(path).and_then(|c| c.threshold))
        .unwrap_or_else(get_default_threshold);
    let model_path = model.unwrap();

    println!("📊 Using threshold: {:.2}", threshold);
    println!("🤖 Using model: {}", model_path);
    if llm {
        println!("🤖 LLM analysis: enabled");
    }
    if git {
        println!("📊 Git analysis: enabled");
    }
    if cache {
        let cache_path = cache_dir
            .clone()
            .unwrap_or_else(|| path.join(".code-intelligence-cache"));
        println!("💾 Cache enabled: {:?}", cache_path);
    } else {
        println!("💾 Cache disabled (use --cache to enable)");
    }
    println!("");

    // Find the dead_code_check binary
    let binary_path = find_binary("dead_code_check");

    let status = if let Some(bin_path) = binary_path {
        let mut cmd = std::process::Command::new(&bin_path);
        cmd.arg(path)
            .args(["--model", &model_path])
            .args(["--threshold", &format!("{:.2}", threshold)]);

        if verbose {
            cmd.arg("--verbose");
        }
        if llm {
            cmd.arg("--llm");
        }
        if git {
            cmd.arg("--git");
        }
        if cache {
            cmd.arg("--cache");
            if let Some(cache_dir) = &cache_dir {
                cmd.args(["--cache-dir", &cache_dir.to_string_lossy()]);
            }
        }

        cmd.status()?
    } else {
        eprintln!("❌ Could not find 'dead_code_check' binary.");
        eprintln!("   Please install it: cargo install --path . --bin dead_code_check");
        return Ok(());
    };

    if status.success() {
        let project_config = ProjectConfig {
            path: path.to_string_lossy().to_string(),
            project_type: project_type,
            threshold: Some(threshold),
            last_analyzed: Some(chrono::Local::now().to_string()),
            dead_count: None,
        };
        let _ = save_project_config(path, project_config);
        println!("\n✅ Analysis complete!");
    } else {
        eprintln!("\n❌ Analysis failed");
    }

    Ok(())
}

fn run_dedup(path: &Path, threshold: f64, ml: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Finding duplicates in: {:?}", path);
    println!("📊 Similarity threshold: {:.2}", threshold);
    if ml {
        println!("🤖 Using ML model for duplicate detection");
    }
    println!("");

    let mut cmd = Command::new("dedup_check");
    cmd.arg(path);

    if ml {
        let model = get_default_model().unwrap_or_else(|| "model.bin".to_string());
        cmd.args(["--duplicate-model", &model]);
    }

    let status = cmd.status()?;

    if status.success() {
        println!("\n✅ Deduplication complete!");
    } else {
        eprintln!("\n❌ Deduplication failed");
    }

    Ok(())
}

fn run_graph(
    path: &Path,
    format: &str,
    output: Option<PathBuf>,
    max_nodes: usize,
    entry: Option<String>,
    depth: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Generating call graph for: {:?}", path);
    println!("   Format: {}", format);
    println!("   Max nodes: {}", max_nodes);

    let output_file = output.unwrap_or_else(|| {
        if format == "dot" {
            PathBuf::from("call_graph.dot")
        } else {
            PathBuf::from(format!("call_graph.{}", format))
        }
    });

    let graph_type = if entry.is_some() {
        "graphviz-focused"
    } else if format == "dot" {
        "graphviz"
    } else if format == "svg" || format == "png" {
        // Generate dot first, then convert
        "graphviz"
    } else {
        "graphviz"
    };

    // Use the main binary for graph generation
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--release", "--"]);
    cmd.arg(path)
        .args(["--format", graph_type])
        .args(["--graph-max-nodes", &max_nodes.to_string()])
        .args(["--output", &output_file.to_string_lossy()]);

    if let Some(entry_point) = entry {
        cmd.args(["--graph-entry", &entry_point]);
        cmd.args(["--graph-depth", &depth.to_string()]);
    }

    let status = cmd.status()?;

    if status.success() {
        println!("\n✅ Graph saved to: {:?}", output_file);
        if format == "svg" || format == "png" {
            println!("   Converting to {} format...", format);
        }
    } else {
        eprintln!("\n❌ Graph generation failed");
    }

    Ok(())
}

fn run_llm(
    path: &Path,
    provider: &str,
    model: Option<String>,
    api_key: Option<String>,
    temperature: f32,
    max_tokens: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Running LLM analysis on: {:?}", path);
    println!("   Provider: {}", provider);
    if let Some(m) = &model {
        println!("   Model: {}", m);
    }
    println!("   Temperature: {:.1}", temperature);
    println!("   Max tokens: {}", max_tokens);
    println!("");

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--release", "--"]);
    cmd.arg(path)
        .args(["--llm"])
        .args(["--llm-provider", provider])
        .args(["--llm-temperature", &temperature.to_string()])
        .args(["--llm-max-tokens", &max_tokens.to_string()]);

    if let Some(m) = model {
        cmd.args(["--llm-model", &m]);
    }
    if let Some(key) = api_key {
        cmd.args(["--llm-api-key", &key]);
    }

    let status = cmd.status()?;

    if status.success() {
        println!("\n✅ LLM analysis complete!");
    } else {
        eprintln!("\n❌ LLM analysis failed");
    }

    Ok(())
}

fn run_list(path: &Path, all: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Listing dead functions in: {:?}", path);

    let outcome_file = path.join(".code-intelligence-outcomes.json");
    if !outcome_file.exists() {
        println!("No tracked outcomes found. Run `ci analyze` first.");
        return Ok(());
    }

    let data = std::fs::read_to_string(&outcome_file)?;
    let outcomes: Vec<serde_json::Value> = serde_json::from_str(&data)?;

    if outcomes.is_empty() {
        println!("No tracked verdicts.");
        return Ok(());
    }

    let filtered: Vec<_> = if all {
        outcomes.iter().collect()
    } else {
        outcomes
            .iter()
            .filter(|v| v.get("outcome").and_then(|o| o.as_str()).unwrap_or("") == "Pending")
            .collect()
    };

    if filtered.is_empty() {
        if all {
            println!("No tracked verdicts.");
        } else {
            println!("✅ No pending verdicts! All tracked functions have been reviewed.");
            println!("   Use `ci list --all` to see all verdicts.");
        }
        return Ok(());
    }

    let status_label = if all { "All" } else { "Pending" };
    println!(
        "\n📋 {} Dead Functions ({} total):",
        status_label,
        filtered.len()
    );
    println!("");
    println!("| # | Function | Confidence | File | Status |");
    println!("|---|----------|------------|------|--------|");

    for (i, v) in filtered.iter().enumerate() {
        let name = v
            .get("function_name")
            .and_then(|n| n.as_str())
            .unwrap_or("unknown");
        let confidence = v.get("confidence").and_then(|c| c.as_f64()).unwrap_or(0.0) * 100.0;
        let file = v
            .get("file")
            .and_then(|f| f.as_str())
            .and_then(|f| f.split('/').last())
            .unwrap_or("unknown");
        let status = v
            .get("outcome")
            .and_then(|o| o.as_str())
            .unwrap_or("unknown");

        let short_name = if name.len() > 30 { &name[..30] } else { name };
        println!(
            "| {} | {} | {:.1}% | {} | {} |",
            i + 1,
            short_name,
            confidence,
            file,
            status
        );
    }

    if !all {
        println!("\n💡 To manage: ci remove <name> or ci keep <name> \"reason\"");
    }

    Ok(())
}

fn run_remove(
    path: &Path,
    name: &str,
    commit: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🗑️ Marking '{}' as removed in: {:?}", name, path);

    let outcome_file = path.join(".code-intelligence-outcomes.json");
    if !outcome_file.exists() {
        println!("No tracked outcomes found. Run `ci analyze` first.");
        return Ok(());
    }

    let data = std::fs::read_to_string(&outcome_file)?;
    let mut outcomes: Vec<serde_json::Value> = serde_json::from_str(&data)?;

    let mut found = false;
    let mut removed_name = String::new();

    for outcome in &mut outcomes {
        let func_name = outcome
            .get("function_name")
            .and_then(|n| n.as_str())
            .unwrap_or("");
        if func_name.contains(name)
            && outcome
                .get("outcome")
                .and_then(|o| o.as_str())
                .unwrap_or("")
                == "Pending"
        {
            removed_name = func_name.to_string();
            outcome["outcome"] = serde_json::json!("Removed");
            outcome["outcome_date"] = serde_json::json!(chrono::Local::now().timestamp());
            if let Some(commit_hash) = &commit {
                outcome["removed_commit"] = serde_json::json!(commit_hash);
            }
            outcome["notes"] = serde_json::json!("Removed by user");
            found = true;
            break;
        }
    }

    if !found {
        println!("⚠️ No pending function found matching '{}'", name);
        println!("   Use `ci list` to see available functions.");
        return Ok(());
    }

    let new_data = serde_json::to_string_pretty(&outcomes)?;
    std::fs::write(&outcome_file, new_data)?;
    println!("✅ Marked '{}' as removed", removed_name);

    Ok(())
}

fn run_keep(path: &Path, name: &str, reason: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📌 Keeping '{}' (false positive): {}", name, reason);

    let outcome_file = path.join(".code-intelligence-outcomes.json");
    if !outcome_file.exists() {
        println!("No tracked outcomes found. Run `ci analyze` first.");
        return Ok(());
    }

    let data = std::fs::read_to_string(&outcome_file)?;
    let mut outcomes: Vec<serde_json::Value> = serde_json::from_str(&data)?;

    let mut found = false;
    let mut kept_name = String::new();

    for outcome in &mut outcomes {
        let func_name = outcome
            .get("function_name")
            .and_then(|n| n.as_str())
            .unwrap_or("");
        if func_name.contains(name)
            && outcome
                .get("outcome")
                .and_then(|o| o.as_str())
                .unwrap_or("")
                == "Pending"
        {
            kept_name = func_name.to_string();
            outcome["outcome"] = serde_json::json!("Kept");
            outcome["outcome_date"] = serde_json::json!(chrono::Local::now().timestamp());
            outcome["notes"] = serde_json::json!(format!("False positive: {}", reason));
            found = true;
            break;
        }
    }

    if !found {
        println!("⚠️ No pending function found matching '{}'", name);
        println!("   Use `ci list` to see available functions.");
        return Ok(());
    }

    let new_data = serde_json::to_string_pretty(&outcomes)?;
    std::fs::write(&outcome_file, new_data)?;
    println!("✅ Marked '{}' as false positive", kept_name);

    Ok(())
}

fn run_stats(path: &Path, detailed: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Outcome Statistics for: {:?}", path);
    println!("");

    let outcome_file = path.join(".code-intelligence-outcomes.json");
    if !outcome_file.exists() {
        println!("No tracked outcomes found.");
        return Ok(());
    }

    let data = std::fs::read_to_string(&outcome_file)?;
    let outcomes: Vec<serde_json::Value> = serde_json::from_str(&data)?;

    let total = outcomes.len();
    let removed = outcomes
        .iter()
        .filter(|v| v.get("outcome").and_then(|o| o.as_str()).unwrap_or("") == "Removed")
        .count();
    let kept = outcomes
        .iter()
        .filter(|v| v.get("outcome").and_then(|o| o.as_str()).unwrap_or("") == "Kept")
        .count();
    let pending = outcomes
        .iter()
        .filter(|v| v.get("outcome").and_then(|o| o.as_str()).unwrap_or("") == "Pending")
        .count();

    let removal_rate = if total > 0 {
        removed as f64 / total as f64
    } else {
        0.0
    };

    println!("📊 Summary:");
    println!("   Total flagged: {}", total);
    println!("   Removed: {} ({:.1}%)", removed, removal_rate * 100.0);
    println!("   Kept (false positives): {}", kept);
    println!("   Pending: {}", pending);

    if detailed {
        // Show per-project breakdown if available
        if let Some(project_config) = get_project_config(path) {
            println!("\n📁 Project Info:");
            if let Some(pt) = project_config.project_type {
                println!("   Type: {}", pt);
            }
            if let Some(t) = project_config.threshold {
                println!("   Threshold: {:.2}", t);
            }
            if let Some(la) = project_config.last_analyzed {
                println!("   Last Analyzed: {}", la);
            }
        }
    }

    if pending > 0 {
        println!(
            "\n💡 {} functions waiting for review. Run `ci list` to see them.",
            pending
        );
    } else if total > 0 {
        println!("\n✅ All functions reviewed!");
    }

    Ok(())
}

fn run_report(
    path: &Path,
    format: &str,
    output: Option<PathBuf>,
    llm: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📄 Generating report for: {:?}", path);
    println!("   Format: {}", format);
    if llm {
        println!("   LLM analysis: enabled");
    }

    let output_file = output.unwrap_or_else(|| {
        let ext = match format {
            "json" => "json",
            "html" => "html",
            "full" => "md",
            _ => "md",
        };
        PathBuf::from(format!("code_analysis.{}", ext))
    });

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--release", "--"]);
    cmd.arg(path)
        .args(["--format", format])
        .args(["--output", &output_file.to_string_lossy()]);

    if llm {
        cmd.arg("--llm");
    }

    let status = cmd.status()?;

    if status.success() {
        println!("\n✅ Report saved to: {:?}", output_file);
    } else {
        eprintln!("\n❌ Report generation failed");
    }

    Ok(())
}

fn run_train(
    data: &Path,
    val_data: Option<&Path>,
    output: &Path,
    precision: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Training model...");
    println!("   Training data: {:?}", data);
    if let Some(vd) = val_data {
        println!("   Validation data: {:?}", vd);
    }
    println!("   Output: {:?}", output);
    println!("   Target precision: {:.2}", precision);
    println!("");

    let mut cmd = Command::new("train_model");
    cmd.args(["--train-data", &data.to_string_lossy()])
        .args(["--output", &output.to_string_lossy()])
        .args(["--target-precision", &precision.to_string()]);

    if let Some(vd) = val_data {
        cmd.args(["--val-data", &vd.to_string_lossy()]);
    }

    let status = cmd.status()?;

    if status.success() {
        println!("\n✅ Model trained and saved to: {:?}", output);
        println!("   Run `ci calibrate` to calibrate the model.");
    } else {
        eprintln!("\n❌ Training failed");
    }

    Ok(())
}

fn run_calibrate(
    model: &Path,
    data: &Path,
    output: &Path,
    method: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Calibrating model...");
    println!("   Model: {:?}", model);
    println!("   Data: {:?}", data);
    println!("   Method: {}", method);
    println!("   Output: {:?}", output);
    println!("");

    let mut cmd = Command::new("calibrate_model");
    cmd.args(["--model", &model.to_string_lossy()])
        .args(["--val-data", &data.to_string_lossy()])
        .args(["--output", &output.to_string_lossy()])
        .args(["--method", method]);

    let status = cmd.status()?;

    if status.success() {
        println!("\n✅ Model calibrated and saved to: {:?}", output);
    } else {
        eprintln!("\n❌ Calibration failed");
    }

    Ok(())
}

fn run_tune(model: &Path, data: &Path, precision: f64) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Tuning threshold...");
    println!("   Model: {:?}", model);
    println!("   Data: {:?}", data);
    println!("   Target precision: {:.2}", precision);
    println!("");

    let mut cmd = Command::new("tune_threshold");
    cmd.args(["--model", &model.to_string_lossy()])
        .args(["--val-data", &data.to_string_lossy()])
        .args(["--target-precision", &precision.to_string()]);

    let status = cmd.status()?;

    if status.success() {
        println!("\n✅ Threshold tuning complete!");
    } else {
        eprintln!("\n❌ Threshold tuning failed");
    }

    Ok(())
}

fn run_compare(
    train_data: &Path,
    val_data: &Path,
    test_data: &Path,
    output: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Comparing ML models...");
    println!("   Train: {:?}", train_data);
    println!("   Validation: {:?}", val_data);
    println!("   Test: {:?}", test_data);
    println!("   Output: {:?}", output);
    println!("");

    let mut cmd = Command::new("model_comparison");
    cmd.args(["--train-data", &train_data.to_string_lossy()])
        .args(["--val-data", &val_data.to_string_lossy()])
        .args(["--test-data", &test_data.to_string_lossy()])
        .args(["--output-dir", &output.to_string_lossy()]);

    let status = cmd.status()?;

    if status.success() {
        println!("\n✅ Model comparison saved to: {:?}", output);
    } else {
        eprintln!("\n❌ Model comparison failed");
    }

    Ok(())
}

fn run_features(data: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Analyzing feature importance...");
    println!("   Data: {:?}", data);
    println!("");

    let mut cmd = Command::new("analyze_features_per_language");
    cmd.arg(data);

    let status = cmd.status()?;

    if status.success() {
        println!("\n✅ Feature analysis complete!");
    } else {
        eprintln!("\n❌ Feature analysis failed");
    }

    Ok(())
}

fn run_export(path: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Exporting training data from: {:?}", path);
    println!("   Output: {:?}", output);
    println!("");

    let mut cmd = Command::new("training_data_exporter");
    cmd.arg(path).arg(output);

    let status = cmd.status()?;

    if status.success() {
        println!("\n✅ Training data exported to: {:?}", output);
    } else {
        eprintln!("\n❌ Export failed");
    }

    Ok(())
}

fn run_merge(input: &str, output: &Path, dedup: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Merging training data...");
    println!("   Input: {}", input);
    println!("   Output: {:?}", output);
    if dedup {
        println!("   Deduplication: enabled");
    }
    println!("");

    // If input is a glob pattern, use the merge_all_training_data binary
    let mut cmd = Command::new("merge_all_training_data");
    cmd.arg("--output").arg(output);
    if dedup {
        cmd.arg("--dedup");
    }

    let status = cmd.status()?;

    if status.success() {
        println!("\n✅ Training data merged to: {:?}", output);
    } else {
        eprintln!("\n❌ Merge failed");
    }

    Ok(())
}

fn run_dashboard(path: &Path, model: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Opening dashboard for: {:?}", path);
    println!("   Press 'q' to quit");
    println!("");

    let mut cmd = Command::new("dead_code_dashboard");
    cmd.arg(path);

    if let Some(m) = model {
        cmd.args(["--model", &m.to_string_lossy()]);
    }

    let status = cmd.status()?;

    if status.success() {
        println!("\n✅ Dashboard closed");
    } else {
        eprintln!("\n❌ Dashboard failed to start");
    }

    Ok(())
}

fn run_self_analyze(
    format: &str,
    output: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Analyzing code-intelligence itself...");
    println!("   Format: {}", format);
    println!("");

    // Find the current directory (should be code-intelligence)
    let current_dir = std::env::current_dir()?;

    let output_file = output.unwrap_or_else(|| {
        let ext = match format {
            "json" => "json",
            "full" => "md",
            _ => "md",
        };
        PathBuf::from(format!("self_analysis.{}", ext))
    });

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--release", "--"]);
    cmd.arg(&current_dir)
        .args(["--format", format])
        .args(["--output", &output_file.to_string_lossy()]);

    let status = cmd.status()?;

    if status.success() {
        println!("\n✅ Self-analysis saved to: {:?}", output_file);
    } else {
        eprintln!("\n❌ Self-analysis failed");
    }

    Ok(())
}

fn run_analyze_features(data: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Analyzing features per language...");
    println!("   Data: {:?}", data);
    println!("");

    let mut cmd = Command::new("analyze_features_per_language");
    cmd.arg(data);

    let status = cmd.status()?;

    if status.success() {
        println!("\n✅ Feature analysis complete!");
    } else {
        eprintln!("\n❌ Feature analysis failed");
    }

    Ok(())
}

fn run_evaluate(
    model: &Path,
    test_data: &Path,
    val_data: Option<&Path>,
    detailed: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Evaluating model...");
    println!("   Model: {:?}", model);
    println!("   Test data: {:?}", test_data);
    if let Some(vd) = val_data {
        println!("   Validation data: {:?}", vd);
    }
    if detailed {
        println!("   Detailed metrics: enabled");
    }
    println!("");

    let mut cmd = Command::new("evaluate_per_language");
    cmd.args(["--model", &model.to_string_lossy()])
        .args(["--test-data", &test_data.to_string_lossy()]);

    if let Some(vd) = val_data {
        cmd.args(["--val-data", &vd.to_string_lossy()]);
    }
    if detailed {
        cmd.arg("--detailed");
    }

    let status = cmd.status()?;

    if status.success() {
        println!("\n✅ Evaluation complete!");
    } else {
        eprintln!("\n❌ Evaluation failed");
    }

    Ok(())
}

/// Find a binary in the PATH or standard cargo install locations
fn find_binary(name: &str) -> Option<PathBuf> {
    // Check current directory
    if let Ok(cwd) = std::env::current_dir() {
        let local_path = cwd.join(name);
        if local_path.exists() {
            return Some(local_path);
        }
    }

    // Check ~/.cargo/bin/ (standard cargo install location)
    if let Ok(home) = std::env::var("HOME") {
        let cargo_bin = PathBuf::from(home).join(".cargo/bin").join(name);
        if cargo_bin.exists() {
            return Some(cargo_bin);
        }
    }

    // Check PATH
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            if dir.is_empty() {
                continue;
            }
            let full_path = PathBuf::from(dir).join(name);
            if full_path.exists() {
                return Some(full_path);
            }
        }
    }

    // Check where ci binary is installed
    if let Ok(exe) = std::env::current_exe() {
        let fallback = PathBuf::from(".");
        let exe_dir = exe.parent().unwrap_or(&fallback);
        let local_path = exe_dir.join(name);
        if local_path.exists() {
            return Some(local_path);
        }
    }

    None
}

fn run_config(action: ConfigAction) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = load_config();

    match action {
        ConfigAction::Set { key, value } => match key.as_str() {
            "model" => {
                config.defaults.model = Some(value.clone());
                save_config(&config)?;
                println!("✅ Model set to: {}", value);
            }
            "threshold" => {
                let threshold = value
                    .parse::<f64>()
                    .map_err(|_| format!("Invalid threshold: {}", value))?;
                config.defaults.threshold = Some(threshold);
                save_config(&config)?;
                println!("✅ Threshold set to: {:.2}", threshold);
            }
            "verbose" => {
                let verbose = value
                    .parse::<bool>()
                    .map_err(|_| format!("Invalid boolean: {}", value))?;
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
                println!("   Available keys: model, threshold, verbose, llm_provider, llm_model");
            }
        },
        ConfigAction::Get { key } => match key.as_str() {
            "model" => {
                if let Some(model) = &config.defaults.model {
                    println!("{}", model);
                } else {
                    println!("(not set)");
                }
            }
            "threshold" => {
                if let Some(threshold) = config.defaults.threshold {
                    println!("{:.2}", threshold);
                } else {
                    println!("(not set)");
                }
            }
            "verbose" => {
                println!("{}", config.defaults.verbose);
            }
            "llm_provider" => {
                if let Some(provider) = &config.defaults.llm_provider {
                    println!("{}", provider);
                } else {
                    println!("(not set)");
                }
            }
            "llm_model" => {
                if let Some(model) = &config.defaults.llm_model {
                    println!("{}", model);
                } else {
                    println!("(not set)");
                }
            }
            _ => {
                println!("⚠️ Unknown config key: {}", key);
            }
        },
        ConfigAction::List => {
            println!("📋 Current Configuration:");
            println!("");
            println!("[defaults]");
            println!("  model = {:?}", config.defaults.model);
            println!("  threshold = {:?}", config.defaults.threshold);
            println!("  verbose = {}", config.defaults.verbose);
            println!("  llm_provider = {:?}", config.defaults.llm_provider);
            println!("  llm_model = {:?}", config.defaults.llm_model);
            println!("");
            if !config.projects.is_empty() {
                println!("[projects]");
                for (key, project) in &config.projects {
                    println!("  {} = {{", key);
                    println!("    path = {:?}", project.path);
                    if let Some(pt) = &project.project_type {
                        println!("    type = {:?}", pt);
                    }
                    if let Some(t) = project.threshold {
                        println!("    threshold = {:.2}", t);
                    }
                    if let Some(dc) = project.dead_count {
                        println!("    dead_count = {}", dc);
                    }
                    if let Some(la) = &project.last_analyzed {
                        println!("    last_analyzed = {:?}", la);
                    }
                    println!("  }}");
                }
            }
        }
    }

    Ok(())
}
