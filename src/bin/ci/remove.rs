// src/bin/ci/remove.rs

use code_intelligence::analysis::outcomes::{OutcomeTracker, VerdictOutcome};
use code_intelligence::error::{err, Result};
use std::path::Path;

pub fn run_remove(path: &Path, name: &str, commit: Option<String>) -> Result<()> {
    let mut tracker = OutcomeTracker::new(path);

    let target_id = tracker
        .get_verdicts()
        .iter()
        .find(|v| v.function_name.contains(name) && matches!(v.outcome, VerdictOutcome::Pending))
        .map(|v| (v.id.clone(), v.function_name.clone()));

    if let Some((id, func_name)) = target_id {
        tracker
            .mark_removed(&id, commit.as_deref())
            .map_err(|e| err::internal(e))?;
        println!("✅ Marked '{}' as removed", func_name);
    } else {
        println!("⚠️ No pending function found matching '{}'", name);
    }

    Ok(())
}
