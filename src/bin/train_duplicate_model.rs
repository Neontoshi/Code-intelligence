// src/bin/train_duplicate_model.rs

use code_intelligence::{
    analysis::training_data::{FunctionFeatures, TrainingExample},
    graph::call_graph::FunctionNode,
    ml::{DuplicateClassifier, DuplicateExample, DuplicateLabel},
    optimize::dedup::comparators::StructuralComparator,
    Pipeline,
};
use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::path::PathBuf;

fn get_similarity_score(a: &FunctionNode, b: &FunctionNode) -> f64 {
    let scores = StructuralComparator::compare(a, b);
    scores.structural
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: train_duplicate_model <project_path_or_json_file> [model_output]");
        eprintln!("");
        eprintln!("Examples:");
        eprintln!("  train_duplicate_model ~/Documents/code-intelligence model.bin");
        eprintln!("  train_duplicate_model combined_training.json model.bin");
        std::process::exit(1);
    }

    let input_path = PathBuf::from(&args[1]);
    let model_file = if args.len() >= 3 {
        PathBuf::from(&args[2])
    } else {
        PathBuf::from("duplicate_model.bin")
    };

    println!(
        "🔬 Training duplicate detection model from: {:?}",
        input_path
    );
    println!("📁 Model output: {:?}", model_file);

    // Check if input is a JSON file or a project path
    let examples = if input_path.extension().map(|e| e == "json").unwrap_or(false) {
        // Load training data from JSON file
        println!("📊 Loading training data from JSON...");
        let data = std::fs::read_to_string(&input_path)?;
        // ⭐ FIX: Use the full path
        let training_examples: Vec<TrainingExample> = serde_json::from_str(&data)?;

        println!("   Loaded {} training examples", training_examples.len());

        // Convert to duplicate examples
        let mut duplicate_examples = Vec::new();
        let mut processed_pairs = HashSet::new();

        // Sample pairs from the training data
        let sample_size = training_examples.len().min(500);
        let sample = &training_examples[..sample_size];

        for i in 0..sample.len() {
            for j in (i + 1)..sample.len() {
                let a = &sample[i];
                let b = &sample[j];

                let key = (a.full_path.clone(), b.full_path.clone());
                if processed_pairs.contains(&key) {
                    continue;
                }
                processed_pairs.insert(key);

                // Calculate similarity based on features
                let similarity = calculate_similarity(&a.features, &b.features);

                let label = if similarity > 0.85 {
                    DuplicateLabel::Duplicate
                } else if similarity < 0.3 {
                    DuplicateLabel::NotDuplicate
                } else {
                    continue; // Skip ambiguous pairs
                };

                duplicate_examples.push(DuplicateExample {
                    func_a: a.features.clone(),
                    func_b: b.features.clone(),
                    label: label.clone(),
                    confidence: similarity,
                });
            }
        }

        duplicate_examples
    } else {
        // Process as a project directory
        println!("📊 Analyzing project: {:?}", input_path);

        let mut pipeline = Pipeline::new();
        let analysis = pipeline.process_project(&input_path).await?;

        let functions: Vec<FunctionNode> = analysis
            .call_graph
            .node_indices()
            .map(|idx| analysis.call_graph[idx].clone())
            .collect();

        println!("   Found {} functions", functions.len());

        let mut duplicates = Vec::new();
        let mut non_duplicates = Vec::new();
        let mut processed_pairs = HashSet::new();

        let sample_size = functions.len().min(500);
        let sample_functions = &functions[..sample_size];

        for i in 0..sample_functions.len() {
            for j in (i + 1)..sample_functions.len() {
                let a = &sample_functions[i];
                let b = &sample_functions[j];

                let key = (a.full_path.clone(), b.full_path.clone());
                if processed_pairs.contains(&key) {
                    continue;
                }
                processed_pairs.insert(key);

                let features_a = FunctionFeatures::from_function(a, &analysis.call_graph);
                let features_b = FunctionFeatures::from_function(b, &analysis.call_graph);

                let similarity = get_similarity_score(a, b);

                if similarity > 0.85 {
                    duplicates.push(DuplicateExample {
                        func_a: features_a,
                        func_b: features_b,
                        label: DuplicateLabel::Duplicate,
                        confidence: similarity,
                    });
                } else if similarity < 0.3 {
                    non_duplicates.push(DuplicateExample {
                        func_a: features_a,
                        func_b: features_b,
                        label: DuplicateLabel::NotDuplicate,
                        confidence: similarity,
                    });
                }
            }
        }

        println!("📊 Collected {} duplicates", duplicates.len());
        println!("   Collected {} non-duplicates", non_duplicates.len());

        // Balance the dataset
        let min_count = duplicates.len().min(non_duplicates.len());

        let mut rng = rand::thread_rng();
        duplicates.shuffle(&mut rng);
        non_duplicates.shuffle(&mut rng);

        let mut balanced = Vec::new();
        balanced.extend(duplicates.into_iter().take(min_count));
        balanced.extend(non_duplicates.into_iter().take(min_count));
        balanced.shuffle(&mut rng);

        balanced
    };

    println!("📊 Final dataset: {} examples", examples.len());
    if examples.is_empty() {
        println!("⚠️ No examples generated! Check your input.");
        return Ok(());
    }

    let duplicates_count = examples
        .iter()
        .filter(|e| e.label == DuplicateLabel::Duplicate)
        .count();
    let not_duplicates_count = examples
        .iter()
        .filter(|e| e.label == DuplicateLabel::NotDuplicate)
        .count();
    println!("   Duplicates: {}", duplicates_count);
    println!("   Not duplicates: {}", not_duplicates_count);

    // Train the model
    println!("\n🧠 Training duplicate classifier...");
    let mut classifier = DuplicateClassifier::new(101);
    let accuracy = classifier.train(&examples);

    println!("   Accuracy: {:.1}%", accuracy * 100.0);

    // Save the model
    classifier.save(&model_file.to_string_lossy())?;
    println!("✅ Model saved to: {:?}", model_file);

    // Show predictions
    println!("\n🔮 Sample Predictions:");
    for example in examples.iter().take(10) {
        let prediction = classifier.predict(&example.func_a, &example.func_b);
        let label = if prediction > 0.7 {
            "DUPLICATE"
        } else {
            "UNIQUE"
        };
        let confidence = if prediction > 0.5 {
            prediction
        } else {
            1.0 - prediction
        };
        println!(
            "   {} → {:.1}% (actual: {:?})",
            label,
            confidence * 100.0,
            example.label
        );
    }

    // Show type context stats
    println!("\n📊 Type Context Stats:");
    let with_types = examples
        .iter()
        .filter(|e| e.func_a.type_name.is_some() && e.func_b.type_name.is_some())
        .count();
    println!(
        "   Examples with type context: {}/{}",
        with_types,
        examples.len()
    );

    Ok(())
}

/// Calculate similarity between two feature sets
fn calculate_similarity(a: &FunctionFeatures, b: &FunctionFeatures) -> f64 {
    let a_vec = a.to_feature_vector();
    let b_vec = b.to_feature_vector();

    // Cosine similarity
    let dot: f64 = a_vec.iter().zip(&b_vec).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a_vec.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b_vec.iter().map(|x| x * x).sum::<f64>().sqrt();

    if norm_a > 0.0 && norm_b > 0.0 {
        dot / (norm_a * norm_b)
    } else {
        0.0
    }
}
