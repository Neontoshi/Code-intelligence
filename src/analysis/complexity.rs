// src/analysis/complexity.rs

use crate::graph::call_graph::CallGraph;
use crate::parser::tree_sitter::ParsedFile;

pub struct ComplexityAnalyzer;

impl ComplexityAnalyzer {
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
