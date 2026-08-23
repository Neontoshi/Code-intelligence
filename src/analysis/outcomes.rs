use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Outcome of a dead code verdict
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerdictOutcome {
    /// Function was removed
    Removed,
    /// Function was kept (false positive or intentionally kept)
    Kept,
    /// Function is pending review
    Pending,
}

/// A tracked verdict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedVerdict {
    pub id: String,
    pub function_name: String,
    pub full_path: String,
    pub file: String,
    pub line: usize,
    pub confidence: f64,
    pub project: String,
    pub verdict_date: u64,
    pub outcome: VerdictOutcome,
    pub outcome_date: Option<u64>,
    pub notes: Option<String>,
    pub removed_commit: Option<String>,
}

/// Statistics about tracked verdicts
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OutcomeStats {
    pub total_flagged: usize,
    pub removed_count: usize,
    pub kept_count: usize,
    pub pending_count: usize,
    pub false_positive_count: usize,
    pub removal_rate: f64,
}

impl OutcomeStats {
    pub fn from_verdicts(verdicts: &[TrackedVerdict]) -> Self {
        let total = verdicts.len();
        let removed = verdicts
            .iter()
            .filter(|v| v.outcome == VerdictOutcome::Removed)
            .count();
        let kept = verdicts
            .iter()
            .filter(|v| v.outcome == VerdictOutcome::Kept)
            .count();
        let pending = verdicts
            .iter()
            .filter(|v| v.outcome == VerdictOutcome::Pending)
            .count();
        let false_positives = verdicts
            .iter()
            .filter(|v| {
                v.outcome == VerdictOutcome::Kept
                    && v.notes
                        .as_ref()
                        .map(|n| n.contains("false positive"))
                        .unwrap_or(false)
            })
            .count();

        Self {
            total_flagged: total,
            removed_count: removed,
            kept_count: kept,
            pending_count: pending,
            false_positive_count: false_positives,
            removal_rate: if total > 0 {
                removed as f64 / total as f64
            } else {
                0.0
            },
        }
    }
}

// Outcome Tracker

pub struct OutcomeTracker {
    verdicts: Vec<TrackedVerdict>,
    storage_path: PathBuf,
}

impl OutcomeTracker {
    pub fn new(project_root: &std::path::Path) -> Self {
        let storage_path = project_root.join(".code-intelligence-outcomes.json");
        let verdicts = Self::load_from_file(&storage_path).unwrap_or_default();

        Self {
            verdicts,
            storage_path,
        }
    }

    /// Load verdicts from storage
    fn load_from_file(path: &PathBuf) -> Result<Vec<TrackedVerdict>, String> {
        if !path.exists() {
            return Ok(Vec::new());
        }

        let data = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read outcomes file: {}", e))?;

        let verdicts: Vec<TrackedVerdict> =
            serde_json::from_str(&data).map_err(|e| format!("Failed to parse outcomes: {}", e))?;

        Ok(verdicts)
    }

    /// Save verdicts to storage
    fn save(&self) -> Result<(), String> {
        let data = serde_json::to_string_pretty(&self.verdicts)
            .map_err(|e| format!("Failed to serialize outcomes: {}", e))?;

        std::fs::write(&self.storage_path, data)
            .map_err(|e| format!("Failed to write outcomes: {}", e))?;

        Ok(())
    }

    ///  Generate a stable ID for a function
    fn generate_stable_id(
        project: &str,
        full_path: &str,
        function_name: &str,
        line: usize,
    ) -> String {
        use sha2::{Digest, Sha256};

        // Use a stable hash based on the function's identity
        // This stays the same across commits as long as the function exists
        let mut hasher = Sha256::new();
        hasher.update(project.as_bytes());
        hasher.update(b"::");
        hasher.update(full_path.as_bytes());
        hasher.update(b"::");
        hasher.update(function_name.as_bytes());
        hasher.update(b"::");
        hasher.update(line.to_string().as_bytes());

        // Include signature hash if available via full_path parsing
        if let Some(sig_start) = full_path.rfind("::") {
            if let Some(sig_end) = full_path.rfind('(') {
                let signature = &full_path[sig_start + 2..sig_end];
                hasher.update(b"::sig::");
                hasher.update(signature.as_bytes());
            }
        }

        let hash = hex::encode(hasher.finalize());
        // Use first 16 chars for readable IDs
        format!("outcome_{}", &hash[..16])
    }

    ///  Get the stable ID for a function without tracking it
    pub fn get_stable_id(
        project: &str,
        full_path: &str,
        function_name: &str,
        line: usize,
    ) -> String {
        Self::generate_stable_id(project, full_path, function_name, line)
    }

    ///  Find a verdict by its stable ID
    pub fn find_by_id(&self, id: &str) -> Option<&TrackedVerdict> {
        self.verdicts.iter().find(|v| v.id == id)
    }

    ///  Find a verdict by function identity
    pub fn find_by_function(
        &self,
        project: &str,
        full_path: &str,
        function_name: &str,
        line: usize,
    ) -> Option<&TrackedVerdict> {
        let id = Self::generate_stable_id(project, full_path, function_name, line);
        self.find_by_id(&id)
    }

    /// Track a new verdict - uses stable ID based on function identity
    pub fn track_verdict(
        &mut self,
        function_name: &str,
        full_path: &str,
        file: &str,
        line: usize,
        confidence: f64,
        project: &str,
    ) -> Result<String, String> {
        // Generate stable ID based on function identity
        let id = Self::generate_stable_id(project, full_path, function_name, line);

        // Check if this verdict already exists (update instead of duplicate)
        if let Some(existing) = self.verdicts.iter_mut().find(|v| v.id == id) {
            // Update existing verdict with new confidence and date
            existing.confidence = confidence;
            existing.verdict_date = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            // If it was previously resolved, reset to pending if new analysis
            if existing.outcome != VerdictOutcome::Pending {
                existing.outcome = VerdictOutcome::Pending;
                existing.outcome_date = None;
                existing.notes = Some("Re-evaluated by new analysis".to_string());
            }
            self.save()?;
            return Ok(id);
        }

        let verdict = TrackedVerdict {
            id: id.clone(),
            function_name: function_name.to_string(),
            full_path: full_path.to_string(),
            file: file.to_string(),
            line,
            confidence,
            project: project.to_string(),
            verdict_date: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            outcome: VerdictOutcome::Pending,
            outcome_date: None,
            notes: None,
            removed_commit: None,
        };

        self.verdicts.push(verdict);
        self.save()?;
        Ok(id)
    }

    /// Update the outcome of a verdict
    pub fn update_outcome(
        &mut self,
        id: &str,
        outcome: VerdictOutcome,
        notes: Option<String>,
        removed_commit: Option<String>,
    ) -> Result<(), String> {
        for verdict in &mut self.verdicts {
            if verdict.id == id {
                verdict.outcome = outcome;
                verdict.outcome_date = Some(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                );
                verdict.notes = notes;
                verdict.removed_commit = removed_commit;
                self.save()?;
                return Ok(());
            }
        }

        Err(format!("Verdict with id {} not found", id))
    }

    /// Mark a verdict as removed (convenience method)
    pub fn mark_removed(&mut self, id: &str, commit_hash: Option<&str>) -> Result<(), String> {
        self.update_outcome(
            id,
            VerdictOutcome::Removed,
            Some("Removed by developer".to_string()),
            commit_hash.map(|s| s.to_string()),
        )
    }

    /// Mark a verdict as false positive (convenience method)
    pub fn mark_false_positive(&mut self, id: &str, reason: &str) -> Result<(), String> {
        self.update_outcome(
            id,
            VerdictOutcome::Kept,
            Some(format!("False positive: {}", reason)),
            None,
        )
    }

    /// Get all verdicts
    pub fn get_verdicts(&self) -> &[TrackedVerdict] {
        &self.verdicts
    }

    /// Get pending verdicts
    pub fn get_pending(&self) -> Vec<&TrackedVerdict> {
        self.verdicts
            .iter()
            .filter(|v| v.outcome == VerdictOutcome::Pending)
            .collect()
    }

    /// Get removed verdicts
    pub fn get_removed(&self) -> Vec<&TrackedVerdict> {
        self.verdicts
            .iter()
            .filter(|v| v.outcome == VerdictOutcome::Removed)
            .collect()
    }

    /// Get kept verdicts
    pub fn get_kept(&self) -> Vec<&TrackedVerdict> {
        self.verdicts
            .iter()
            .filter(|v| v.outcome == VerdictOutcome::Kept)
            .collect()
    }

    /// Get statistics
    pub fn get_stats(&self) -> OutcomeStats {
        OutcomeStats::from_verdicts(&self.verdicts)
    }

    /// Import verdicts from dead_code_check output and track them
    pub fn import_verdicts(
        &mut self,
        dead_verdicts: &[&crate::analysis::verdict_source::state::Verdict],
        project: &str,
    ) -> Result<usize, String> {
        let mut count = 0;
        for verdict in dead_verdicts {
            // Use stable ID with line number from verdict if available
            let line = verdict.static_score.map(|s| s as usize).unwrap_or(0);
            self.track_verdict(
                &verdict.function_name,
                &verdict.full_path,
                &verdict.full_path,
                line,
                verdict.confidence,
                project,
            )?;
            count += 1;
        }
        Ok(count)
    }

    /// Generate a report
    pub fn generate_report(&self) -> String {
        let stats = self.get_stats();
        let mut output = String::new();

        output.push_str("# 🧹 Dead Code Outcome Report\n\n");
        output.push_str("## 📊 Summary\n\n");
        output.push_str(&format!("- **Total flagged**: {}\n", stats.total_flagged));
        output.push_str(&format!(
            "- **Removed**: {} ({:.1}%)\n",
            stats.removed_count,
            stats.removal_rate * 100.0
        ));
        output.push_str(&format!("- **Kept**: {}\n", stats.kept_count));
        output.push_str(&format!("- **Pending**: {}\n", stats.pending_count));
        output.push_str(&format!(
            "- **False positives**: {}\n\n",
            stats.false_positive_count
        ));

        if stats.total_flagged == 0 {
            output.push_str("No tracked verdicts yet.\n");
            return output;
        }

        output.push_str("## 📋 Pending Reviews\n\n");
        let pending = self.get_pending();
        if pending.is_empty() {
            output.push_str("✅ No pending reviews!\n\n");
        } else {
            output.push_str("| Function | File | Confidence | ID |\n");
            output.push_str("|----------|------|------------|----|\n");
            for v in pending.iter().take(20) {
                let file_short = v.file.split('/').last().unwrap_or(&v.file);
                output.push_str(&format!(
                    "| {} | {} | {:.1}% | `{}` |\n",
                    v.function_name,
                    file_short,
                    v.confidence * 100.0,
                    v.id
                ));
            }
            if pending.len() > 20 {
                output.push_str(&format!("| ... and {} more | | | |\n", pending.len() - 20));
            }
            output.push('\n');
        }

        output.push_str("## ✅ Removed Functions\n\n");
        let removed = self.get_removed();
        if removed.is_empty() {
            output.push_str("No functions removed yet.\n\n");
        } else {
            output.push_str("| Function | File | Confidence |\n");
            output.push_str("|----------|------|------------|\n");
            for v in removed.iter().take(20) {
                let file_short = v.file.split('/').last().unwrap_or(&v.file);
                output.push_str(&format!(
                    "| {} | {} | {:.1}% |\n",
                    v.function_name,
                    file_short,
                    v.confidence * 100.0
                ));
            }
            if removed.len() > 20 {
                output.push_str(&format!("| ... and {} more | | |\n", removed.len() - 20));
            }
            output.push('\n');
        }

        output.push_str("## 📝 False Positives\n\n");
        let kept = self.get_kept();
        if kept.is_empty() {
            output.push_str("No false positives recorded.\n\n");
        } else {
            output.push_str("| Function | File | Confidence | Reason |\n");
            output.push_str("|----------|------|------------|--------|\n");
            for v in kept.iter().take(20) {
                let file_short = v.file.split('/').last().unwrap_or(&v.file);
                let reason = v.notes.as_deref().unwrap_or("Unknown");
                output.push_str(&format!(
                    "| {} | {} | {:.1}% | {} |\n",
                    v.function_name,
                    file_short,
                    v.confidence * 100.0,
                    reason
                ));
            }
            if kept.len() > 20 {
                output.push_str(&format!("| ... and {} more | | | |\n", kept.len() - 20));
            }
            output.push('\n');
        }

        output.push_str("## 💡 Next Steps\n\n");
        if stats.pending_count > 0 {
            output.push_str(&format!(
                "1. Review the **{} pending** functions\n",
                stats.pending_count
            ));
        }
        if stats.removal_rate < 0.5 && stats.total_flagged > 0 {
            output.push_str("2. Check if your model needs recalibration (low removal rate)\n");
        }
        if stats.false_positive_count > stats.total_flagged / 2 {
            output.push_str("3. Model may need retraining (high false positive rate)\n");
        }

        output
    }
}
