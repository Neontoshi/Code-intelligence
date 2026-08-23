// src/bin/ci/report.rs

use crate::helpers::get_default_model;
use code_intelligence::analysis::service::{AnalysisService, AnalysisServiceConfig};
use code_intelligence::error::Result;
use std::path::PathBuf;

pub async fn run_report(
    path: PathBuf,
    format: &str,
    output: Option<PathBuf>,
    llm: bool,
) -> Result<()> {
    println!("📄 Generating report for: {:?}", path);
    println!("   Format: {}", format);

    let output_file = output.unwrap_or_else(|| {
        let ext = match format {
            "json" => "json",
            "html" => "html",
            "full" => "md",
            _ => "md",
        };
        PathBuf::from(format!("code_analysis.{}", ext))
    });

    // Run analysis
    let config = AnalysisServiceConfig {
        model_path: get_default_model().map(PathBuf::from),
        threshold: None,
        verbose: false,
        debug: false,
        cache: false,
        cache_dir: None,
        llm,
        git: false,
    };

    let mut service = AnalysisService::new(config);
    service.load_model()?;
    let result = service.analyze(&path).await?;

    // Generate output based on format
    let content = match format {
        "json" => result.project_analysis.to_json(),
        "full" => result.project_analysis.to_full_report(),
        _ => result.project_analysis.to_markdown(),
    };

    std::fs::write(&output_file, content)?;
    println!("✅ Report saved to: {:?}", output_file);

    Ok(())
}
