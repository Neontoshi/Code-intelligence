// src/bin/analyze_features_per_language.rs

use code_intelligence::analysis::training_data::{TrainingExample, TrainingLabel};
use code_intelligence::error::Result;
use code_intelligence::ml::feature_schema::FEATURE_SCHEMA;
use std::collections::HashMap;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let data_file = if args.len() >= 2 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from("combined_training.json")
    };

    println!("🔬 Analyzing feature importance per language...");
    let data = std::fs::read_to_string(&data_file)?;
    let examples: Vec<TrainingExample> = serde_json::from_str(&data)?;

    // Group by language
    let mut by_language: HashMap<String, Vec<&TrainingExample>> = HashMap::new();
    for example in &examples {
        if example.label != TrainingLabel::Unknown {
            by_language
                .entry(example.language.clone())
                .or_default()
                .push(example);
        }
    }

    // For each language, find the most important features
    for (language, examples) in &by_language {
        println!("\n📊 {} ({} examples)", language, examples.len());
        println!("   Top features for ALIVE vs DEAD:");

        // Calculate feature averages for ALIVE vs DEAD
        let mut alive_features = vec![0.0; FEATURE_SCHEMA.feature_count()];
        let mut dead_features = vec![0.0; FEATURE_SCHEMA.feature_count()];
        let mut alive_count = 0;
        let mut dead_count = 0;

        for example in examples {
            let features = example.features.to_feature_vector();
            if example.label == TrainingLabel::Alive {
                for (i, &f) in features.iter().enumerate() {
                    if i < alive_features.len() {
                        alive_features[i] += f;
                    }
                }
                alive_count += 1;
            } else if example.label == TrainingLabel::Dead {
                for (i, &f) in features.iter().enumerate() {
                    if i < dead_features.len() {
                        dead_features[i] += f;
                    }
                }
                dead_count += 1;
            }
        }

        // Calculate averages
        if alive_count > 0 {
            for f in &mut alive_features {
                *f /= alive_count as f64;
            }
        }
        if dead_count > 0 {
            for f in &mut dead_features {
                *f /= dead_count as f64;
            }
        }

        // Calculate differences
        let mut diffs: Vec<(String, f64)> = Vec::new();
        let names = FEATURE_SCHEMA.feature_names();

        for i in 0..FEATURE_SCHEMA.feature_count() {
            let diff = alive_features[i] - dead_features[i];
            if let Some(name) = names.get(i) {
                diffs.push((name.clone(), diff));
            }
        }

        // Sort by absolute difference
        diffs.sort_by(|a, b| b.1.abs().total_cmp(&a.1.abs()));

        println!("   Most distinguishing features:");
        for (name, diff) in diffs.iter().take(5) {
            let direction = if *diff > 0.0 { "→ ALIVE" } else { "→ DEAD" };
            println!("      {}: {:.3} {}", name, diff, direction);
        }

        // Show count distribution
        println!("   Alive: {}, Dead: {}", alive_count, dead_count);
    }

    Ok(())
}
