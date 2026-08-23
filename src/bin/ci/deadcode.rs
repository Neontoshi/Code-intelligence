// src/bin/ci/deadcode.rs

use crate::helpers::get_default_model;
use code_intelligence::analysis::dead_code::DeadCodeAnalyzer;
use code_intelligence::analysis::service::{AnalysisService, AnalysisServiceConfig};
use code_intelligence::error::{err, Result};
use std::path::PathBuf;

pub async fn run_deadcode(
    path: PathBuf,
    threshold: f64,
    output: Option<PathBuf>,
    model_path: Option<PathBuf>,
    verbose: bool,
) -> Result<()> {
    println!("🔍 Analyzing dead code in: {:?}", path);
    println!("📊 Threshold: {:.2}", threshold);
    println!();

    // Get model path
    let model_path = model_path
        .or_else(get_default_model)
        .map(PathBuf::from)
        .ok_or_else(|| err::config("No model configured. Run: ci config set model <path>"))?;

    if !model_path.exists() {
        return Err(err::model(format!(
            "Model file not found: {:?}",
            model_path
        )));
    }

    // Build config
    let config = AnalysisServiceConfig {
        model_path: Some(model_path),
        threshold: Some(threshold),
        verbose,
        debug: verbose,
        cache: false,
        cache_dir: None,
        llm: false,
        git: false,
    };

    let mut service = AnalysisService::new(config);
    service.load_model()?;
    let result = service.analyze(&path).await?;

    // service.analyze() already computed root detection, reachability,
    // dynamic refs, verdicts, and the structural dead-code analysis at the
    // requested threshold — reuse it instead of rebuilding it here.
    let dead_analysis = &result.dead_code_analysis;

    // Generate report (generate_report is stateless — a fresh analyzer is fine)
    let report = DeadCodeAnalyzer::new().generate_report(dead_analysis);

    // Print to terminal
    println!("{}", report);

    // Save to file if output specified
    if let Some(output_path) = output {
        std::fs::write(&output_path, &report)?;
        println!("\n✅ Report saved to: {:?}", output_path);
    }

    // Print summary
    println!("\n📊 Summary:");
    println!("   Dead functions: {}", dead_analysis.functions.len());
    println!("   Dead types: {}", dead_analysis.summary.dead_types);
    println!("   Dead modules: {}", dead_analysis.summary.dead_modules);
    println!("   Dead files: {}", dead_analysis.summary.dead_files);
    println!(
        "   Estimated LOC removable: {}",
        dead_analysis.summary.estimated_loc_removable
    );
    println!(
        "   Avg confidence: {:.1}%",
        dead_analysis.summary.avg_confidence * 100.0
    );

    Ok(())
}
