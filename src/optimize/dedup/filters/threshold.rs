// src/optimize/dedup/filters/threshold.rs

use crate::graph::call_graph::FunctionNode;
use crate::optimize::dedup::types::DedupConfig;

#[derive(Debug, Clone)]
pub struct ThresholdFilter {
    pub min_similarity: f64,
    pub min_lines: usize,
}

impl ThresholdFilter {
    pub fn new(min_similarity: f64) -> Self {
        Self {
            min_similarity,
            min_lines: 4,
        }
    }

    pub fn with_min_lines(mut self, min_lines: usize) -> Self {
        self.min_lines = min_lines;
        self
    }

    pub fn passes_threshold(&self, similarity: f64) -> bool {
        similarity >= self.min_similarity
    }
}

pub struct ThresholdTuner {
    pub config: DedupConfig,
}

impl ThresholdTuner {
    pub fn new(config: DedupConfig) -> Self {
        Self { config }
    }

    pub fn auto_tune(functions: &[FunctionNode]) -> f64 {
        if functions.is_empty() {
            return 0.85;
        }

        let avg_loc: f64 = functions
            .iter()
            .map(|f| {
                if f.body_end_line >= f.body_start_line {
                    f.body_end_line - f.body_start_line + 1
                } else {
                    1
                }
            })
            .sum::<usize>() as f64
            / functions.len() as f64;

        if avg_loc > 50.0 {
            0.80
        } else if avg_loc > 20.0 {
            0.85
        } else {
            0.88
        }
    }

    pub fn tune(&self, base_threshold: f64) -> f64 {
        base_threshold.clamp(0.0, 1.0)
    }
}

/// Filter out trivial 1-3 line functions, standard trait implementations, and common getters/builders
pub fn is_actionable_duplicate_candidate(func: &FunctionNode) -> bool {
    let line_count = if func.body_end_line >= func.body_start_line {
        func.body_end_line - func.body_start_line + 1
    } else {
        1
    };

    // 1. Skip functions that are too short to be worth deduplicating
    if line_count < 4 {
        return false;
    }

    // 2. Ignore standard boilerplate trait methods and common constructors
    let boilerplate_names = [
        "default",
        "clone",
        "fmt",
        "from",
        "into",
        "try_from",
        "try_into",
        "new",
        "len",
        "is_empty",
        "reset",
        "clear",
        "as_ref",
        "as_mut",
        "drop",
        "to_string",
        "to_json",
        "to_jsonl",
        "to_markdown",
    ];
    if boilerplate_names.contains(&func.name.as_str()) {
        return false;
    }

    // 3. Ignore simple builder chaining methods (e.g. with_config, with_timeout)
    if func.name.starts_with("with_") && line_count <= 5 {
        return false;
    }

    // 4. Ignore simple getters/setters/checkers
    if (func.name.starts_with("get_")
        || func.name.starts_with("set_")
        || func.name.starts_with("is_")
        || func.name.starts_with("has_"))
        && line_count <= 5
    {
        return false;
    }

    true
}
