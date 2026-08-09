//! Code Intelligence CLI - Global dead code detection tool
//!
//! Usage:
//!   ci analyze [path]        - Analyze current or specified project
//!   ci list [path]           - List dead functions found
//!   ci remove <name>         - Mark a function as removed
//!   ci keep <name> "reason"  - Mark as false positive
//!   ci stats [path]          - Show outcome statistics
//!   ci report [path]         - Generate report
//!   ci config set <key> <value>
//!   ci config get <key>
//!   ci config list

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

// ============================================================================
// Config
// ============================================================================

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
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            model: None,
            threshold: None,
            verbose: false,
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

// ============================================================================
// CLI Arguments
// ============================================================================

#[derive(Parser, Debug)]
#[command(
    name = "ci",
    author = "Code Intelligence Team",
    version = "0.1.0",
    about = "Code Intelligence - Dead code detection for any project",
    long_about = "A global CLI tool for detecting and managing dead code across multiple projects."
)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Analyze current or specified project
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
    },

    /// List dead functions in current or specified project
    List {
        /// Path to list (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
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

    /// Show outcome statistics for current or specified project
    Stats {
        /// Path (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Generate report for current or specified project
    Report {
        /// Path (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Output format (markdown, json, html)
        #[arg(long, default_value = "markdown")]
        format: String,
        /// Output file (defaults to stdout)
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Configure global settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Train the model with new data
    Train {
        /// Additional data path (optional)
        #[arg(default_value = "data/train.json")]
        data: PathBuf,
        /// Output model path
        #[arg(long, default_value = "model.bin")]
        output: PathBuf,
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

// ============================================================================
// Main
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Commands::Analyze {
            path,
            threshold,
            verbose,
        } => {
            let project_path = resolve_path(&path)?;
            run_analyze(&project_path, threshold, verbose)?;
        }
        Commands::List { path } => {
            let project_path = resolve_path(&path)?;
            run_list(&project_path)?;
        }
        Commands::Remove { name, commit, path } => {
            let project_path = resolve_path(&path)?;
            run_remove(&project_path, &name, commit)?;
        }
        Commands::Keep { name, reason, path } => {
            let project_path = resolve_path(&path)?;
            run_keep(&project_path, &name, &reason)?;
        }
        Commands::Stats { path } => {
            let project_path = resolve_path(&path)?;
            run_stats(&project_path)?;
        }
        Commands::Report {
            path,
            format,
            output,
        } => {
            let project_path = resolve_path(&path)?;
            run_report(&project_path, &format, output)?;
        }
        Commands::Config { action } => {
            run_config(action)?;
        }
        Commands::Train { data, output } => {
            run_train(&data, &output)?;
        }
    }

    Ok(())
}

// ============================================================================
// Config Helper Functions
// ============================================================================

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

    if path.join("Cargo.toml").exists() {
        Some("rust".to_string())
    } else if path.join("package.json").exists() {
        // Check if TypeScript
        if path.join("tsconfig.json").exists() {
            Some("typescript".to_string())
        } else {
            Some("javascript".to_string())
        }
    } else if path.join("go.mod").exists() {
        Some("go".to_string())
    } else if path.join("pom.xml").exists() || path.join("build.gradle").exists() {
        Some("java".to_string())
    } else if path.join("requirements.txt").exists() || path.join("pyproject.toml").exists() {
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

// ============================================================================
// Command Implementations
// ============================================================================

fn run_analyze(
    path: &Path,
    threshold: Option<f64>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Analyzing project: {:?}", path);

    // Detect project type
    let project_type = detect_project_type(path);
    if let Some(pt) = &project_type {
        println!("📊 Detected project type: {}", pt);
    } else {
        println!("⚠️ Could not detect project type (will try auto-detection)");
    }

    // Get config values
    let model = get_default_model();
    if model.is_none() {
        println!("⚠️ No model configured. Run: ci config set model <path>");
        println!("   Or train a model: ci train");
        return Ok(());
    }

    let threshold = threshold
        .or(get_project_config(path).and_then(|c| c.threshold))
        .unwrap_or_else(get_default_threshold);
    let model_path = model.unwrap();

    // Build command
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--release", "--bin", "dead_code_check"])
        .arg(path)
        .args(["--model", &model_path])
        .args(["--threshold", &format!("{:.2}", threshold)]);

    if verbose {
        cmd.arg("--verbose");
    }

    println!("📊 Using threshold: {:.2}", threshold);
    println!("🤖 Using model: {}", model_path);
    println!("");

    // Run and capture output
    let status = cmd.status()?;

    if status.success() {
        // Update project config
        let project_config = ProjectConfig {
            path: path.to_string_lossy().to_string(),
            project_type: project_type,
            threshold: Some(threshold),
            last_analyzed: Some(chrono::Local::now().to_string()),
            dead_count: None, // Will be updated by parsing output
        };
        let _ = save_project_config(path, project_config);
        println!("\n✅ Analysis complete!");
    } else {
        eprintln!("\n❌ Analysis failed");
    }

    Ok(())
}

fn run_list(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Listing dead functions in: {:?}", path);

    let outcome_file = path.join(".code-intelligence-outcomes.json");
    if !outcome_file.exists() {
        println!("No tracked outcomes found in this project.");
        println!("Run `ci analyze` first to find dead code.");
        return Ok(());
    }

    let data = std::fs::read_to_string(&outcome_file)?;
    let outcomes: Vec<serde_json::Value> = serde_json::from_str(&data)?;

    if outcomes.is_empty() {
        println!("No tracked verdicts.");
        return Ok(());
    }

    // Filter pending
    let pending: Vec<_> = outcomes
        .iter()
        .filter(|v| v.get("outcome").and_then(|o| o.as_str()).unwrap_or("") == "Pending")
        .collect();

    if pending.is_empty() {
        println!("✅ No pending verdicts! All tracked functions have been reviewed.");
        return Ok(());
    }

    println!("\n📋 Pending Dead Functions ({} total):", pending.len());
    println!("   (Use `ci remove <name>` or `ci keep <name> \"reason\"`)");
    println!("");
    println!("| # | Function | Confidence | File |");
    println!("|---|----------|------------|------|");

    for (i, v) in pending.iter().enumerate() {
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

        // Get the short name (first 30 chars)
        let short_name = if name.len() > 30 { &name[..30] } else { name };
        println!(
            "| {} | {} | {:.1}% | {} |",
            i + 1,
            short_name,
            confidence,
            file
        );
    }

    println!("\n💡 To manage: ci remove <function-name> or ci keep <function-name> \"reason\"");

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
            // Store name before modifying
            removed_name = func_name.to_string();

            // Now modify the outcome (no borrow conflicts)
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
            // Store name before modifying
            kept_name = func_name.to_string();

            // Now modify the outcome (no borrow conflicts)
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

fn run_stats(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
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
    _output: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📄 Generating report for: {:?}", path);
    println!("   Format: {}", format);

    // This would call the existing report generation
    println!("⚠️ Report generation coming soon!");
    println!(
        "   Run `cargo run --bin dead_code_check {} --format {} --output ...`",
        path.display(),
        format
    );

    Ok(())
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
            _ => {
                println!("⚠️ Unknown config key: {}", key);
                println!("   Available keys: model, threshold, verbose");
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

fn run_train(_data: &Path, _output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Training model...");
    println!("⚠️ Training command coming soon!");
    println!(
        "   Run `cargo run --release --bin train_model -- --train-data {} --output {}`",
        _data.display(),
        _output.display()
    );
    Ok(())
}
