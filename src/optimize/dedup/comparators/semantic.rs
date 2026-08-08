// src/optimize/dedup/comparators/semantic.rs

use crate::graph::call_graph::FunctionNode;
use crate::utils::string_utils::levenshtein_ratio;

pub struct SemanticComparator;

impl SemanticComparator {
    pub fn compare(a: &FunctionNode, b: &FunctionNode) -> f64 {
        let purpose_a = Self::infer_purpose(a);
        let purpose_b = Self::infer_purpose(b);

        if purpose_a != purpose_b {
            return 0.0;
        }

        let mut score = 0.0;
        let mut total = 0.0;

        // Parameter similarity (25%)
        let param_diff = (a.params.len() as i32 - b.params.len() as i32).abs();
        if param_diff <= 1 {
            score += 0.25;
        } else if param_diff <= 2 {
            score += 0.12;
        }
        total += 0.25;

        // Return type similarity (15%)
        let return_sim = Self::return_similarity(&a.returns, &b.returns);
        score += return_sim * 0.15;
        total += 0.15;

        // Name similarity (30%) — functions with different names are less likely duplicates
        let name_sim = levenshtein_ratio(&a.name, &b.name);
        score += name_sim * 0.30;
        total += 0.30;

        // Name category check (30%)
        let name_category_a = Self::name_category(&a.name);
        let name_category_b = Self::name_category(&b.name);
        if name_category_a == name_category_b {
            score += 0.30;
        }
        total += 0.30;

        if total > 0.0 {
            score / total
        } else {
            0.0
        }
    }

    fn infer_purpose(func: &FunctionNode) -> String {
        let name = func.name.to_lowercase();

        if name.contains("validate") {
            "validation".to_string()
        } else if name.contains("build") {
            "construction".to_string()
        } else if name.contains("create") {
            "creation".to_string()
        } else if name.contains("update") {
            "update".to_string()
        } else if name.contains("get") {
            "retrieval".to_string()
        } else if name.contains("set") {
            "assignment".to_string()
        } else if name.contains("process") {
            "processing".to_string()
        } else if name.contains("handle") {
            "handling".to_string()
        } else if name.contains("parse") {
            "parsing".to_string()
        } else if name.contains("convert") {
            "conversion".to_string()
        } else if name.contains("init") {
            "initialization".to_string()
        } else if name.contains("close") {
            "cleanup".to_string()
        } else if name.contains("commit") {
            "commit".to_string()
        } else if name.contains("reveal") {
            "reveal".to_string()
        } else if name.contains("cancel") {
            "cancel".to_string()
        } else if name.contains("fetch") {
            "fetch".to_string()
        } else if name.contains("precompute") {
            "precompute".to_string()
        } else if name.contains("submit") {
            "submit".to_string()
        } else if name.contains("upload") {
            "upload".to_string()
        } else if name.contains("audit") {
            "audit".to_string()
        } else if name.contains("verify") {
            "verify".to_string()
        } else if name.contains("load") || name.contains("save") {
            "io".to_string()
        } else if name.contains("has") || name.contains("is") {
            "check".to_string()
        } else {
            "unknown".to_string()
        }
    }

    fn return_similarity(returns_a: &[String], returns_b: &[String]) -> f64 {
        if returns_a.is_empty() && returns_b.is_empty() {
            return 1.0;
        }
        if returns_a.is_empty() || returns_b.is_empty() {
            return 0.0;
        }

        let common = returns_a.iter().filter(|r| returns_b.contains(r)).count();
        common as f64 / returns_a.len().max(returns_b.len()) as f64
    }

    /// Categorize function names to detect if they serve similar purposes
    fn name_category(name: &str) -> &'static str {
        let lower = name.to_lowercase();

        // Getters
        if lower.starts_with("get") || lower.starts_with("fetch") || lower.starts_with("find") {
            "getter"
        }
        // Setters
        else if lower.starts_with("set")
            || lower.starts_with("update")
            || lower.starts_with("put")
        {
            "setter"
        }
        // Builders/Creators
        else if lower.starts_with("new")
            || lower.starts_with("create")
            || lower.starts_with("build")
        {
            "creator"
        }
        // Validators/Checkers
        else if lower.starts_with("is")
            || lower.starts_with("has")
            || lower.starts_with("validate")
            || lower.starts_with("check")
        {
            "validator"
        }
        // Actions
        else if lower.starts_with("save")
            || lower.starts_with("load")
            || lower.starts_with("delete")
            || lower.starts_with("remove")
        {
            "action"
        }
        // Converters
        else if lower.starts_with("to") || lower.starts_with("into") || lower.starts_with("from")
        {
            "converter"
        }
        // Processors
        else if lower.starts_with("process")
            || lower.starts_with("handle")
            || lower.starts_with("parse")
        {
            "processor"
        }
        // Default
        else {
            "other"
        }
    }
}
