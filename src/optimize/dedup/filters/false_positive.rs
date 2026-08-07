// src/optimize/dedup/filters/false_positive.rs

use crate::graph::call_graph::FunctionNode;

pub struct FalsePositiveFilter;

impl FalsePositiveFilter {
    pub fn is_likely_false_positive(func: &FunctionNode, source: Option<&str>) -> bool {
        let name = func.name.as_str();

        // Constructor patterns
        if name == "new" && func.params.len() >= 1 {
            return true;
        }

        // Test functions
        if name.starts_with("test_") || name.starts_with("bench_") {
            return true;
        }

        // Main/entry point functions
        if name == "main" || name == "async_main" || name == "run" {
            return true;
        }

        // Trivial bodies: two near-empty wrappers "matching" on shape isn't
        // a meaningful duplicate report even if every other signal agrees.
        if let Some(src) = source {
            let meaningful_lines = src
                .lines()
                .filter(|l| {
                    let t = l.trim();
                    !t.is_empty() && !t.starts_with("//") && !t.starts_with("/*")
                })
                .count();
            if meaningful_lines <= 2 {
                return true;
            }
        }

        false
    }

    pub fn filter_duplicate_group(
        group: &[FunctionNode],
        sources: Option<&crate::optimize::dedup::core::SourceIndex>,
    ) -> bool {
        group
            .iter()
            .all(|f| Self::is_likely_false_positive(f, sources.and_then(|s| s.get(&f.full_path))))
    }
}
