// src/bin/verify_dead_candidates.rs

//! Utility to help manually verify dead code candidates
//! This creates a checklist for review

use clap::Parser;
use code_intelligence::analysis::training_data::{TrainingExample, TrainingLabel};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Generate review checklist for dead code candidates"
)]
struct Args {
    /// Training data file to verify
    #[arg(short, long, default_value = "data/val.json")]
    data: PathBuf,

    /// Output markdown file for review checklist
    #[arg(short, long, default_value = "review_checklist.md")]
    output: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Check if data file exists
    if !args.data.exists() {
        eprintln!("❌ Data file not found: {:?}", args.data);
        eprintln!("   Run `cargo run --bin merge_all_training_data` first");
        std::process::exit(1);
    }

    let data = std::fs::read_to_string(&args.data)?;
    let examples: Vec<TrainingExample> = serde_json::from_str(&data)?;

    let dead_candidates: Vec<_> = examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Dead)
        .collect();

    if dead_candidates.is_empty() {
        println!("✅ No dead candidates found in validation set!");
        return Ok(());
    }

    let mut markdown = String::new();
    markdown.push_str("# 🧹 Dead Code Verification Checklist\n\n");
    markdown.push_str(&format!(
        "**Review these {} candidates** by:\n\n",
        dead_candidates.len()
    ));
    markdown.push_str("1. 🔍 Search for the function in the codebase\n");
    markdown.push_str("2. 📝 Check if it's imported/used anywhere\n");
    markdown.push_str("3. 🧪 Try removing it in a local branch and run tests\n");
    markdown.push_str("4. ✅ Mark **✅** if truly dead, **❌** if false positive\n");
    markdown.push_str("5. 📝 Add notes about why it's dead or alive\n\n");

    markdown.push_str("| # | Function | File | Confidence | Label Reason | Verified | Notes |\n");
    markdown.push_str("|---|----------|------|------------|--------------|----------|-------|\n");

    for (i, example) in dead_candidates.iter().enumerate() {
        let confidence = example.confidence * 100.0;
        let reason = example.label_reason.as_deref().unwrap_or("unknown");
        let function_name = &example.function_name;
        let file_path = &example.file;

        // Shorten file path for display
        let short_file = file_path.split('/').last().unwrap_or(file_path).to_string();

        markdown.push_str(&format!(
            "| {} | `{}` | `{}` | {:.1}% | {} |  |  |\n",
            i + 1,
            function_name,
            short_file,
            confidence,
            reason
        ));
    }

    markdown.push_str("\n---\n\n");
    markdown.push_str("## 📊 Summary\n\n");
    markdown.push_str(&format!(
        "- **Total candidates to review**: {}\n",
        dead_candidates.len()
    ));
    markdown.push_str("- **Mark as** ✅ (truly dead) or ❌ (false positive)\n");
    markdown.push_str(
        "- **Add notes** about why (e.g., \"used in test\", \"called via reflection\", etc.)\n\n",
    );
    markdown.push_str("### Example Review\n\n");
    markdown.push_str("| # | Function | File | Confidence | Label Reason | Verified | Notes |\n");
    markdown.push_str("|---|----------|------|------------|--------------|----------|-------|\n");
    markdown.push_str(
        "| 1 | `process_data` | `handlers.rs` | 92.5% | root | ✅ | Removed, tests passed |\n",
    );
    markdown.push_str(
        "| 2 | `validate_input` | `validators.rs` | 88.0% | root | ❌ | Used by external SDK |\n",
    );

    std::fs::write(&args.output, markdown)?;
    println!("✅ Review checklist saved to: {:?}", args.output);
    println!("📝 Total candidates to review: {}", dead_candidates.len());
    println!("\n📋 Next steps:");
    println!("   1. Open review_checklist.md");
    println!("   2. For each candidate, verify if it's truly dead");
    println!("   3. Mark ✅ or ❌ and add notes");
    println!("   4. Come back when done to proceed with calibration");

    Ok(())
}
