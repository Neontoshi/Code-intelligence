// src/analysis/complexity.rs

use crate::graph::call_graph::{CallGraph, FunctionNode};
use crate::parser::tree_sitter::ParsedFile;

pub struct ComplexityAnalyzer;

impl ComplexityAnalyzer {
    /// Calculate cyclomatic complexity for a function based on control flow
    pub fn calculate_complexity(_func: &FunctionNode, source: &str) -> f64 {
        let mut complexity = 1.0;

        // Count control flow structures
        let patterns = [
            ("if ", 0.5),
            ("else ", 0.3),
            ("for ", 0.5),
            ("while ", 0.5),
            ("match ", 0.5),
            ("switch ", 0.5),
            ("case ", 0.2),
            ("&&", 0.2),
            ("||", 0.2),
            ("?", 0.3),
            ("catch ", 0.3),
            ("try ", 0.2),
        ];

        for (pattern, weight) in patterns {
            let count = source.matches(pattern).count();
            complexity += count as f64 * weight;
        }

        // Function length impact
        let lines = source.lines().count();
        if lines > 20 {
            complexity += (lines - 20) as f64 * 0.05;
        }

        // Cap at reasonable maximum
        complexity.min(50.0)
    }

    /// Calculate overall project complexity metrics
    pub fn project_complexity(call_graph: &CallGraph, files: &[ParsedFile]) -> ProjectComplexity {
        let mut complexities = Vec::new();
        let mut total_lines = 0;

        for file in files {
            total_lines += file.source.lines().count();
        }

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            let complexity = func.complexity;
            complexities.push(complexity);
        }

        let avg_complexity = if !complexities.is_empty() {
            complexities.iter().sum::<f64>() / complexities.len() as f64
        } else {
            0.0
        };

        let max_complexity: f64 = complexities.iter().fold(0.0, |a: f64, &b| a.max(b));

        ProjectComplexity {
            average_complexity: avg_complexity,
            max_complexity,
            total_functions: complexities.len(),
            total_lines,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectComplexity {
    pub average_complexity: f64,
    pub max_complexity: f64,
    pub total_functions: usize,
    pub total_lines: usize,
}
