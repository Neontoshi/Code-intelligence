// src/bin/merge_all_training.rs

use code_intelligence::analysis::training_data::TrainingExample;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let training_dir = PathBuf::from("training_data");
    let output_file = PathBuf::from("combined_training.json");

    println!("📊 Merging all training data...");

    let mut all_examples = Vec::new();
    let mut stats = std::collections::HashMap::new();

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

            match serde_json::from_str::<Vec<TrainingExample>>(&data) {
                Ok(examples) => {
                    println!("      Found {} examples", examples.len());
                    all_examples.extend(examples);
                    stats.insert(repo_name, all_examples.len());
                }
                Err(e) => {
                    eprintln!("      ⚠️ Failed to parse: {}", e);
                }
            }
        }
    }

    println!("\n📊 Total examples: {}", all_examples.len());
    println!("\n   By repository:");
    for (repo, count) in &stats {
        println!("      {}: {}", repo, count);
    }

    // Save merged data
    let json = serde_json::to_string_pretty(&all_examples)?;
    std::fs::write(&output_file, json)?;
    println!("\n✅ Merged training data saved to: {:?}", output_file);

    // Also save as JSONL
    let jsonl_path = output_file.with_extension("jsonl");
    let jsonl_content: String = all_examples
        .iter()
        .filter_map(|e| serde_json::to_string(e).ok())
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&jsonl_path, jsonl_content)?;
    println!("✅ JSONL format saved to: {:?}", jsonl_path);

    Ok(())
}

