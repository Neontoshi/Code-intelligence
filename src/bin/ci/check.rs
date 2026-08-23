// src/bin/ci/check.rs

use crate::helpers::get_default_model;
use code_intelligence::analysis::service::{AnalysisService, AnalysisServiceConfig};
use code_intelligence::error::Result;
use code_intelligence::graph::GraphMetrics;
use std::path::PathBuf;

pub async fn run_check(
    path: PathBuf,
    max_dead: Option<usize>,
    max_ratio: Option<f64>,
    format: &str,
    output: Option<PathBuf>,
    fail_on_dead: bool,
    threshold: f64,
    conservative: bool,
) -> Result<()> {
    println!("🤖 Running in CI mode for: {:?}", path);
    println!("   Threshold: {:.2}", threshold);
    if conservative {
        println!("   Conservative mode: ON");
    }

    let config = AnalysisServiceConfig {
        model_path: get_default_model().map(PathBuf::from),
        threshold: Some(threshold),
        verbose: false,
        debug: false,
        cache: false,
        cache_dir: None,
        llm: false,
        git: false,
    };

    let mut service = AnalysisService::new(config);
    service.load_model()?;
    let result = service.analyze(&path).await?;

    // Generate report
    let report = if format == "json" {
        serde_json::json!({
            "project": path.to_string_lossy(),
            "threshold": threshold,
            "total_functions": result.call_graph.node_count(),
            "dead_functions": result.dead_verdicts.len(),
            "alive_functions": result.alive_verdicts.len(),
            "dead_ratio": if result.call_graph.node_count() > 0 {
                result.dead_verdicts.len() as f64 / result.call_graph.node_count() as f64
            } else { 0.0 },
            "status": if result.dead_verdicts.is_empty() { "PASS" } else { "FAIL" },
        })
        .to_string()
    } else {
        format!(
            "📊 CI Report\n===========\n\
             Project: {}\n\
             Threshold: {:.2}\n\
             Total Functions: {}\n\
             Dead Functions: {}\n\
             Dead Ratio: {:.1}%\n\
             Status: {}\n",
            path.to_string_lossy(),
            threshold,
            result.call_graph.node_count(),
            result.dead_verdicts.len(),
            if result.call_graph.node_count() > 0 {
                result.dead_verdicts.len() as f64 / result.call_graph.node_count() as f64 * 100.0
            } else {
                0.0
            },
            if result.dead_verdicts.is_empty() {
                "✅ PASS"
            } else {
                "❌ FAIL"
            }
        )
    };

    if let Some(output_path) = output {
        std::fs::write(output_path, &report)?;
    } else {
        println!("{}", report);
    }

    // Check conditions
    if let Some(max) = max_dead {
        if result.dead_verdicts.len() > max {
            eprintln!(
                "❌ Dead code count {} exceeds limit {}",
                result.dead_verdicts.len(),
                max
            );
            std::process::exit(1);
        }
    }

    if let Some(max) = max_ratio {
        let ratio = if result.call_graph.node_count() > 0 {
            result.dead_verdicts.len() as f64 / result.call_graph.node_count() as f64
        } else {
            0.0
        };
        if ratio > max {
            eprintln!(
                "❌ Dead ratio {:.1}% exceeds limit {:.1}%",
                ratio * 100.0,
                max * 100.0
            );
            std::process::exit(1);
        }
    }

    if fail_on_dead && !result.dead_verdicts.is_empty() {
        eprintln!("❌ Found {} dead functions", result.dead_verdicts.len());
        std::process::exit(1);
    }

    if result.dead_verdicts.is_empty() {
        println!("✅ No dead code found!");
    }

    Ok(())
}
