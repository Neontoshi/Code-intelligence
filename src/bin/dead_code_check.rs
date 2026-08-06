// src/bin/dead_code_check.rs

use code_intelligence::analysis::dead_code::DeadCodeDetector;
use code_intelligence::analysis::git_analysis::GitAnalyzer;
use code_intelligence::Pipeline;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let path = if args.len() >= 2 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from(".")
    };

    println!("🔍 Analyzing dead code in: {:?}\n", path);

    let mut pipeline = Pipeline::new();
    let analysis = pipeline.process_project(&path).await?;

    // Try to get git analysis
    let git_analysis = GitAnalyzer::analyze(&path).ok();

    // Run comprehensive dead code analysis
    let dead_analysis = DeadCodeDetector::analyze(
        &analysis.call_graph,
        &analysis.type_graph,
        &analysis.import_graph,
        &analysis.dependency_graph,
        &analysis.files,
        git_analysis.as_ref(),
    );

    // Generate report
    let report = DeadCodeDetector::generate_report(&dead_analysis);
    println!("{}", report);

    Ok(())
}
