// src/bin/ci/stats.rs

use code_intelligence::analysis::outcomes::OutcomeTracker;
use code_intelligence::error::Result;
use std::path::Path;

pub fn run_stats(path: &Path, detailed: bool) -> Result<()> {
    let tracker = OutcomeTracker::new(path);
    let stats = tracker.get_stats();

    println!("\n📊 Outcome Statistics for: {:?}", path);
    println!("");
    println!("   Total flagged: {}", stats.total_flagged);
    println!(
        "   Removed: {} ({:.1}%)",
        stats.removed_count,
        stats.removal_rate * 100.0
    );
    println!("   Kept (false positives): {}", stats.kept_count);
    println!("   Pending: {}", stats.pending_count);

    if detailed {
        let feedback_stats = tracker.get_feedback_stats();
        println!("\n📈 Detailed Feedback Stats:");
        println!("   Total decisions: {}", feedback_stats.total_decisions);
        println!(
            "   Feedback ratio: {:.1}%",
            feedback_stats.feedback_ratio * 100.0
        );
        println!(
            "   False positive rate: {:.1}%",
            feedback_stats.false_positive_rate * 100.0
        );
    }

    Ok(())
}
