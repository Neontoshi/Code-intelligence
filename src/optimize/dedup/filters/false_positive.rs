// src/optimize/dedup/filters/false_positive.rs

use crate::graph::call_graph::FunctionNode;
use crate::optimize::dedup::core::SourceIndex;

pub struct FalsePositiveFilter;

impl FalsePositiveFilter {
    /// Filter out trivial functions, getters/setters, constructors, and standard trait impls
    pub fn is_likely_false_positive(func: &FunctionNode, source: Option<&str>) -> bool {
        // 1. Standard boilerplate function names across Rust / OOP
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
            "to_feature_vector",
            "node_count",
            "edge_count",
            "iter_nodes",
            "iter_edges",
            "file_paths",
            "function_names",
            "file_count",
            "function_count",
            "call_edge_count",
        ];

        if boilerplate_names.contains(&func.name.as_str()) {
            return true;
        }

        // 2. Trait implementations are standard language requirements
        if func.trait_impl.is_some() {
            return true;
        }

        // 3. Simple builder chaining methods (e.g. with_config, with_timeout)
        if func.name.starts_with("with_") {
            return true;
        }

        // 4. Simple accessors / flags
        if func.name.starts_with("get_")
            || func.name.starts_with("set_")
            || func.name.starts_with("is_")
            || func.name.starts_with("has_")
            || func.name.starts_with("push_")
        {
            // If body is short, it's definitely boilerplate
            if let Some(src) = source {
                if src.lines().count() <= 6 || src.split_whitespace().count() <= 15 {
                    return true;
                }
            } else {
                return true;
            }
        }

        // 5. Check body size using actual source text
        if let Some(src) = source {
            let line_count = src.lines().count();
            let token_count = src.split_whitespace().count();

            // Ignore functions with fewer than 5 lines or fewer than 18 tokens
            if line_count < 5 || token_count < 18 {
                return true;
            }
        }

        false
    }

    /// Check if an entire group consists of false positives
    pub fn filter_duplicate_group(group: &[FunctionNode], sources: Option<&SourceIndex>) -> bool {
        if group.len() < 2 {
            return true;
        }

        // If any function in the group is boilerplate, dismiss the group
        for func in group {
            let src = sources.and_then(|s| s.get(&func.full_path));
            if Self::is_likely_false_positive(func, src) {
                return true;
            }
        }

        false
    }
}
