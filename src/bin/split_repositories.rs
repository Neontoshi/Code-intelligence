//! Repository-level train/validation/test split
//!
//! This tool loads all repository files from data/raw/jsonl/
//! and splits them cleanly at the repository level to prevent data leakage.

use clap::Parser;
use code_intelligence::analysis::training_data::{TrainingExample, TrainingLabel};
use code_intelligence::error::Result;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about = "Split training data at repository level")]
struct Args {
    /// Input directory containing individual .jsonl files per repo
    #[arg(short, long, default_value = "data/raw/jsonl")]
    input_dir: PathBuf,

    /// Output directory for splits
    #[arg(short, long, default_value = "data")]
    output_dir: PathBuf,

    /// Train split ratio (0.0 - 1.0)
    #[arg(long, default_value = "0.70")]
    train_ratio: f64,

    /// Validation split ratio (0.0 - 1.0)
    #[arg(long, default_value = "0.15")]
    val_ratio: f64,

    /// Test split ratio (0.0 - 1.0)
    #[arg(long, default_value = "0.15")]
    test_ratio: f64,

    /// Seed for reproducibility
    #[arg(long, default_value = "42")]
    seed: u64,

    /// Minimum examples per repository to include
    #[arg(long, default_value = "5")]
    min_examples: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitStats {
    pub train_repos: usize,
    pub val_repos: usize,
    pub test_repos: usize,
    pub train_examples: usize,
    pub val_examples: usize,
    pub test_examples: usize,
    pub train_alive: usize,
    pub train_dead: usize,
    pub val_alive: usize,
    pub val_dead: usize,
    pub test_alive: usize,
    pub test_dead: usize,
    pub repo_names: RepoList,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoList {
    pub train: Vec<String>,
    pub val: Vec<String>,
    pub test: Vec<String>,
}

fn load_repo_jsonl(path: &Path, repo_name: &str) -> Vec<TrainingExample> {
    let mut examples = Vec::new();
    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            if let Ok(mut ex) = serde_json::from_str::<TrainingExample>(&line) {
                ex.repository_id = Some(repo_name.to_string());
                examples.push(ex);
            }
        }
    }
    examples
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("📊 Loading repositories from: {:?}", args.input_dir);
    let mut by_repo: HashMap<String, Vec<TrainingExample>> = HashMap::new();

    if let Ok(entries) = std::fs::read_dir(&args.input_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let examples = load_repo_jsonl(&path, stem);
                    if examples.len() >= args.min_examples {
                        by_repo.insert(stem.to_string(), examples);
                    }
                }
            }
        }
    }

    let mut repos: Vec<(String, usize)> = by_repo
        .iter()
        .map(|(name, ex)| (name.clone(), ex.len()))
        .collect();

    println!("   Repositories found: {}", repos.len());
    if repos.is_empty() {
        eprintln!("❌ No repositories found in {:?}", args.input_dir);
        std::process::exit(1);
    }

    let mut rng = rand::rngs::StdRng::seed_from_u64(args.seed);
    repos.shuffle(&mut rng);

    let total_repos = repos.len();
    let train_count = (total_repos as f64 * args.train_ratio).round() as usize;
    let val_count = (total_repos as f64 * args.val_ratio).round() as usize;
    let test_count = total_repos - train_count - val_count;

    let train_repos: Vec<String> = repos.iter().take(train_count).map(|(n, _)| n.clone()).collect();
    let val_repos: Vec<String> = repos.iter().skip(train_count).take(val_count).map(|(n, _)| n.clone()).collect();
    let test_repos: Vec<String> = repos.iter().skip(train_count + val_count).take(test_count).map(|(n, _)| n.clone()).collect();

    let mut train_examples = Vec::new();
    let mut val_examples = Vec::new();
    let mut test_examples = Vec::new();

    for repo in &train_repos {
        if let Some(mut ex) = by_repo.remove(repo) {
            for e in &mut ex { e.dataset_split = Some("train".to_string()); }
            train_examples.extend(ex);
        }
    }

    for repo in &val_repos {
        if let Some(mut ex) = by_repo.remove(repo) {
            for e in &mut ex { e.dataset_split = Some("val".to_string()); }
            val_examples.extend(ex);
        }
    }

    for repo in &test_repos {
        if let Some(mut ex) = by_repo.remove(repo) {
            for e in &mut ex { e.dataset_split = Some("test".to_string()); }
            test_examples.extend(ex);
        }
    }

    let train_alive = train_examples.iter().filter(|e| e.label == TrainingLabel::Alive).count();
    let train_dead = train_examples.iter().filter(|e| e.label == TrainingLabel::Dead).count();
    let val_alive = val_examples.iter().filter(|e| e.label == TrainingLabel::Alive).count();
    let val_dead = val_examples.iter().filter(|e| e.label == TrainingLabel::Dead).count();
    let test_alive = test_examples.iter().filter(|e| e.label == TrainingLabel::Alive).count();
    let test_dead = test_examples.iter().filter(|e| e.label == TrainingLabel::Dead).count();

    std::fs::create_dir_all(&args.output_dir)?;

    std::fs::write(args.output_dir.join("train.json"), serde_json::to_string_pretty(&train_examples)?)?;
    std::fs::write(args.output_dir.join("val.json"), serde_json::to_string_pretty(&val_examples)?)?;
    std::fs::write(args.output_dir.join("test.json"), serde_json::to_string_pretty(&test_examples)?)?;

    let stats = SplitStats {
        train_repos: train_repos.len(),
        val_repos: val_repos.len(),
        test_repos: test_repos.len(),
        train_examples: train_examples.len(),
        val_examples: val_examples.len(),
        test_examples: test_examples.len(),
        train_alive,
        train_dead,
        val_alive,
        val_dead,
        test_alive,
        test_dead,
        repo_names: RepoList {
            train: train_repos,
            val: val_repos,
            test: test_repos,
        },
    };

    std::fs::write(args.output_dir.join("split_stats.json"), serde_json::to_string_pretty(&stats)?)?;

    println!("\n📊 Split Summary:");
    println!("   Train: {} repos, {} examples (Alive: {}, Dead: {})", stats.train_repos, stats.train_examples, stats.train_alive, stats.train_dead);
    println!("   Val:   {} repos, {} examples (Alive: {}, Dead: {})", stats.val_repos, stats.val_examples, stats.val_alive, stats.val_dead);
    println!("   Test:  {} repos, {} examples (Alive: {}, Dead: {})", stats.test_repos, stats.test_examples, stats.test_alive, stats.test_dead);
    println!("\n✅ Splits saved to: {:?}", args.output_dir);

    Ok(())
}
