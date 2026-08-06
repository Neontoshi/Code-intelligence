use crate::graph::call_graph::FunctionNode;

pub struct FalsePositiveFilter;

impl FalsePositiveFilter {
    /// `source` is optional so every existing call site keeps compiling —
    /// pass `None` if you haven't wired SourceIndex through yet.
    pub fn is_likely_false_positive(func: &FunctionNode, source: Option<&str>) -> bool {
        let name = func.name.as_str();

        // Standard trait implementations
        if Self::is_standard_trait_impl(name) {
            return true;
        }

        // Simple getters/setters
        if Self::is_simple_accessor(name) && func.params.len() <= 1 && func.returns.len() == 1 {
            return true;
        }

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

    fn is_standard_trait_impl(name: &str) -> bool {
        matches!(
            name,
            "from"
                | "into"
                | "try_from"
                | "try_into"
                | "fmt"
                | "display"
                | "debug"
                | "default"
                | "clone"
                | "drop"
                | "from_str"
                | "to_string"
                | "as_ref"
                | "as_mut"
                | "borrow"
                | "serialize"
                | "deserialize"
                | "to_owned"
                | "to_vec"
        )
    }

    fn is_simple_accessor(name: &str) -> bool {
        let lower = name.to_lowercase();
        lower.starts_with("get_")
            || lower.starts_with("set_")
            || lower.starts_with("is_")
            || lower.starts_with("has_")
            || lower.starts_with("with_")
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
