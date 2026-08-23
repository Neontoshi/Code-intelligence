// src/bin/merge_training_data.rs

use code_intelligence::analysis::training_data::TrainingExample;
use code_intelligence::error::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: merge_training_data <output.json> <input1.json> <input2.json> ...");
        eprintln!("Example: merge_training_data combined.json training_data.json go_training.json api_training.json");
        std::process::exit(1);
    }

    let output_path = PathBuf::from(&args[1]);
    let input_paths: Vec<PathBuf> = args[2..].iter().map(PathBuf::from).collect();

    println!("📊 Merging training data...");
    println!("   Output: {:?}", output_path);
    println!("   Inputs: {:?}", input_paths);

    let mut all_examples = Vec::new();

    for input_path in &input_paths {
        if !input_path.exists() {
            println!("   ⚠️ Skipping: {:?} (file not found)", input_path);
            continue;
        }

        println!("   Loading: {:?}", input_path);
        let data = std::fs::read_to_string(input_path)?;

        // Try to parse as array
        match serde_json::from_str::<Vec<TrainingExample>>(&data) {
            Ok(examples) => {
                println!("      Found {} examples", examples.len());
                all_examples.extend(examples);
            }
            Err(e) => {
                println!("      ⚠️ Failed to parse: {}", e);
                // Try to parse as JSONL (one JSON per line)
                let lines: Vec<&str> = data.lines().collect();
                let mut jsonl_examples = Vec::new();
                for line in lines {
                    if !line.trim().is_empty() {
                        if let Ok(example) = serde_json::from_str::<TrainingExample>(line) {
                            jsonl_examples.push(example);
                        }
                    }
                }
                if !jsonl_examples.is_empty() {
                    println!("      Found {} JSONL examples", jsonl_examples.len());
                    all_examples.extend(jsonl_examples);
                } else {
                    println!("      ⚠️ No valid examples found in file");
                }
            }
        }
    }

    println!("\n📊 Total examples: {}", all_examples.len());

    if all_examples.is_empty() {
        println!("⚠️ No examples found! Exiting.");
        std::process::exit(1);
    }

    // Count labels
    use code_intelligence::analysis::training_data::TrainingLabel;
    let alive = all_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Alive)
        .count();
    let dead = all_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Dead)
        .count();
    let unknown = all_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Unknown)
        .count();

    println!("   Alive: {}", alive);
    println!("   Dead: {}", dead);
    println!("   Unknown: {}", unknown);

    // Save merged data
    let json = serde_json::to_string_pretty(&all_examples)?;
    std::fs::write(&output_path, json)?;
    println!("\n✅ Merged training data saved to: {:?}", output_path);

    // Also save as JSONL
    let jsonl_path = output_path.with_extension("jsonl");
    let jsonl_content: String = all_examples
        .iter()
        .filter_map(|e| serde_json::to_string(e).ok())
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&jsonl_path, jsonl_content)?;
    println!("✅ JSONL format saved to: {:?}", jsonl_path);

    // Show sample
    if let Some(first) = all_examples.first() {
        println!("\n📝 Sample Example:");
        println!("   Function: {}", first.function_name);
        println!("   Label: {:?}", first.label);
        println!("   Language: {}", first.language);
        println!("   Features:");
        println!("      - Params: {}", first.features.param_count);
        println!("      - Public: {}", first.features.is_public);
        println!("      - Fan-in: {}", first.features.fan_in);
        println!("      - Complexity: {}", first.features.complexity);
    }

    Ok(())
}
