// src/bin/ci/keep.rs

use code_intelligence::analysis::outcomes::{OutcomeTracker, VerdictOutcome};
use code_intelligence::error::Result;
use std::path::Path;

pub fn run_keep(path: &Path, name: &str, reason: &str) -> Result<()> {
    let mut tracker = OutcomeTracker::new(path);

    let target_id = tracker
        .get_verdicts()
        .iter()
        .find(|v| v.function_name.contains(name) && matches!(v.outcome, VerdictOutcome::Pending))
        .map(|v| (v.id.clone(), v.function_name.clone()));

    if let Some((id, func_name)) = target_id {
        tracker
            .mark_false_positive(&id, reason)
            .map_err(|e| anyhow::anyhow!("Internal error: {}", e))?;
        println!("✅ Marked '{}' as false positive", func_name);
    } else {
        println!("⚠️ No pending function found matching '{}'", name);
    }

    Ok(())
}
