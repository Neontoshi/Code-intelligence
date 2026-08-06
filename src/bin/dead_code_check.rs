// src/bin/dead_code_check.rs

use code_intelligence::analysis::dead_code::{DeadCodeAnalysis, DeadCodeDetector};
use code_intelligence::analysis::git_analysis::GitAnalyzer;
use code_intelligence::Pipeline;
use std::collections::HashSet;
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

    // Enhanced filtering with better pattern matching
    let filtered_functions: Vec<_> = dead_analysis
        .functions
        .iter()
        .filter(|f| {
            // Skip React components (uppercase names in .tsx/.jsx)
            if f.file.ends_with(".tsx") || f.file.ends_with(".jsx") {
                let is_component = f
                    .name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false);
                if is_component {
                    return false;
                }
            }

            // Skip React hooks
            if f.name.starts_with("use") && !f.name.starts_with("useSolanaGiveaway") {
                return false;
            }

            // Skip state setters
            if f.name.starts_with("set")
                && f.name
                    .chars()
                    .nth(3)
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
            {
                return false;
            }

            // Skip React Router variables
            if f.name == "links"
                || f.name == "location"
                || f.name == "navigate"
                || f.name == "params"
                || f.name == "searchParams"
                || f.name == "match"
                || f.name == "routes"
            {
                return false;
            }

            // Skip Router components (they're used in Routes)
            if f.name == "CreatePage" || f.name == "SearchPage" || f.name == "App" {
                return false;
            }

            // Skip known alive functions
            let alive_functions = [
                "constructor",
                "request",
                "buildCreateAndCommitGiveaway",
                "submitGiveaway",
                "buildReveal",
                "submitReveal",
            ];
            if alive_functions.contains(&f.name.as_str()) {
                return false;
            }

            // Only show functions with confidence > 70%
            f.score.score > 0.70
        })
        .cloned()
        .collect();

    // Create filtered analysis
    let filtered_analysis = DeadCodeAnalysis {
        functions: filtered_functions.clone(),
        types: dead_analysis.types,
        modules: dead_analysis.modules,
        reachability: dead_analysis.reachability,
        summary: dead_analysis.summary,
    };

    // Generate report with filtered results
    let report = DeadCodeDetector::generate_report(&filtered_analysis);
    println!("{}", report);

    // Show detailed filtered stats
    println!("\n📊 Filtered Results:");
    println!(
        "   Original dead functions: {}",
        dead_analysis.functions.len()
    );
    println!(
        "   Remaining dead functions: {}",
        filtered_analysis.functions.len()
    );
    println!(
        "   Filtered false positives: {}",
        dead_analysis.functions.len() - filtered_analysis.functions.len()
    );
    println!("   Confidence threshold: > 70%");

    // Show what was removed - using HashSet for O(n) lookup
    if dead_analysis.functions.len() > filtered_analysis.functions.len() {
        println!("\n📋 Filtered out:");

        // Build a set of full_paths from filtered_functions for quick lookup
        let filtered_paths: HashSet<String> = filtered_analysis
            .functions
            .iter()
            .map(|f| f.full_path.clone())
            .collect();

        for f in dead_analysis.functions.iter() {
            if !filtered_paths.contains(&f.full_path) {
                println!(
                    "   - {} (from {})",
                    f.name,
                    f.file.split('/').last().unwrap_or("")
                );
            }
        }
    }

    Ok(())
}
