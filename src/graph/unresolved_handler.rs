// src/graph/unresolved_handler.rs

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct UnresolvedEdge {
    pub caller: String,
    pub callee: String,
    pub line: usize,
    pub reason: UnresolvedReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UnresolvedReason {
    FunctionNotFound,
    AmbiguousMatch,
    DynamicDispatch,
    Reflection,
    FFI,
    MacroGenerated,
    Unknown,
}

pub struct UnresolvedHandler {
    unresolved: Vec<UnresolvedEdge>,
    stats: UnresolvedStats,
}

#[derive(Debug, Clone, Default)]
pub struct UnresolvedStats {
    pub total: usize,
    pub by_reason: HashMap<UnresolvedReason, usize>,
    pub by_file: HashMap<String, usize>,
    pub by_function: HashMap<String, usize>,
}

impl UnresolvedHandler {
    pub fn new() -> Self {
        Self {
            unresolved: Vec::new(),
            stats: UnresolvedStats::default(),
        }
    }

    pub fn track_unresolved(&mut self, edge: UnresolvedEdge) {
        self.unresolved.push(edge.clone());
        self.stats.total += 1;
        *self.stats.by_reason.entry(edge.reason).or_insert(0) += 1;
        *self.stats.by_file.entry(edge.caller.clone()).or_insert(0) += 1;
        *self
            .stats
            .by_function
            .entry(edge.caller.clone())
            .or_insert(0) += 1;
    }

    pub fn get_unresolved(&self) -> &[UnresolvedEdge] {
        &self.unresolved
    }

    pub fn get_stats(&self) -> &UnresolvedStats {
        &self.stats
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str("## 🔍 Unresolved Call Report\n\n");
        report.push_str(&format!("Total unresolved: {}\n\n", self.stats.total));

        report.push_str("### By Reason\n\n");
        for (reason, count) in &self.stats.by_reason {
            report.push_str(&format!("- {:?}: {}\n", reason, count));
        }

        report.push_str("\n### Top Files with Unresolved Calls\n\n");
        let mut files: Vec<_> = self.stats.by_file.iter().collect();
        files.sort_by(|a, b| b.1.cmp(a.1));
        for (file, count) in files.iter().take(10) {
            report.push_str(&format!("- {}: {}\n", file, count));
        }

        report.push_str("\n### Sample Unresolved Edges\n\n");
        for edge in self.unresolved.iter().take(10) {
            report.push_str(&format!(
                "- {} → {} (line {}): {:?}\n",
                edge.caller, edge.callee, edge.line, edge.reason
            ));
        }

        report
    }

    /// Analyze unresolved edges and suggest fixes
    pub fn suggest_fixes(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        let ffi_count = self
            .stats
            .by_reason
            .get(&UnresolvedReason::FFI)
            .unwrap_or(&0);
        if *ffi_count > 0 {
            suggestions.push(format!(
                "{} FFI calls unresolved - consider adding FFI stubs or using bindgen",
                ffi_count
            ));
        }

        let dynamic_count = self
            .stats
            .by_reason
            .get(&UnresolvedReason::DynamicDispatch)
            .unwrap_or(&0);
        if *dynamic_count > 0 {
            suggestions.push(format!(
                "{} dynamic dispatch calls unresolved - consider adding type information",
                dynamic_count
            ));
        }

        let reflection_count = self
            .stats
            .by_reason
            .get(&UnresolvedReason::Reflection)
            .unwrap_or(&0);
        if *reflection_count > 0 {
            suggestions.push(format!(
                "{} reflection calls unresolved - consider adding runtime type tracking",
                reflection_count
            ));
        }

        if suggestions.is_empty() {
            suggestions.push("✅ No unresolved call issues detected.".to_string());
        }

        suggestions
    }
}

impl Default for UnresolvedHandler {
    fn default() -> Self {
        Self::new()
    }
}
