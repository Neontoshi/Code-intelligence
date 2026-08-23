// src/bin/split_repositories.rs

//! Repository-level train/validation/test split
//!
//! This tool ensures that all functions from the same repository
//! stay together in the same split (train/val/test) to prevent
//! data leakage.

use clap::Parser;
use code_intelligence::analysis::training_data::TrainingExample;
use code_intelligence::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "Split training data at repository level")]
struct Args {
    /// Input JSON file containing all training examples
    #[arg(short, long, default_value = "combined_training.json")]
    input: PathBuf,

    /// Output directory for splits
    #[arg(short, long, default_value = "data")]
    output_dir: PathBuf,

    /// Train split ratio (0.0 - 1.0)
    #[arg(long, default_value = "0.7")]
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

fn main() -> Result<()> {
    let args = Args::parse();

    // Validate ratios
    let total_ratio = args.train_ratio + args.val_ratio + args.test_ratio;
    if (total_ratio - 1.0).abs() > 0.001 {
        eprintln!("⚠️ Ratios sum to {}, not 1.0. Normalizing...", total_ratio);
    }

    // Load data
    println!("📊 Loading training data from: {:?}", args.input);
    let data = std::fs::read_to_string(&args.input)?;
    let examples: Vec<TrainingExample> = serde_json::from_str(&data)?;

    println!("   Total examples: {}", examples.len());

    // Group by repository
    let mut by_repo: HashMap<String, Vec<TrainingExample>> = HashMap::new();
    for example in examples {
        let repo_id = example
            .repository_id
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        by_repo.entry(repo_id).or_default().push(example);
    }

    // Filter repos with too few examples
    let repos: Vec<(String, Vec<TrainingExample>)> = by_repo
        .into_iter()
        .filter(|(_, examples)| examples.len() >= args.min_examples)
        .collect();

    println!("   Repositories found: {}", repos.len());

    if repos.is_empty() {
        eprintln!(
            "❌ No repositories with enough examples (min: {})",
            args.min_examples
        );
        std::process::exit(1);
    }

    // Sort repos by size for stratified split
    let mut sorted_repos: Vec<(String, usize)> = repos
        .iter()
        .map(|(name, examples)| (name.clone(), examples.len()))
        .collect();
    sorted_repos.sort_by(|a, b| b.1.cmp(&a.1));

    // Calculate split sizes
    let total_repos = sorted_repos.len();
    let train_count = (total_repos as f64 * args.train_ratio).round() as usize;
    let val_count = (total_repos as f64 * args.val_ratio).round() as usize;
    let test_count = total_repos - train_count - val_count;

    println!("\n📊 Split sizes:");
    println!("   Train: {} repos", train_count);
    println!("   Val: {} repos", val_count);
    println!("   Test: {} repos", test_count);

    // Shuffle with seed for reproducibility
    use rand::seq::SliceRandom;
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(args.seed);

    // Create splits
    let mut shuffled_repos = sorted_repos;
    shuffled_repos.shuffle(&mut rng);

    let train_repos: Vec<String> = shuffled_repos
        .iter()
        .take(train_count)
        .map(|(name, _)| name.clone())
        .collect();

    let val_repos: Vec<String> = shuffled_repos
        .iter()
        .skip(train_count)
        .take(val_count)
        .map(|(name, _)| name.clone())
        .collect();

    let test_repos: Vec<String> = shuffled_repos
        .iter()
        .skip(train_count + val_count)
        .take(test_count)
        .map(|(name, _)| name.clone())
        .collect();

    // Build split datasets
    let mut train_examples = Vec::new();
    let mut val_examples = Vec::new();
    let mut test_examples = Vec::new();

    let repo_map: HashMap<String, Vec<TrainingExample>> = repos.into_iter().collect();

    for repo in &train_repos {
        if let Some(examples) = repo_map.get(repo) {
            let mut cloned = examples.clone();
            for example in &mut cloned {
                example.dataset_split = Some("train".to_string());
            }
            train_examples.extend(cloned);
        }
    }

    for repo in &val_repos {
        if let Some(examples) = repo_map.get(repo) {
            let mut cloned = examples.clone();
            for example in &mut cloned {
                example.dataset_split = Some("val".to_string());
            }
            val_examples.extend(cloned);
        }
    }

    for repo in &test_repos {
        if let Some(examples) = repo_map.get(repo) {
            let mut cloned = examples.clone();
            for example in &mut cloned {
                example.dataset_split = Some("test".to_string());
            }
            test_examples.extend(cloned);
        }
    }

    // Count labels
    use code_intelligence::analysis::training_data::TrainingLabel;

    let train_alive = train_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Alive)
        .count();
    let train_dead = train_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Dead)
        .count();

    let val_alive = val_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Alive)
        .count();
    let val_dead = val_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Dead)
        .count();

    let test_alive = test_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Alive)
        .count();
    let test_dead = test_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Dead)
        .count();

    // Create output directory
    std::fs::create_dir_all(&args.output_dir)?;

    // Save splits
    let train_path = args.output_dir.join("train.json");
    let val_path = args.output_dir.join("val.json");
    let test_path = args.output_dir.join("test.json");

    std::fs::write(&train_path, serde_json::to_string_pretty(&train_examples)?)?;
    std::fs::write(&val_path, serde_json::to_string_pretty(&val_examples)?)?;
    std::fs::write(&test_path, serde_json::to_string_pretty(&test_examples)?)?;

    // Also save as JSONL for streaming
    let train_jsonl: String = train_examples
        .iter()
        .filter_map(|e| serde_json::to_string(e).ok())
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(args.output_dir.join("train.jsonl"), train_jsonl)?;

    // Save stats
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

    let stats_path = args.output_dir.join("split_stats.json");
    std::fs::write(&stats_path, serde_json::to_string_pretty(&stats)?)?;

    // Print summary
    println!("\n📊 Split Summary:");
    println!(
        "   Train: {} repos, {} examples (Alive: {}, Dead: {})",
        stats.train_repos, stats.train_examples, stats.train_alive, stats.train_dead
    );
    println!(
        "   Val:   {} repos, {} examples (Alive: {}, Dead: {})",
        stats.val_repos, stats.val_examples, stats.val_alive, stats.val_dead
    );
    println!(
        "   Test:  {} repos, {} examples (Alive: {}, Dead: {})",
        stats.test_repos, stats.test_examples, stats.test_alive, stats.test_dead
    );

    println!("\n✅ Splits saved to: {:?}", args.output_dir);
    println!("   - train.json ({} examples)", stats.train_examples);
    println!("   - val.json ({} examples)", stats.val_examples);
    println!("   - test.json ({} examples)", stats.test_examples);

    // Check for label leakage
    println!("\n🔍 Label Distribution Check:");
    let train_ratio = if stats.train_examples > 0 {
        stats.train_dead as f64 / stats.train_examples as f64
    } else {
        0.0
    };
    let val_ratio = if stats.val_examples > 0 {
        stats.val_dead as f64 / stats.val_examples as f64
    } else {
        0.0
    };
    let test_ratio = if stats.test_examples > 0 {
        stats.test_dead as f64 / stats.test_examples as f64
    } else {
        0.0
    };

    println!(
        "   Dead ratio - Train: {:.1}%, Val: {:.1}%, Test: {:.1}%",
        train_ratio * 100.0,
        val_ratio * 100.0,
        test_ratio * 100.0
    );

    let max_diff = (train_ratio - val_ratio)
        .abs()
        .max((train_ratio - test_ratio).abs());
    if max_diff > 0.1 {
        println!(
            "   ⚠️ Warning: Large label distribution difference between splits (max diff: {:.1}%)",
            max_diff * 100.0
        );
        println!("   Consider using stratified sampling for better balance.");
    } else {
        println!("   ✅ Label distributions are well balanced across splits.");
    }

    Ok(())
}
