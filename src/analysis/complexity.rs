use crate::graph::call_graph::{CallGraph, FunctionNode};
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashMap;

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

        let mut _control_flow_count = 0;
        for (pattern, weight) in patterns {
            let count = source.matches(pattern).count();
            _control_flow_count += count;
            complexity += count as f64 * weight;
        }

        // Nesting depth impact
        let nesting_depth = Self::calculate_nesting_depth(source);
        complexity += nesting_depth as f64 * 0.2;

        // Function length impact
        let lines = source.lines().count();
        if lines > 20 {
            complexity += (lines - 20) as f64 * 0.05;
        }

        // Cap at reasonable maximum
        complexity.min(50.0)
    }

    fn calculate_nesting_depth(source: &str) -> usize {
        let mut max_depth = 0;
        let mut current_depth: usize = 0;

        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('{') {
                current_depth += 1;
                max_depth = max_depth.max(current_depth);
            } else if trimmed.starts_with('}') {
                current_depth = current_depth.saturating_sub(1);
            }
        }

        max_depth
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
            // Estimate complexity from function name and parameters
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
            complexity_distribution: Self::distribute_complexities(&complexities),
        }
    }

    fn distribute_complexities(complexities: &[f64]) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();

        for &complexity in complexities {
            let bucket = if complexity <= 5.0 {
                "simple"
            } else if complexity <= 10.0 {
                "moderate"
            } else if complexity <= 20.0 {
                "complex"
            } else {
                "very_complex"
            };

            *distribution.entry(bucket.to_string()).or_insert(0) += 1;
        }

        distribution
    }
}

#[derive(Debug, Clone)]
pub struct ProjectComplexity {
    pub average_complexity: f64,
    pub max_complexity: f64,
    pub total_functions: usize,
    pub total_lines: usize,
    pub complexity_distribution: HashMap<String, usize>,
}
