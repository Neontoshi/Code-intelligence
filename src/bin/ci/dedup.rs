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

    // Get duplicate model if ML enabled
    let model = if ml {
        let model_path = duplicate_model
            .or_else(get_default_duplicate_model)
            .ok_or_else(|| err::config("No duplicate model configured"))?;

        if !model_path.exists() {
            return Err(err::model(format!(
                "Duplicate model not found: {:?}",
                model_path
            )));
        }

        Some(DuplicateClassifier::load(&*model_path.to_string_lossy())?)
    } else {
        None
    };

    // Run pipeline
    let mut pipeline = Pipeline::new();
    let analysis = pipeline.process_project(&path).await?;

    // Find duplicates
    let mut dedup = if let Some(model) = model {
        Deduplicator::new_with_ml(Some(model))
    } else {
        Deduplicator::new()
    };
    dedup = dedup.with_threshold(threshold);

    let result = dedup.find_duplicates(&analysis.call_graph, &analysis.files);

    // Generate report
    let report = dedup.report(&result);

    // Print to terminal
    println!("{}", report);

    // Save to file if output specified
    if let Some(output_path) = output {
        std::fs::write(&output_path, &report)?;
        println!("\n✅ Report saved to: {:?}", output_path);
    }

    // Print summary
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
