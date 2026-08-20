// src/optimize/dedup/core.rs

use crate::graph::call_graph::FunctionNode;
use crate::parser::tree_sitter::ParsedFile;
use sha2::Digest;
use std::collections::HashMap;

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
        Self { by_path }
    }

    pub fn build_from_graph(
        call_graph: &crate::graph::call_graph::CallGraph,
        files: &[crate::parser::tree_sitter::ParsedFile],
    ) -> Self {
        let functions: Vec<crate::graph::call_graph::FunctionNode> = call_graph
            .node_indices()
            .map(|idx| call_graph[idx].clone())
            .collect();
        Self::build(&functions, files)
    }

    pub fn get(&self, full_path: &str) -> Option<&str> {
        self.by_path.get(full_path).map(|s| s.as_str())
    }

    fn extract_body(
        func: &FunctionNode,
        file_by_path: &HashMap<&str, &ParsedFile>,
    ) -> Option<String> {
        let file = *file_by_path.get(func.file.as_str())?;

        let info = file.functions.iter().find(|fi| {
            if fi.name != func.name {
                return false;
            }
            if fi.line == func.line {
                return true;
            }
            let fi_full_path = match &fi.container {
                Some(c) => format!("{}::{}::{}", file.path, c, fi.name),
                None => format!("{}::{}", file.path, fi.name),
            };
            fi_full_path == func.full_path
        })?;

        let (start, end) = info.body_range;
        if start >= end || end > file.source.len() {
            return None;
        }

        file.source.get(start..end).map(|s| s.to_string())
    }
}

pub fn compute_signature_hash(func: &FunctionNode) -> String {
    let mut hasher = sha2::Sha256::new();
    let sig = format!(
        "sig:{}|{}|{}|{}|{}",
        func.name,
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

    if source.trim().len() < 20 || source.lines().count() < 4 {
        return String::new();
    }

    let mut hasher = sha2::Sha256::new();
    let mut var_map = HashMap::new();
    let mut var_counter = 0;

    let mut normalized = String::new();
    for line in source.lines() {
        let mut processed_line = line.to_string();

        let id_regex = Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\b").unwrap();
        let mut replacements = Vec::new();

        for cap in id_regex.captures_iter(line) {
            let word = cap.get(1).unwrap().as_str();

            let skip_words = [
                "if", "else", "for", "while", "match", "fn", "pub", "async", "await", "return",
                "let", "mut", "struct", "enum", "trait", "impl", "use", "mod", "true", "false",
                "null", "None", "Some", "Ok", "Err", "Result", "Option", "Vec", "String", "Box",
                "Arc", "Rc", "self", "Self", "super", "crate",
            ];
            if skip_words.contains(&word) {
                continue;
            }

            let start = cap.get(0).unwrap().start();
            let end = cap.get(0).unwrap().end();
            let prev_char = line[..start].chars().last().unwrap_or(' ');
            if prev_char == '.' || prev_char == ':' || prev_char == '<' {
                continue;
            }
            // Preserve call targets — two functions calling different
            // helpers are not the same function, even with an identical
            // surrounding skeleton (e.g. `compute_a(x)` vs `compute_b(x)`).
            let next_char = line[end..].chars().next().unwrap_or(' ');
            if next_char == '(' {
                continue;
            }

            let var_id = var_map.entry(word.to_string()).or_insert_with(|| {
                var_counter += 1;
                var_counter
            });

            replacements.push((start, end, format!("VAR{}", var_id)));
        }

        for (start, end, replacement) in replacements.iter().rev() {
            processed_line.replace_range(*start..*end, replacement);
        }

        normalized.push_str(&processed_line);
        normalized.push('\n');
    }

    let normalized_body: String = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
    hasher.update(b"ast:");
    hasher.update(normalized_body.as_bytes());

    format!("{:x}", hasher.finalize())
}

pub fn compute_exact_hash(func: &FunctionNode, source: Option<&str>) -> String {
    let mut hasher = sha2::Sha256::new();

    match source {
        Some(body) if body.trim().len() >= 25 && body.lines().count() >= 4 => {
            let normalized: String = body.split_whitespace().collect::<Vec<_>>().join(" ");
            hasher.update(b"body:");
            hasher.update(normalized.as_bytes());
            format!("{:x}", hasher.finalize())
        }
        _ => {
            // Never allow fallback metadata to produce collisions across different functions
            let unique = format!("unique:{}:{}", func.file, func.full_path);
            hasher.update(unique.as_bytes());
            format!("{:x}", hasher.finalize())
        }
    }
}
