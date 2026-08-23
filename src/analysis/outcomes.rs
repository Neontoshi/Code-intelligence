use crate::analysis::training_data::FunctionFeatures;
use crate::analysis::verdict_source::LabelSource;
use crate::analysis::{TrainingExample, TrainingLabel};
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

/// Feedback statistics for ML improvement
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeedbackStats {
    pub total_decisions: usize,
    pub removed: usize,
    pub kept: usize,
    pub pending: usize,
    pub false_positives: usize,
    pub true_positives: usize,
    pub true_negatives: usize,
    pub feedback_ratio: f64,
    pub false_positive_rate: f64,
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
    /// ⭐ NEW: Export decisions as training examples
    pub fn export_decisions_as_training_data(&self) -> Vec<TrainingExample> {
        let mut examples = Vec::new();

        for verdict in &self.verdicts {
            // Only include decisions that are final (Removed or Kept)
            if verdict.outcome == VerdictOutcome::Removed {
                // Function was removed → it was truly dead
                let example = TrainingExample {
                    function_name: verdict.function_name.clone(),
                    full_path: verdict.full_path.clone(),
                    file: verdict.file.clone(),
                    language: self.detect_language(&verdict.file),
                    features: self.create_features_from_verdict(verdict),
                    label: TrainingLabel::Dead,
                    confidence: verdict.confidence,
                    source: "dashboard_decision".to_string(),
                    repository_id: Some(verdict.project.clone()),
                    commit_hash: verdict.removed_commit.clone(),
                    dataset_split: Some("verified".to_string()),
                    label_reason: Some("Removed by developer via dashboard".to_string()),
                    label_version: Some(2),
                    label_source: LabelSource::HumanVerified,
                    generated_by_model: None,
                    verified_by: Some(verdict.notes.clone().unwrap_or_default()),
                    created_at: Some(verdict.outcome_date.unwrap_or(0) as i64),
                };
                examples.push(example);
            } else if verdict.outcome == VerdictOutcome::Kept {
                // Function was kept → it's alive (false positive)
                let is_false_positive = verdict
                    .notes
                    .as_ref()
                    .map(|n| n.contains("false positive"))
                    .unwrap_or(false);

                if is_false_positive {
                    let example = TrainingExample {
                        function_name: verdict.function_name.clone(),
                        full_path: verdict.full_path.clone(),
                        file: verdict.file.clone(),
                        language: self.detect_language(&verdict.file),
                        features: self.create_features_from_verdict(verdict),
                        label: TrainingLabel::Alive,
                        confidence: verdict.confidence,
                        source: "dashboard_decision".to_string(),
                        repository_id: Some(verdict.project.clone()),
                        commit_hash: None,
                        dataset_split: Some("verified".to_string()),
                        label_reason: Some("Kept as false positive via dashboard".to_string()),
                        label_version: Some(2),
                        label_source: LabelSource::HumanVerified,
                        generated_by_model: None,
                        verified_by: Some(verdict.notes.clone().unwrap_or_default()),
                        created_at: Some(verdict.outcome_date.unwrap_or(0) as i64),
                    };
                    examples.push(example);
                }
            }
        }

        examples
    }

    /// ⭐ NEW: Get feedback statistics
    pub fn get_feedback_stats(&self) -> FeedbackStats {
        let total_decisions = self.verdicts.len();
        let removed = self
            .verdicts
            .iter()
            .filter(|v| v.outcome == VerdictOutcome::Removed)
            .count();
        let kept = self
            .verdicts
            .iter()
            .filter(|v| v.outcome == VerdictOutcome::Kept)
            .count();
        let pending = self
            .verdicts
            .iter()
            .filter(|v| v.outcome == VerdictOutcome::Pending)
            .count();

        let false_positives = self
            .verdicts
            .iter()
            .filter(|v| {
                v.outcome == VerdictOutcome::Kept
                    && v.notes
                        .as_ref()
                        .map(|n| n.contains("false positive"))
                        .unwrap_or(false)
            })
            .count();

        let true_positives = removed; // Removed = correctly identified as dead
        let true_negatives = self
            .verdicts
            .iter()
            .filter(|v| {
                v.outcome == VerdictOutcome::Kept
                    && !v
                        .notes
                        .as_ref()
                        .map(|n| n.contains("false positive"))
                        .unwrap_or(false)
            })
            .count();

        FeedbackStats {
            total_decisions,
            removed,
            kept,
            pending,
            false_positives,
            true_positives,
            true_negatives,
            feedback_ratio: if total_decisions > 0 {
                (removed + kept) as f64 / total_decisions as f64
            } else {
                0.0
            },
            false_positive_rate: if kept > 0 {
                false_positives as f64 / kept as f64
            } else {
                0.0
            },
        }
    }

    /// ⭐ NEW: Create features from a verdict (for training examples)
    fn create_features_from_verdict(&self, verdict: &TrackedVerdict) -> FunctionFeatures {
        // Create a simplified FunctionFeatures from the tracked verdict
        // This is a best-effort reconstruction since we don't have the full AST
        use crate::analysis::training_data::FunctionFeatures;

        let name_lower = verdict.function_name.to_lowercase();
        let is_in_test_file = verdict.file.contains("/tests/")
            || verdict.file.contains("/test/")
            || verdict.file.ends_with("_test.rs");

        FunctionFeatures {
            param_count: 0,
            return_count: 0,
            is_public: false,
            is_async: false,
            name_length: verdict.function_name.len(),
            starts_with_use: verdict.function_name.starts_with("use"),
            starts_with_test: verdict.function_name.starts_with("test_")
                || verdict.function_name.starts_with("Test"),
            starts_with_bench: verdict.function_name.starts_with("bench_")
                || verdict.function_name.starts_with("Benchmark"),
            ends_with_test: verdict.function_name.ends_with("_test"),
            contains_trait_impl: false,
            signature_hash: String::new(),
            body_hash: String::new(),
            fan_in: 0,
            fan_out: 0,
            complexity: 1.0,
            call_depth: 0,
            is_cycle: false,
            file_extension: verdict.file.split('.').last().unwrap_or("").to_string(),
            is_in_test_file,
            is_in_benches: verdict.file.contains("/benches/"),
            is_in_meta: verdict.file.contains("/.meta/"),
            is_in_examples: verdict.file.contains("/examples/"),
            is_generated: verdict.file.contains(".gen.") || verdict.file.contains("_gen."),
            name_contains_use: name_lower.contains("use"),
            name_contains_test: name_lower.contains("test"),
            name_contains_init: name_lower.contains("init"),
            name_contains_get: name_lower.contains("get"),
            name_contains_set: name_lower.contains("set"),
            name_contains_new: name_lower.contains("new"),
            name_contains_create: name_lower.contains("create"),
            name_contains_build: name_lower.contains("build"),
            name_contains_parse: name_lower.contains("parse"),
            name_contains_validate: name_lower.contains("validate"),
            name_contains_handle: name_lower.contains("handle"),
            name_contains_process: name_lower.contains("process"),
            name_contains_convert: name_lower.contains("convert"),
            name_contains_commit: name_lower.contains("commit"),
            name_contains_reveal: name_lower.contains("reveal"),
            name_contains_submit: name_lower.contains("submit"),
            name_contains_upload: name_lower.contains("upload"),
            name_contains_download: name_lower.contains("download"),
            name_contains_fetch: name_lower.contains("fetch"),
            name_contains_verify: name_lower.contains("verify"),
            name_contains_audit: name_lower.contains("audit"),
            type_name: None,
            type_path: None,
            is_method: false,
            is_trait_impl: false,
            trait_name: None,
            is_associated: false,
        }
    }

    /// ⭐ NEW: Detect language from file path
    fn detect_language(&self, file: &str) -> String {
        if file.ends_with(".rs") {
            "rust".to_string()
        } else if file.ends_with(".go") {
            "go".to_string()
        } else if file.ends_with(".py") {
            "python".to_string()
        } else if file.ends_with(".js") || file.ends_with(".jsx") {
            "javascript".to_string()
        } else if file.ends_with(".ts") || file.ends_with(".tsx") {
            "typescript".to_string()
        } else if file.ends_with(".java") {
            "java".to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// ⭐ NEW: Save feedback as training data
    pub fn save_feedback_as_training_data(
        &self,
        output_path: &std::path::Path,
    ) -> Result<(), String> {
        let examples = self.export_decisions_as_training_data();
        if examples.is_empty() {
            return Err("No feedback examples to export".to_string());
        }

        let json = serde_json::to_string_pretty(&examples)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        std::fs::write(output_path, json).map_err(|e| format!("Failed to write: {}", e))?;

        Ok(())
    }
}
