use crate::optimize::dedup::types::DataFlowSignature;
use std::collections::HashSet;

pub struct DataFlowAnalyzer;

impl DataFlowAnalyzer {
    pub fn analyze(source: &str) -> DataFlowSignature {
        let mut reads = HashSet::new();
        let mut writes = HashSet::new();
        let mut transformations = Vec::new();
        let mut async_depth = 0;

        // Detect transformations
        if source.contains(".unwrap()") {
            transformations.push("unwrap".to_string());
        }
        if source.contains(".map(") {
            transformations.push("map".to_string());
        }
        if source.contains(".filter(") {
            transformations.push("filter".to_string());
        }
        if source.contains(".collect::<") {
            transformations.push("collect".to_string());
        }
        if source.contains(".fold(") {
            transformations.push("fold".to_string());
        }
        if source.contains(".reduce(") {
            transformations.push("reduce".to_string());
        }
        if source.contains(".and_then(") {
            transformations.push("and_then".to_string());
        }
        if source.contains(".or_else(") {
            transformations.push("or_else".to_string());
        }

        // Detect async patterns
        if source.contains("await") {
            async_depth += 1;
        }
        if source.contains(".await") {
            async_depth += 1;
        }
        if source.contains("async fn") {
            async_depth += 1;
        }
        if source.contains("tokio::spawn") {
            async_depth += 1;
        }

        // Detect reads/writes (simplified)
        for token in source.split_whitespace() {
            let token = token.trim();
            if token.starts_with("&") || token.starts_with("self.") || token.starts_with("state.") {
                reads.insert(token.to_string());
            }
            if token.contains(".write")
                || token.contains(".set")
                || token.contains(".push")
                || token.contains(".insert")
                || token.contains(".remove")
            {
                writes.insert(token.to_string());
            }
        }

        // Detect error handling patterns
        if source.contains("Result") {
            transformations.push("result".to_string());
        }
        if source.contains("Option") {
            transformations.push("option".to_string());
        }
        if source.contains("?") {
            transformations.push("try".to_string());
        }
        if source.contains("unwrap") {
            transformations.push("unwrap".to_string());
        }

        DataFlowSignature {
            reads: reads.into_iter().collect(),
            writes: writes.into_iter().collect(),
            transformations,
            async_depth,
        }
    }

    pub fn similarity(a: &DataFlowSignature, b: &DataFlowSignature) -> f64 {
        let mut score = 0.0;
        let mut total = 0.0;

        // Read patterns (25%)
        let read_common = a.reads.iter().filter(|r| b.reads.contains(r)).count();
        let read_union = a.reads.len() + b.reads.len() - read_common;
        if read_union > 0 {
            score += (read_common as f64 / read_union as f64) * 0.25;
        }
        total += 0.25;

        // Write patterns (25%)
        let write_common = a.writes.iter().filter(|w| b.writes.contains(w)).count();
        let write_union = a.writes.len() + b.writes.len() - write_common;
        if write_union > 0 {
            score += (write_common as f64 / write_union as f64) * 0.25;
        }
        total += 0.25;

        // Transformations (35%)
        let trans_common = a
            .transformations
            .iter()
            .filter(|t| b.transformations.contains(t))
            .count();
        let trans_union = a.transformations.len() + b.transformations.len() - trans_common;
        if trans_union > 0 {
            score += (trans_common as f64 / trans_union as f64) * 0.35;
        }
        total += 0.35;

        // Async depth (15%)
        let depth_diff = (a.async_depth as i32 - b.async_depth as i32).abs();
        score += (1.0 - (depth_diff as f64 / 5.0).min(1.0)) * 0.15;
        total += 0.15;

        if total > 0.0 {
            score / total
        } else {
            0.0
        }
    }
}
