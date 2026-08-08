// src/bin/merge_all_training_data.rs

use code_intelligence::analysis::training_data::TrainingExample;
use std::collections::HashMap;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let training_dir = PathBuf::from("training_data");

    // Create output directory if it doesn't exist
    std::fs::create_dir_all("data")?;

    // Load all examples grouped by repository
    let mut by_repo: HashMap<String, Vec<TrainingExample>> = HashMap::new();

    println!("📊 Loading training data from: {:?}", training_dir);

    for entry in std::fs::read_dir(&training_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            let repo_name = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            println!("   Loading: {}", repo_name);
            let data = std::fs::read_to_string(&path)?;
            let examples: Vec<TrainingExample> = serde_json::from_str(&data)?;
            by_repo.insert(repo_name, examples);
        }
    }

    println!("\n📊 Found {} repositories", by_repo.len());

    // Count total examples
    let total_examples: usize = by_repo.values().map(|v| v.len()).sum();
    println!("   Total examples: {}", total_examples);

    // Split repos into train/validation/test sets
    let repo_names: Vec<String> = by_repo.keys().cloned().collect();

    // Use deterministic shuffle with seed for reproducibility
    use rand::seq::SliceRandom;
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let mut shuffled = repo_names.clone();
    shuffled.shuffle(&mut rng);

    let total = shuffled.len();
    let train_count = (total as f64 * 0.7).ceil() as usize;
    let val_count = (total as f64 * 0.15).ceil() as usize;

    let train_repos = &shuffled[0..train_count];
    let val_repos = &shuffled[train_count..train_count + val_count];
    let test_repos = &shuffled[train_count + val_count..];

    // Build datasets
    let mut train_examples = Vec::new();
    let mut val_examples = Vec::new();
    let mut test_examples = Vec::new();

    for repo in train_repos {
        if let Some(examples) = by_repo.get(repo) {
            train_examples.extend(examples.clone());
        }
    }
    for repo in val_repos {
        if let Some(examples) = by_repo.get(repo) {
            val_examples.extend(examples.clone());
        }
    }
    for repo in test_repos {
        if let Some(examples) = by_repo.get(repo) {
            test_examples.extend(examples.clone());
        }
    }

    // Save datasets
    println!("\n📊 Saving datasets to ./data/");
    std::fs::create_dir_all("data")?;

    std::fs::write(
        "data/train.json",
        serde_json::to_string_pretty(&train_examples)?,
    )?;
    std::fs::write(
        "data/val.json",
        serde_json::to_string_pretty(&val_examples)?,
    )?;
    std::fs::write(
        "data/test.json",
        serde_json::to_string_pretty(&test_examples)?,
    )?;

    // Also save as JSONL for streaming
    let train_jsonl: String = train_examples
        .iter()
        .filter_map(|e| serde_json::to_string(e).ok())
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write("data/train.jsonl", train_jsonl)?;

    println!("\n📊 Dataset split complete:");
    println!(
        "   Train: {} repos, {} examples",
        train_repos.len(),
        train_examples.len()
    );
    println!(
        "   Validation: {} repos, {} examples",
        val_repos.len(),
        val_examples.len()
    );
    println!(
        "   Test: {} repos, {} examples",
        test_repos.len(),
        test_examples.len()
    );

    println!("\n   Repositories:");
    println!("      Train: {:?}", train_repos);
    println!("      Val: {:?}", val_repos);
    println!("      Test: {:?}", test_repos);

    // Show label distribution
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

    println!("\n   Label Distribution:");
    println!("      Train: Alive={}, Dead={}", train_alive, train_dead);
    println!("      Val:   Alive={}, Dead={}", val_alive, val_dead);
    println!("      Test:  Alive={}, Dead={}", test_alive, test_dead);

    Ok(())
}
