// src/bin/evaluate_per_language.rs

use code_intelligence::analysis::training_data::{TrainingExample, TrainingLabel};
use code_intelligence::ml::classifier::DeadCodeClassifier;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct LanguageStats {
    total: usize,
    correct: usize,
    false_positives: usize,
    false_negatives: usize,
    precision: f64,
    recall: f64,
    f1: f64,
    accuracy: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let data_file = if args.len() >= 2 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from("combined_training.json")
    };

    let model_file = if args.len() >= 3 {
        PathBuf::from(&args[2])
    } else {
        PathBuf::from("model.bin")
    };

    println!("🔬 Loading training data from: {:?}", data_file);
    let data = std::fs::read_to_string(&data_file)?;
    let examples: Vec<TrainingExample> = serde_json::from_str(&data)?;

    println!("📊 Loaded {} examples", examples.len());

    println!("🧠 Loading model from: {:?}", model_file);
    let classifier = DeadCodeClassifier::load(&model_file.to_string_lossy())?;

    // Group examples by language
    let mut by_language: HashMap<String, Vec<&TrainingExample>> = HashMap::new();
    for example in &examples {
        if example.label != TrainingLabel::Unknown {
            by_language
                .entry(example.language.clone())
                .or_default()
                .push(example);
        }
    }

    println!("\n📊 Per-Language Evaluation:\n");
    println!(
        "{:>12} | {:>8} | {:>10} | {:>10} | {:>10} | {:>10} | {:>10} | {:>10}",
        "Language", "Count", "Accuracy", "Precision", "Recall", "F1", "FP", "FN"
    );
    println!(
        "{:-<12}-+-{:-<8}-+-{:-<10}-+-{:-<10}-+-{:-<10}-+-{:-<10}-+-{:-<10}-+-{:-<10}",
        "", "", "", "", "", "", "", ""
    );

    let mut total_stats = HashMap::new();
    let mut overall_total = 0;
    let mut overall_correct = 0;
    let mut overall_fp = 0;
    let mut overall_fn = 0;

    for (language, language_examples) in &by_language {
        let mut correct = 0;
        let mut false_positives = 0;
        let mut false_negatives = 0;
        let total = language_examples.len();

        for example in language_examples {
            let prediction = classifier.predict(example);
            let actual = &example.label;

            if prediction == *actual {
                correct += 1;
            } else if prediction == TrainingLabel::Dead && *actual == TrainingLabel::Alive {
                false_positives += 1;
            } else if prediction == TrainingLabel::Alive && *actual == TrainingLabel::Dead {
                false_negatives += 1;
            }
        }

        let accuracy = correct as f64 / total as f64;
        let precision = if correct + false_positives > 0 {
            correct as f64 / (correct + false_positives) as f64
        } else {
            0.0
        };
        let recall = if correct + false_negatives > 0 {
            correct as f64 / (correct + false_negatives) as f64
        } else {
            0.0
        };
        let f1 = if precision + recall > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        };

        let stats = LanguageStats {
            total,
            correct,
            false_positives,
            false_negatives,
            precision,
            recall,
            f1,
            accuracy,
        };

        total_stats.insert(language.clone(), stats);
        overall_total += total;
        overall_correct += correct;
        overall_fp += false_positives;
        overall_fn += false_negatives;

        let emoji = if accuracy > 0.90 {
            "🟢"
        } else if accuracy > 0.80 {
            "🟡"
        } else {
            "🔴"
        };

        println!(
            "{:>12} | {:>8} | {:>9.1}% | {:>9.1}% | {:>9.1}% | {:>9.1}% | {:>8} | {:>8}",
            format!("{} {}", emoji, language),
            total,
            accuracy * 100.0,
            precision * 100.0,
            recall * 100.0,
            f1 * 100.0,
            false_positives,
            false_negatives
        );
    }

    // Overall stats
    let overall_accuracy = overall_correct as f64 / overall_total as f64;
    let overall_precision = if overall_correct + overall_fp > 0 {
        overall_correct as f64 / (overall_correct + overall_fp) as f64
    } else {
        0.0
    };
    let overall_recall = if overall_correct + overall_fn > 0 {
        overall_correct as f64 / (overall_correct + overall_fn) as f64
    } else {
        0.0
    };
    let overall_f1 = if overall_precision + overall_recall > 0.0 {
        2.0 * (overall_precision * overall_recall) / (overall_precision + overall_recall)
    } else {
        0.0
    };

    println!(
        "{:-<12}-+-{:-<8}-+-{:-<10}-+-{:-<10}-+-{:-<10}-+-{:-<10}-+-{:-<10}-+-{:-<10}",
        "", "", "", "", "", "", "", ""
    );
    println!(
        "{:>12} | {:>8} | {:>9.1}% | {:>9.1}% | {:>9.1}% | {:>9.1}% | {:>8} | {:>8}",
        "TOTAL",
        overall_total,
        overall_accuracy * 100.0,
        overall_precision * 100.0,
        overall_recall * 100.0,
        overall_f1 * 100.0,
        overall_fp,
        overall_fn
    );

    // Recommendations
    println!("\n💡 Recommendations:");
    for (language, stats) in &total_stats {
        if stats.accuracy < 0.85 {
            println!("   🔴 Add more training data for {}", language);
        } else if stats.false_positives > stats.false_negatives {
            println!("   🟡 Model has too many false positives for {}", language);
            println!("      Consider lowering threshold or adding more ALIVE examples");
        } else if stats.false_negatives > stats.false_positives {
            println!("   🟡 Model has too many false negatives for {}", language);
            println!("      Consider raising threshold or adding more DEAD examples");
        }
    }

    // Language distribution
    println!("\n📊 Language Distribution:");
    for (language, examples) in &by_language {
        let alive = examples
            .iter()
            .filter(|e| e.label == TrainingLabel::Alive)
            .count();
        let dead = examples
            .iter()
            .filter(|e| e.label == TrainingLabel::Dead)
            .count();
        println!(
            "   {}: {} examples ({} alive, {} dead)",
            language,
            examples.len(),
            alive,
            dead
        );
    }

    Ok(())
}
