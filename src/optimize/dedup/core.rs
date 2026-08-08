// src/optimize/dedup/core.rs

use crate::graph::call_graph::FunctionNode;
use crate::parser::tree_sitter::ParsedFile;
use sha2::Digest;
use std::collections::HashMap;

/// Resolves function bodies to source text when available, so hashing and
/// ML feature extraction can look at actual code instead of only metadata.
pub struct SourceIndex {
    by_path: HashMap<String, String>,
}

impl SourceIndex {
    pub fn build(functions: &[FunctionNode], files: &[ParsedFile]) -> Self {
        let file_by_path: HashMap<&str, &ParsedFile> =
            files.iter().map(|f| (f.path.as_str(), f)).collect();

        let mut by_path = HashMap::new();
        for func in functions {
            if let Some(body) = Self::extract_body(func, &file_by_path) {
                by_path.insert(func.full_path.clone(), body);
            }
        }
        #[cfg(debug_assertions)]
        {
            let total = functions.len();
            let found = by_path.len();
            if found < total {
                eprintln!(
                    "⚠️ SourceIndex: Found bodies for {}/{} functions ({} missing)",
                    found,
                    total,
                    total - found
                );
            } else {
                eprintln!("✅ SourceIndex: Found bodies for all {} functions", total);
            }
        }

        Self { by_path }
    }

    pub fn get(&self, full_path: &str) -> Option<&str> {
        self.by_path.get(full_path).map(|s| s.as_str())
    }

    /// Extracts the source code of a function body.
    ///
    /// Matches by comparing the `full_path` (which includes container info)
    /// against the constructed path from the parsed file.
    fn extract_body(
        func: &FunctionNode,
        file_by_path: &HashMap<&str, &ParsedFile>,
    ) -> Option<String> {
        let file = *file_by_path.get(func.file.as_str())?;
        // 2. Find the function info in the parsed file.
        //    We construct the full path from the parsed info and compare it
        //    to the func's full_path.
        let info = file.functions.iter().find(|fi| {
            let fi_full_path = match &fi.container {
                Some(c) => format!("{}::{}::{}", file.path, c, fi.name),
                None => format!("{}::{}", file.path, fi.name),
            };
            fi_full_path == func.full_path
        })?;

        // 3. Slice the source code using the body_range.
        let (start, end) = info.body_range;

        // Ensure bounds are valid
        if start >= end || end > file.source.len() {
            return None;
        }

        // 4. Return the sliced string
        file.source.get(start..end).map(|s| s.to_string())
    }
}

/// Used as a fast first-pass filter before more expensive comparisons.
pub fn compute_signature_hash(func: &FunctionNode) -> String {
    let mut hasher = sha2::Sha256::new();
    let sig = format!(
        "sig:{}|{}|{}|{}",
        func.params.len(),
        func.returns.len(),
        func.is_public,
        func.is_async
    );
    hasher.update(sig.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn compute_ast_hash(_func: &FunctionNode, source: &str) -> String {
    use regex::Regex;
    use std::collections::HashMap;

    let mut hasher = sha2::Sha256::new();
    let mut var_map = HashMap::new();
    let mut var_counter = 0;

    // Split into lines and process
    let mut normalized = String::new();
    for line in source.lines() {
        let mut processed_line = line.to_string();

        // Replace identifiers with VAR1, VAR2, etc.
        let id_regex = Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\b").unwrap();
        let mut replacements = Vec::new();

        for cap in id_regex.captures_iter(line) {
            let word = cap.get(1).unwrap().as_str();

            // Skip keywords and common literals
            let skip_words = [
                "if", "else", "for", "while", "match", "fn", "pub", "async", "await", "return",
                "let", "mut", "struct", "enum", "trait", "impl", "use", "mod", "true", "false",
                "null", "None", "Some", "Ok", "Err", "Result", "Option", "Vec", "String", "Box",
                "Arc", "Rc", "self", "Self", "super", "crate",
            ];
            if skip_words.contains(&word) {
                continue;
            }

            // Skip if it's a type annotation or method call
            let start = cap.get(0).unwrap().start();
            let prev_char = line[..start].chars().last().unwrap_or(' ');
            if prev_char == '.' || prev_char == ':' || prev_char == '<' {
                continue;
            }

            // Get or assign a variable number
            let var_id = var_map.entry(word.to_string()).or_insert_with(|| {
                var_counter += 1;
                var_counter
            });

            let end = cap.get(0).unwrap().end();
            replacements.push((start, end, format!("VAR{}", var_id)));
        }

        // Apply replacements (in reverse order to keep indices valid)
        for (start, end, replacement) in replacements.iter().rev() {
            processed_line.replace_range(*start..*end, replacement);
        }

        normalized.push_str(&processed_line);
        normalized.push('\n');
    }

    // Normalize whitespace and hash
    let normalized_body: String = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
    hasher.update(b"ast:");
    hasher.update(normalized_body.as_bytes());

    format!("{:x}", hasher.finalize())
}

/// Exact-match hash. Uses real body text when `SourceIndex` can resolve it;
/// falls back to a metadata signature otherwise.
pub fn compute_exact_hash(func: &FunctionNode, source: Option<&str>) -> String {
    let mut hasher = sha2::Sha256::new();

    match source {
        Some(body) => {
            // Normalize whitespace so reformatted-but-identical code still matches.
            let normalized: String = body.split_whitespace().collect::<Vec<_>>().join(" ");
            hasher.update(b"body:");
            hasher.update(normalized.as_bytes());
        }
        None => {
            let sig = format!(
                "meta:{}|{}|{}|{}|{}",
                func.params.len(),
                func.returns.len(),
                func.is_public,
                func.is_async,
                func.complexity
            );
            hasher.update(sig.as_bytes());
        }
    }

    format!("{:x}", hasher.finalize())
}
