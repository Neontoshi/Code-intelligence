use crate::helpers::get_default_model;
use code_intelligence::analysis::dead_code::DeadCodeAnalyzer;
use code_intelligence::analysis::service::{AnalysisService, AnalysisServiceConfig};
use code_intelligence::error::Result;
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

    let model_path = model_path.or_else(get_default_model).map(PathBuf::from);

    let config = AnalysisServiceConfig {
        model_path,
        threshold: Some(threshold),
        verbose,
        debug: verbose,
        cache: false,
        cache_dir: None,
        llm: false,
        git: false,
    };

    let mut service = AnalysisService::new(config);
    let result = service.analyze(&path).await?;

    let dead_analysis = &result.dead_code_analysis;

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
