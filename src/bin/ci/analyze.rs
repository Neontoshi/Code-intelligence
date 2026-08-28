// src/bin/ci/analyze.rs

use crate::helpers::{detect_project_type, get_default_model, save_project_config};
use crate::types::ProjectConfig;
use code_intelligence::analysis::service::{AnalysisService, AnalysisServiceConfig};
use code_intelligence::error::Result;
use code_intelligence::graph::GraphMetrics;
use code_intelligence::optimize::Deduplicator;
use std::path::PathBuf;

pub async fn run_analyze(
    path: PathBuf,
    threshold: Option<f64>,
    verbose: bool,
    llm: bool,
    git: bool,
    cache: bool,
    cache_dir: Option<PathBuf>,
    model_path: Option<PathBuf>,
) -> Result<()> {
    println!("🔍 Analyzing project: {:?}", path);
    println!("{}", "=".repeat(60));
    println!();

    let project_type = detect_project_type(&path);
    if let Some(pt) = &project_type {
        println!("📊 Detected project type: {}", pt);
    }

    let model_path = model_path.or_else(get_default_model).map(PathBuf::from);

    let config = AnalysisServiceConfig {
        model_path,
        threshold,
        verbose,
        debug: verbose,
        cache,
        cache_dir,
        llm,
        git,
    };

    let mut service = AnalysisService::new(config);
    let result = service.analyze(&path).await?;

    println!("\n{}", "═".repeat(60));
    println!("🔍 DEAD CODE ANALYSIS");
    println!("{}", "═".repeat(60));

    let dead_analysis = &result.dead_code_analysis;

    println!("\n📊 Dead Code Summary:");
    println!("   Total functions: {}", result.call_graph.node_count());
    println!("   Dead functions: {}", dead_analysis.functions.len());
    println!("   Alive functions: {}", result.alive_verdicts.len());
    println!("   Unknown: {}", result.unknown_verdicts.len());
    println!("   Effective threshold: {:.2}", result.effective_threshold);
    println!(
        "   Dead code ratio: {:.1}%",
        if result.call_graph.node_count() > 0 {
            dead_analysis.functions.len() as f64 / result.call_graph.node_count() as f64 * 100.0
        } else {
            0.0
        }
    );
    println!(
        "   Estimated LOC removable: {}",
        dead_analysis.summary.estimated_loc_removable
    );

    // Show dead functions table
    if !dead_analysis.functions.is_empty() {
        println!("\n🎯 Dead Functions (Priority Order):");
        println!(
            "   {:<4} {:<40} {:<12} {:<10} {:<8}",
            "#", "Function", "Confidence", "Impact", "LOC"
        );
        println!(
            "   {:-<4} {:-<40} {:-<12} {:-<10} {:-<8}",
            "", "", "", "", ""
        );
        for (i, func) in dead_analysis.functions.iter().enumerate() {
            let confidence = func.score.score * 100.0;
            let emoji = if confidence >= 95.0 {
                "🔴"
            } else if confidence >= 80.0 {
                "🟠"
            } else {
                "🟡"
            };
            println!(
                "   {:<4} {:<40} {} {:.1}%  {:<10} {:<8}",
                i + 1,
                &func.name[..func.name.len().min(38)],
                emoji,
                confidence,
                func.impact
                    .estimated_removal_impact
                    .split('-')
                    .next()
                    .unwrap_or("")
                    .trim(),
                func.impact.lines_of_code
            );
        }
        println!();
        println!("   Run `ci deadcode . --output report.md` for full details");
    } else {
        println!("\n   ✅ No dead code found! 🎉");
    }

    // 2. DUPLICATE CODE ANALYSIS
    println!("\n{}", "═".repeat(60));
    println!("🔄 DUPLICATE CODE ANALYSIS");
    println!("{}", "═".repeat(60));

    let dedup = Deduplicator::new().with_threshold(0.85);
    let dedup_result = dedup.find_duplicates(&result.call_graph, &result.files);

    if dedup_result.duplicate_groups.is_empty() {
        println!("\n   ✅ No duplicate code found! 🎉");
    } else {
        println!("\n📊 Duplicate Code Summary:");
        println!(
            "   Duplicate groups: {}",
            dedup_result.duplicate_groups.len()
        );
        println!(
            "   Total token savings: ~{}",
            dedup_result.total_saved_tokens
        );
        println!(
            "   Confidence: {:.1}%",
            dedup_result.accuracy_metrics.confidence_score * 100.0
        );

        println!("\n🔍 Duplicate Groups:");
        println!(
            "   {:<4} {:<12} {:<10} {:<15} {:<15}",
            "#", "Type", "Functions", "Similarity", "Savings"
        );
        println!(
            "   {:-<4} {:-<12} {:-<10} {:-<15} {:-<15}",
            "", "", "", "", ""
        );
        for (i, group) in dedup_result.duplicate_groups.iter().enumerate() {
            println!(
                "   {:<4} {:?}     {:<10} {:.1}%      ~{}",
                i + 1,
                group.duplicate_type,
                group.functions.len(),
                group.similarity_score * 100.0,
                group.total_token_savings
            );
        }
        println!();
        println!("   Run `ci dedup . --output report.md` for full details");
    }

    // 3. IMPORTANT FUNCTIONS
    println!("\n{}", "═".repeat(60));
    println!("🔥 IMPORTANT FUNCTIONS (High Impact)");
    println!("{}", "═".repeat(60));

    let mut important: Vec<_> = result
        .call_graph
        .node_indices()
        .map(|idx| {
            (
                idx,
                result.call_graph[idx].importance_score,
                result.call_graph[idx].name.clone(),
            )
        })
        .collect();
    important.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    println!("\n   Top 10 most important functions (by call frequency):");
    println!(
        "   {:<40} {:<12} {:<10}",
        "Function", "Importance", "Callers"
    );
    println!("   {:-<40} {:-<12} {:-<10}", "", "", "");
    for (idx, score, name) in important.iter().take(10) {
        let callers = result.call_graph.get_callers(*idx).len();
        let emoji = if *score > 0.7 {
            "🔥"
        } else if *score > 0.5 {
            "📌"
        } else {
            "📄"
        };
        println!(
            "   {} {:<38} {:.2}        {:<10}",
            emoji,
            &name[..name.len().min(37)],
            score,
            callers
        );
    }

    // 4. RECOMMENDATIONS
    println!("\n{}", "═".repeat(60));
    println!("💡 RECOMMENDATIONS");
    println!("{}", "═".repeat(60));

    let dead_count = dead_analysis.functions.len();
    let dup_count = dedup_result.duplicate_groups.len();
    let loc_removable = dead_analysis.summary.estimated_loc_removable;

    if dead_count > 0 {
        println!(
            "\n   1. 🧹 Remove {} dead functions ({} LOC)",
            dead_count, loc_removable
        );
        println!("      → `ci deadcode . --output deadcode.md`");
    }
    if dup_count > 0 {
        println!("   2. 🔄 Refactor {} duplicate groups", dup_count);
        println!("      → `ci dedup . --output dedup.md`");
    }
    if dead_count == 0 && dup_count == 0 {
        println!("\n   ✅ Your codebase is clean! No dead code or duplicates found.");
    }
    if dead_count > 0 || dup_count > 0 {
        println!("   3. 📊 Generate complete report");
        println!("      → `ci report . --format markdown --output full_report.md`");
    }

    println!("\n{}", "=".repeat(60));
    println!("\n✅ Analysis complete!");

    // Save project config
    let project_config = ProjectConfig {
        path: path.to_string_lossy().to_string(),
        project_type,
        threshold,
        last_analyzed: Some(chrono::Local::now().to_string()),
        dead_count: Some(dead_analysis.functions.len()),
    };
    let _ = save_project_config(&path, project_config);

    Ok(())
}
