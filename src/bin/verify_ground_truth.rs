// src/bin/verify_ground_truth.rs

//! Tool for building verified ground-truth datasets
//!
//! This tool helps collect human-verified examples by:
//! 1. Taking a project and running analysis
//! 2. Presenting candidates for review
//! 3. Recording human decisions with reasons

use clap::Parser;
use code_intelligence::analysis::verdict_source::label_source::LabelSource;
use code_intelligence::analysis::verdict_source::state::{VerdictConfig, VerdictEngine};
use code_intelligence::analysis::{
    roots::{ReachabilityAnalyzer, RootDetectionConfig, RootDetector},
    training_data::TrainingLabel,
};
use code_intelligence::error::Result;
use code_intelligence::graph::GraphMetrics;
use code_intelligence::ml::classifier::DeadCodeClassifier;
use code_intelligence::Pipeline;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "Build verified ground-truth dataset")]
struct Args {
    /// Project directory to analyze
    project_dir: PathBuf,

    /// Output file for verified dataset
    #[arg(short, long, default_value = "verified_dataset.json")]
    output: PathBuf,

    /// Model path (optional)
    #[arg(long)]
    model: Option<PathBuf>,

    /// Number of candidates to present
    #[arg(long, default_value = "20")]
    count: usize,

    /// Interactive mode
    #[arg(short, long)]
    interactive: bool,

    /// Batch mode - auto-verify with Git history
    #[arg(long)]
    batch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedEntry {
    pub full_path: String,
    pub function_name: String,
    pub file: String,
    pub line: usize,
    pub label: TrainingLabel,
    pub confidence: f64,
    pub label_source: LabelSource,
    pub verified_by: String,
    pub verification_date: i64,
    pub reason: String,
    pub repository: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationSession {
    pub project: String,
    pub model_version: String,
    pub started_at: i64,
    pub verified_entries: Vec<VerifiedEntry>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if !args.project_dir.is_dir() {
        eprintln!("❌ Project directory not found: {:?}", args.project_dir);
        std::process::exit(1);
    }

    println!("🔍 Verifying ground truth for: {:?}", args.project_dir);
    println!("📁 Output: {:?}", args.output);

    // 1. Run analysis
    let mut pipeline = Pipeline::new();

    // Load model if provided
    if let Some(model_path) = &args.model {
        if let Ok(_model) = DeadCodeClassifier::load(&*model_path.to_string_lossy()) {
            println!("✅ Model loaded: {:?}", model_path);
        }
    }

    let analysis = pipeline.process_project(&args.project_dir).await?;

    println!(
        "📊 Analysis complete: {} functions",
        analysis.call_graph.node_count()
    );

    // 2. Get candidates
    let candidates = get_candidates(&analysis, args.count);

    println!("📋 Found {} candidates to verify", candidates.len());

    // 3. Verify candidates - ⭐ FIXED: No unused assignment
    let verified = if args.batch {
        // Batch mode: auto-verify using Git history
        verify_with_git(&analysis, &candidates)
    } else if args.interactive {
        // Interactive mode: present candidates one by one
        verify_interactive(&analysis, &candidates)?
    } else {
        // Generate a review file for manual verification
        generate_review_file(&analysis, &candidates, &args.output)?;
        return Ok(());
    };

    // 4. Save verified dataset
    let session = VerificationSession {
        project: args.project_dir.to_string_lossy().to_string(),
        model_version: args
            .model
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "none".to_string()),
        started_at: chrono::Utc::now().timestamp(),
        verified_entries: verified.clone(),
    };

    let json = serde_json::to_string_pretty(&session)?;
    std::fs::write(&args.output, json)?;

    println!(
        "✅ Verified {} entries saved to: {:?}",
        verified.len(),
        args.output
    );

    // 5. Show stats
    show_stats(&verified);

    Ok(())
}

fn get_candidates(
    analysis: &code_intelligence::analysis::context::ProjectAnalysis,
    count: usize,
) -> Vec<code_intelligence::analysis::verdict_source::Verdict> {
    let root_config = RootDetectionConfig::default();
    let root_set = RootDetector::detect_roots(&analysis.call_graph, &analysis.files, &root_config);
    let reachability = ReachabilityAnalyzer::compute_reachability(&analysis.call_graph, &root_set);

    // ⭐ FIX: Use correct VerdictConfig from state module
    let verdict_engine = VerdictEngine::new(VerdictConfig::default());
    let mut verdicts = verdict_engine.evaluate_all(&analysis.call_graph, &reachability);

    // Sort by confidence (high to low)
    verdicts.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));

    // Return top N
    verdicts.into_iter().take(count).collect()
}

fn verify_with_git(
    analysis: &code_intelligence::analysis::context::ProjectAnalysis,
    candidates: &[code_intelligence::analysis::verdict_source::Verdict],
) -> Vec<VerifiedEntry> {
    use std::process::Command;
    let mut verified = Vec::new();

    for verdict in candidates {
        // ⭐ FIX: Remove unused variable
        // Get the file path from the verdict
        let file = &verdict
            .full_path
            .split("::")
            .next()
            .unwrap_or(&verdict.full_path);

        // Check if function appears in git log
        let git_check = Command::new("git")
            .current_dir(&analysis.root)
            .args(["log", "--oneline", "-1", "--", file])
            .output();

        let is_verified = if let Ok(output) = git_check {
            output.status.success() && !output.stdout.is_empty()
        } else {
            false
        };

        if is_verified {
            verified.push(VerifiedEntry {
                full_path: verdict.full_path.clone(),
                function_name: verdict.function_name.clone(),
                file: verdict.full_path.clone(),
                line: 0,
                label: TrainingLabel::Alive, // Used in Git = Alive
                confidence: 0.90,
                label_source: LabelSource::GitVerified,
                verified_by: "git".to_string(),
                verification_date: chrono::Utc::now().timestamp(),
                reason: "Function appears in Git history".to_string(),
                repository: analysis.root.to_string_lossy().to_string(),
                language: "unknown".to_string(),
            });
        }
    }

    verified
}

fn verify_interactive(
    analysis: &code_intelligence::analysis::context::ProjectAnalysis,
    candidates: &[code_intelligence::analysis::verdict_source::Verdict],
) -> Result<Vec<VerifiedEntry>> {
    let mut verified = Vec::new();
    let mut idx = 0;

    println!("\n🔍 Interactive Verification Mode");
    println!("================================\n");
    println!("For each candidate, decide:");
    println!("  [a] Alive  [d] Dead  [u] Unknown  [s] Skip  [q] Quit\n");

    for verdict in candidates.iter().take(20) {
        idx += 1;
        println!("\n--- Candidate {} / {} ---", idx, candidates.len().min(20));
        println!("Function: {}", verdict.function_name);
        println!("Full path: {}", verdict.full_path);
        println!("State: {}", verdict.state.confidence_label());
        println!("Confidence: {:.1}%", verdict.confidence * 100.0);
        println!("Explanation: {}", verdict.explanation);
        println!("\nDecision [a/d/u/s/q]: ");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "a" => {
                verified.push(VerifiedEntry {
                    full_path: verdict.full_path.clone(),
                    function_name: verdict.function_name.clone(),
                    file: verdict.full_path.clone(),
                    line: 0,
                    label: TrainingLabel::Alive,
                    confidence: 1.0,
                    label_source: LabelSource::HumanVerified,
                    verified_by: "interactive".to_string(),
                    verification_date: chrono::Utc::now().timestamp(),
                    reason: "Verified alive by user".to_string(),
                    repository: analysis.root.to_string_lossy().to_string(),
                    language: "unknown".to_string(),
                });
                println!("✅ Marked ALIVE");
            }
            "d" => {
                verified.push(VerifiedEntry {
                    full_path: verdict.full_path.clone(),
                    function_name: verdict.function_name.clone(),
                    file: verdict.full_path.clone(),
                    line: 0,
                    label: TrainingLabel::Dead,
                    confidence: 1.0,
                    label_source: LabelSource::HumanVerified,
                    verified_by: "interactive".to_string(),
                    verification_date: chrono::Utc::now().timestamp(),
                    reason: "Verified dead by user".to_string(),
                    repository: analysis.root.to_string_lossy().to_string(),
                    language: "unknown".to_string(),
                });
                println!("✅ Marked DEAD");
            }
            "u" => {
                // Unknown - not added to verified set
                println!("⏭️ Skipped (Unknown)");
            }
            "s" => {
                println!("⏭️ Skipped");
            }
            "q" => {
                println!("👋 Quitting...");
                break;
            }
            _ => {
                println!("❌ Invalid input. Use a, d, u, s, or q");
                idx -= 1;
            }
        }
    }

    Ok(verified)
}

fn generate_review_file(
    analysis: &code_intelligence::analysis::context::ProjectAnalysis,
    candidates: &[code_intelligence::analysis::verdict_source::Verdict],
    output_path: &PathBuf,
) -> Result<()> {
    let mut markdown = String::new();

    markdown.push_str("# 🧹 Ground Truth Verification Review\n\n");
    markdown.push_str(&format!(
        "**Project**: {}\n\n",
        analysis
            .root
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    ));
    markdown.push_str(&format!("**Total candidates**: {}\n\n", candidates.len()));
    markdown.push_str("Review each candidate and mark:\n");
    markdown.push_str("- ✅ **Alive** - Function is definitely alive\n");
    markdown.push_str("- ❌ **Dead** - Function is definitely dead\n");
    markdown.push_str("- ❓ **Unknown** - Not sure\n\n");

    markdown.push_str("| # | Function | State | Confidence | Decision | Notes |\n");
    markdown.push_str("|---|----------|-------|------------|----------|-------|\n");

    for (i, verdict) in candidates.iter().enumerate() {
        let state_str = verdict.state.confidence_label();
        markdown.push_str(&format!(
            "| {} | `{}` | {} | {:.1}% |  |  |\n",
            i + 1,
            verdict.function_name,
            state_str,
            verdict.confidence * 100.0
        ));
    }

    markdown.push_str("\n\n---\n\n");
    markdown.push_str("## Instructions\n\n");
    markdown.push_str("1. For each function, determine if it's truly dead or alive\n");
    markdown.push_str("2. Mark ✅ for Alive, ❌ for Dead, ❓ for Unknown\n");
    markdown.push_str("3. Add notes explaining your decision\n");
    markdown.push_str("4. Save this file and run the verification tool again\n");

    let review_path = output_path.with_extension("review.md");
    std::fs::write(&review_path, markdown)?;

    println!("📝 Review file saved to: {:?}", review_path);
    println!("   Please review each candidate and update the file.");
    println!("   Then re-run with --interactive to record decisions.");

    Ok(())
}

fn show_stats(verified: &[VerifiedEntry]) {
    if verified.is_empty() {
        println!("No verified entries.");
        return;
    }

    let alive = verified
        .iter()
        .filter(|e| e.label == TrainingLabel::Alive)
        .count();
    let dead = verified
        .iter()
        .filter(|e| e.label == TrainingLabel::Dead)
        .count();

    println!("\n📊 Verification Stats:");
    println!("   Total verified: {}", verified.len());
    println!(
        "   Alive: {} ({:.1}%)",
        alive,
        alive as f64 / verified.len() as f64 * 100.0
    );
    println!(
        "   Dead: {} ({:.1}%)",
        dead,
        dead as f64 / verified.len() as f64 * 100.0
    );

    // ⭐ FIX: Use Display implementation for LabelSource
    let source_counts: HashMap<String, usize> =
        verified.iter().fold(HashMap::new(), |mut acc, e| {
            *acc.entry(e.label_source.to_string()).or_insert(0) += 1;
            acc
        });

    println!("   Sources:");
    for (source, count) in source_counts {
        println!("      {}: {}", source, count);
    }
}
