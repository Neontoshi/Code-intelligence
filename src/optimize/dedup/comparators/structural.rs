// src/optimize/dedup/comparators/structural.rs

use crate::graph::call_graph::FunctionNode;
use crate::optimize::dedup::types::{FileContext, SimilarityScores};
use crate::utils::string_utils::levenshtein_ratio;
use std::path::Path;

pub struct StructuralComparator;

impl StructuralComparator {
    pub fn compare(a: &FunctionNode, b: &FunctionNode) -> SimilarityScores {
        let mut scores = SimilarityScores {
            structural: 0.0,
            semantic: 0.0,
            call_graph: 0.0,
            ast: 0.0,
            data_flow: 0.0,
            ml: 0.0,
            context: 0.0,
        };

        // Parameter similarity (20%)
        if a.params.len() == b.params.len() {
            scores.structural += 0.20;
        } else {
            let diff = (a.params.len() as i32 - b.params.len() as i32).abs();
            scores.structural += (1.0 - (diff as f64 / 10.0)).max(0.0) * 0.20;
        }

        // Return type similarity (15%)
        if a.returns == b.returns {
            scores.structural += 0.15;
        } else if a.returns.is_empty() && b.returns.is_empty() {
            scores.structural += 0.15;
        } else if !a.returns.is_empty() && !b.returns.is_empty() {
            let common = a.returns.iter().filter(|r| b.returns.contains(r)).count();
            scores.structural +=
                (common as f64 / a.returns.len().max(b.returns.len()) as f64) * 0.15;
        }

        // Complexity similarity (10%)
        let comp_diff = (a.complexity - b.complexity).abs();
        if comp_diff <= 1.0 {
            scores.structural += 0.10;
        } else if comp_diff <= 3.0 {
            scores.structural += 0.07;
        } else if comp_diff <= 5.0 {
            scores.structural += 0.03;
        }

        // Public/Async signature (10%)
        if a.is_public == b.is_public {
            scores.structural += 0.05;
        }
        if a.is_async == b.is_async {
            scores.structural += 0.05;
        }

        // Name similarity (45%) — heavily penalize different names
        let name_sim = levenshtein_ratio(&a.name, &b.name);
        scores.structural += name_sim * 0.45;

        // Context similarity (bonus)
        scores.context = Self::file_context_similarity(a, b);

        scores
    }

    fn file_context_similarity(a: &FunctionNode, b: &FunctionNode) -> f64 {
        let path_a = Path::new(&a.file);
        let path_b = Path::new(&b.file);

        let components_a: Vec<_> = path_a.components().collect();
        let components_b: Vec<_> = path_b.components().collect();

        let common = components_a
            .iter()
            .zip(&components_b)
            .filter(|(x, y)| x == y)
            .count();

        let max_len = components_a.len().max(components_b.len());
        if max_len == 0 {
            1.0
        } else {
            common as f64 / max_len as f64
        }
    }

    pub fn get_file_context(func: &FunctionNode) -> FileContext {
        let path = Path::new(&func.file);

        FileContext {
            directory: path
                .parent()
                .unwrap_or(Path::new(""))
                .to_string_lossy()
                .to_string(),
            filename: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            extension: path
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            module_path: Self::extract_module_path(&func.file),
        }
    }

    fn extract_module_path(file: &str) -> String {
        let parts: Vec<&str> = file.split('/').collect();
        if parts.len() >= 2 {
            let start = if parts.len() >= 3 { parts.len() - 2 } else { 0 };
            parts[start..parts.len() - 1].join("/")
        } else {
            String::new()
        }
    }
}
