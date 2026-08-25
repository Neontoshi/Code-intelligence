// src/bin/ci/dedup.rs

use crate::helpers::get_default_duplicate_model;
use code_intelligence::error::{err, Result};
use std::path::PathBuf;

pub async fn run_dedup_report(
    path: PathBuf,
    threshold: f64,
    output: Option<PathBuf>,
    ml: bool,
    duplicate_model: Option<PathBuf>,
    verbose: bool,
) -> Result<()> {
    use code_intelligence::ml::duplicate_classifier::DuplicateClassifier;
    use code_intelligence::optimize::Deduplicator;
    use code_intelligence::Pipeline;

    println!("🔍 Finding duplicates in: {:?}", path);
    println!("📊 Similarity threshold: {:.2}", threshold);
    if ml {
        println!("🤖 ML detection: enabled");
    }
    println!();

    let model = if ml {
        let loaded_model = match (duplicate_model, get_default_duplicate_model()) {
            (Some(custom_path), _) => {
                if !custom_path.exists() {
                    return Err(err::model(format!(
                        "Specified duplicate model not found: {:?}",
                        custom_path
                    )));
                }
                if verbose {
                    println!("✅ Loaded custom duplicate model from: {:?}", custom_path);
                }
                DuplicateClassifier::load(&custom_path).map_err(|e| err::model(e))?
            }
            (None, Some(config_path)) if config_path.exists() => {
                if verbose {
                    println!(
                        "✅ Loaded configured duplicate model from: {:?}",
                        config_path
                    );
                }
                DuplicateClassifier::load(&config_path).map_err(|e| err::model(e))?
            }
            _ => {
                if verbose {
                    println!("🧠 Using built-in embedded duplicate model");
                }
                DuplicateClassifier::load_embedded().map_err(|e| err::model(e))?
            }
        };
        Some(loaded_model)
    } else {
        None
    };

    // Run pipeline
    let mut pipeline = Pipeline::new();
    let analysis = pipeline.process_project(&path).await?;

    // Execute deduplication
    let mut dedup = if let Some(model) = model {
        Deduplicator::new_with_ml(Some(model))
    } else {
        Deduplicator::new()
    };
    dedup = dedup.with_threshold(threshold);

    let result = dedup.find_duplicates(&analysis.call_graph, &analysis.files);

    // Output and save
    let report = dedup.report(&result);
    println!("{}", report);

    if let Some(output_path) = output {
        std::fs::write(&output_path, &report)?;
        println!("\n✅ Report saved to: {:?}", output_path);
    }

    println!("\n📊 Summary:");
    println!("   Duplicate groups: {}", result.duplicate_groups.len());
    println!("   Total token savings: ~{}", result.total_saved_tokens);
    println!(
        "   Confidence: {:.1}%",
        result.accuracy_metrics.confidence_score * 100.0
    );

    if verbose && !result.duplicate_groups.is_empty() {
        println!("\n🔍 Duplicate Groups:");
        for (i, group) in result.duplicate_groups.iter().enumerate() {
            println!(
                "   {}: {} functions, similarity: {:.1}%, type: {:?}",
                i + 1,
                group.functions.len(),
                group.similarity_score * 100.0,
                group.duplicate_type
            );
        }
    }

    Ok(())
}
