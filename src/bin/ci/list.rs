// src/bin/ci/list.rs

use code_intelligence::error::{err, Result};
use code_intelligence::graph::GraphMetrics;
use std::path::Path;

use crate::helpers::get_default_model;

pub async fn run_list(path: &Path, all: bool) -> Result<()> {
    use code_intelligence::analysis::outcomes::OutcomeTracker;
    use code_intelligence::analysis::service::{AnalysisService, AnalysisServiceConfig};

    println!("🔍 Scanning for dead code in: {:?}", path);
    if all {
        println!("   (showing dead + unknown verdicts)");
    }
    println!();

    // Get model path
    let model_path = get_default_model()
        .ok_or_else(|| err::config("No model configured. Run: ci config set model <path>"))?;

    if !model_path.exists() {
        return Err(err::model(format!(
            "Model file not found: {:?}",
            model_path
        )));
    }

    let path_buf = path.to_path_buf();

    let config = AnalysisServiceConfig {
        model_path: Some(model_path),
        threshold: Some(0.92),
        verbose: false,
        debug: false,
        cache: false,
        cache_dir: None,
        llm: false,
        git: false,
    };

    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let mut service = AnalysisService::new(config);
            service.load_model()?;
            service.analyze(&path_buf).await
        })
    })?;

    // Everything below used to be recomputed by hand (root detection,
    // reachability, dynamic refs, verdict evaluation). service.analyze()
    // already did all of that once — reuse it instead of doing it twice.
    let dead_verdicts = &result.dead_verdicts;
    let unknown_verdicts: Vec<_> = if all {
        result.unknown_verdicts.iter().collect()
    } else {
        Vec::new()
    };

    if dead_verdicts.is_empty() && unknown_verdicts.is_empty() {
        println!("✅ No dead functions found!");
        return Ok(());
    }

    // Show summary
    println!("\n📊 Dead Code Summary:");
    println!("{}", "═".repeat(60));
    println!("   Total functions: {}", result.call_graph.node_count());
    println!("   Dead functions: {}", dead_verdicts.len());
    if all {
        println!("   Unknown functions: {}", unknown_verdicts.len());
    }
    if result.call_graph.node_count() > 0 {
        println!(
            "   Dead code ratio: {:.1}%",
            dead_verdicts.len() as f64 / result.call_graph.node_count() as f64 * 100.0
        );
    }

    // Build the combined display list: Dead first, then Unknown (if --all)
    let mut display: Vec<(
        &code_intelligence::analysis::verdict_source::state::Verdict,
        &str,
    )> = dead_verdicts.iter().map(|v| (v, "Dead")).collect();
    if all {
        display.extend(unknown_verdicts.iter().map(|v| (*v, "Unknown")));
    }

    display.sort_by(|a, b| {
        b.0.confidence
            .partial_cmp(&a.0.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    println!("\n📋 Functions:");
    println!("");
    if all {
        println!("| # | Function | Verdict | Confidence | File |");
        println!("|---|----------|---------|------------|------|");
    } else {
        println!("| # | Function | Confidence | File |");
        println!("|---|----------|------------|------|");
    }

    for (i, (verdict, label)) in display.iter().enumerate() {
        let file = verdict
            .full_path
            .split("::")
            .next()
            .unwrap_or(&verdict.full_path);
        let short_file = file.split('/').last().unwrap_or(file);
        if all {
            println!(
                "| {} | {} | {} | {:.1}% | {} |",
                i + 1,
                verdict.function_name,
                label,
                verdict.confidence * 100.0,
                short_file
            );
        } else {
            println!(
                "| {} | {} | {:.1}% | {} |",
                i + 1,
                verdict.function_name,
                verdict.confidence * 100.0,
                short_file
            );
        }
    }

    println!("\n💡 Commands:");
    println!("   ci deadcode . --output report.md  - Full detailed report");
    println!("   ci remove <name>                   - Mark as removed");
    println!("   ci keep <name> \"reason\"           - Mark as false positive");

    let mut tracker = OutcomeTracker::new(path);
    let project_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let dead_verdict_refs: Vec<&code_intelligence::analysis::verdict_source::state::Verdict> =
        dead_verdicts.iter().collect();
    let _ = tracker.import_verdicts(&dead_verdict_refs, &project_name);

    Ok(())
}
