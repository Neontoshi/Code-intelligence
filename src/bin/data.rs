// src/bin/data.rs

//! Unified data management tool with subcommands
//!
//! Usage:
//!   data export <path> [--output <path>]
//!   data merge [--input <glob>] [--output <path>] [--dedup]
//!   data collect [--repos <urls>] [--output <dir>] [--max-repos <n>]

use clap::{Parser, Subcommand};
use code_intelligence::error::{err, Result};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about = "Training data management toolkit")]
struct Args {
    #[command(subcommand)]
    command: DataCommand,
}

#[derive(Subcommand, Debug)]
enum DataCommand {
    /// Export training data from a project
    Export {
        /// Path to analyze
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Output file
        #[arg(short, long, default_value = "training_data.json")]
        output: PathBuf,
    },
    /// Merge training data files
    Merge {
        /// Input files (glob pattern)
        #[arg(short, long, default_value = "training_data/*.json")]
        input: String,
        /// Output file
        #[arg(short, long, default_value = "combined_training.json")]
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
        #[arg(short, long, default_value = "training_data")]
        output: PathBuf,
        /// Max repos to process
        #[arg(long, default_value = "50")]
        max_repos: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        DataCommand::Export { path, output } => {
            run_export(&path, &output).await?;
        }
        DataCommand::Merge {
            input,
            output,
            dedup,
        } => {
            run_merge(&input, &output, dedup)?;
        }
        DataCommand::Collect {
            repos,
            output,
            max_repos,
        } => {
            run_collect(&repos, &output, max_repos).await?;
        }
    }

    Ok(())
}

async fn run_export(path: &Path, output: &Path) -> Result<()> {
    use code_intelligence::analysis::roots::{
        ReachabilityAnalyzer, RootDetectionConfig, RootDetector,
    };
    use code_intelligence::analysis::training_data::{TrainingDataCollector, TrainingLabel};
    use code_intelligence::Pipeline;

    println!("📊 Exporting training data from: {:?}", path);

    let mut pipeline = Pipeline::new();
    let analysis = pipeline.process_project(path).await?;

    let root_config = RootDetectionConfig::default();
    let root_set = RootDetector::detect_roots(&analysis.call_graph, &analysis.files, &root_config);
    let reachability = ReachabilityAnalyzer::compute_reachability(&analysis.call_graph, &root_set);

    let mut collector = TrainingDataCollector::new();

    for idx in analysis.call_graph.node_indices() {
        let func = &analysis.call_graph[idx];
        let full_path = &func.full_path;

        let is_reachable = reachability.is_reachable(full_path);
        let has_callers = func.fan_in > 0;

        if is_reachable || has_callers {
            collector.add_high_confidence_example(
                func,
                &analysis.call_graph,
                TrainingLabel::Alive,
                0.90,
                "reachable",
            );
        } else if func.is_public {
            // Public API exports without callers in libraries are Alive library entrypoints
            collector.add_high_confidence_example(
                func,
                &analysis.call_graph,
                TrainingLabel::Alive,
                0.80,
                "public_export",
            );
        } else if func.fan_in == 0 {
            collector.add_high_confidence_example(
                func,
                &analysis.call_graph,
                TrainingLabel::Dead,
                0.85,
                "unreachable_private",
            );
        }
    }

    let json = collector
        .to_json()
        .map_err(|e| err::internal(e.to_string()))?;
    std::fs::write(output, json)?;

    println!("✅ Training data exported to: {:?}", output);
    println!("   Examples: {}", collector.examples.len());

    Ok(())
}

fn run_merge(input: &str, output: &Path, dedup: bool) -> Result<()> {
    use code_intelligence::analysis::training_data::TrainingExample;

    println!("📊 Merging training data...");
    println!("   Input pattern: {}", input);

    use std::fs;
    let mut all_examples = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for entry in fs::read_dir(input)
        .map_err(|e| err::internal(format!("Failed to read directory: {}", e)))?
    {
        let path = entry
            .map_err(|e| err::internal(format!("Failed to read entry: {}", e)))?
            .path();

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

        let output_file = output.join(format!("{}.json", repo_name));
        let _ = run_export(&repo_dir, &output_file).await;
        count += 1;
    }

    println!("✅ Processed {} repositories", count);
    Ok(())
}
