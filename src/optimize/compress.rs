// src/optimize/compress.rs

use crate::graph::call_graph::{CallGraph, FunctionNode};
use crate::graph::traits::GraphMetrics;
use crate::optimize::symbols::SymbolTable;
use crate::parser::tree_sitter::ParsedFile;

pub struct SemanticCompressor {
    max_functions: usize,
    _include_bodies: bool,
    _use_symbols: bool,
    symbol_table: SymbolTable,
}

impl SemanticCompressor {
    pub fn new() -> Self {
        Self {
            max_functions: 100,
            _include_bodies: true,
            _use_symbols: true,
            symbol_table: SymbolTable::universal(),
        }
    }

    pub fn compress(&self, call_graph: &CallGraph, files: &[ParsedFile]) -> String {
        let mut output = String::new();

        output.push_str("## Project Architecture\n\n");
        output.push_str(&format!(
            "Functions: {} | Files: {} | Relationships: {}\n\n",
            call_graph.node_count(),
            files.len(),
            call_graph.edge_count()
        ));

        output.push_str("### 🚀 Public API\n\n");
        let entry_points: Vec<_> = call_graph
            .node_indices()
            .filter(|idx| {
                let func = &call_graph[*idx];
                func.is_public && call_graph.get_callers(*idx).is_empty()
            })
            .map(|idx| &call_graph[idx])
            .collect();

        for func in &entry_points {
            output.push_str(&self.format_function(func, call_graph));
        }

        output.push_str("\n### 🔥 Core Functions\n\n");
        let mut sorted: Vec<_> = call_graph
            .node_indices()
            .map(|idx| &call_graph[idx])
            .collect();
        sorted.sort_by(|a, b| b.importance_score.partial_cmp(&a.importance_score).unwrap());

        for func in sorted.iter().take(self.max_functions) {
            if !entry_points.contains(func) {
                output.push_str(&self.format_function(func, call_graph));
            }
        }

        output.push_str("\n### 📞 Call Graph\n\n```\n");
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            let callees = call_graph.get_callees(idx);
            if !callees.is_empty() {
                output.push_str(&format!(
                    "{} → {}\n",
                    func.name,
                    callees
                        .iter()
                        .map(|f| f.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }
        output.push_str("```\n");

        output
    }

    pub fn compress_enhanced(&self, call_graph: &CallGraph, files: &[ParsedFile]) -> String {
        let mut symbols: Vec<(String, String)> = self
            .symbol_table
            .replacements
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        for file in files {
            let short_name = file
                .path
                .split('/')
                .last()
                .unwrap_or(&file.path)
                .to_string();
            let symbol = format!("📄{}", short_name.chars().take(3).collect::<String>());
            symbols.push((short_name, symbol));
        }

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            let short_name = func.name.chars().take(4).collect::<String>();
            let symbol = format!("ƒ{}", short_name);
            if !symbols.iter().any(|(s, _)| *s == func.name) {
                symbols.push((func.name.clone(), symbol));
            }
        }

        symbols.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        let mut compressed = String::new();
        for file in files {
            let mut content = file.source.clone();
            for (pattern, symbol) in &symbols {
                content = content.replace(pattern, &symbol);
            }
            compressed.push_str(&content);
            compressed.push('\n');
        }

        compressed
    }

    pub fn compress_source(&self, files: &[ParsedFile]) -> String {
        let mut output = String::new();

        output.push_str("\n---\n\n## Compressed Source\n\n");

        for file in files {
            if file.functions.is_empty() && file.types.is_empty() {
                continue;
            }

            let filename = file.path.split('/').last().unwrap_or(&file.path);
            output.push_str(&format!("### `{}`\n\n", filename));

            for t in &file.types {
                output.push_str(&format!(
                    "∁ {} ({})\n",
                    t.name,
                    format!("{:?}", t.kind).to_lowercase()
                ));
            }

            for func in &file.functions {
                output.push_str(&format!("ƒ {}(", func.name));
                output.push_str(
                    &func
                        .params
                        .iter()
                        .map(|p| {
                            let mut s = p.name.clone();
                            if let Some(t) = &p.type_hint {
                                s.push_str(&format!(":{}", t));
                            }
                            s
                        })
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                output.push_str(")");

                if let Some(ret) = &func.return_type {
                    output.push_str(&format!("→{}", ret));
                }

                output.push_str(" {");

                if !func.calls.is_empty() {
                    output.push_str(&format!(
                        " 📞{}",
                        func.calls
                            .iter()
                            .map(|c| c.to_string())
                            .collect::<Vec<_>>()
                            .join("→")
                    ));
                }

                output.push_str(" }\n");
            }

            output.push('\n');
        }

        output
    }

    pub fn full_report(&self, call_graph: &CallGraph, files: &[ParsedFile]) -> String {
        let mut output = self.compress(call_graph, files);
        output.push_str(&self.compress_source(files));
        output
    }

    fn format_function(&self, func: &FunctionNode, call_graph: &CallGraph) -> String {
        let mut s = String::new();

        s.push_str(&format!("ƒ {}", func.name));
        if func.importance_score > 0.7 {
            s.push_str(" 🔥");
        } else if func.importance_score > 0.3 {
            s.push_str(" 📌");
        }
        if func.is_async {
            s.push_str(" ∂");
        }
        s.push('\n');

        if let Some(doc) = &func.doc_comment {
            s.push_str(&format!("  {}\n", doc.lines().next().unwrap_or("")));
        }

        s.push_str(&format!(
            "  📍 {}:{}\n",
            func.file.split('/').last().unwrap_or(&func.file),
            func.line
        ));

        if !func.params.is_empty() {
            s.push_str(&format!("  📥 {}\n", func.params.join(", ")));
        }

        if !func.returns.is_empty() {
            s.push_str(&format!("  📤 {}\n", func.returns.join(", ")));
        }

        let idx = call_graph
            .node_indices()
            .find(|i| call_graph[*i].full_path == func.full_path);
        if let Some(idx) = idx {
            let callees = call_graph.get_callees(idx);
            if !callees.is_empty() {
                s.push_str(&format!(
                    "  📞 {}\n",
                    callees
                        .iter()
                        .map(|f| f.name.as_str())
                        .collect::<Vec<_>>()
                        .join(" → ")
                ));
            }
        }

        s.push('\n');
        s
    }
}
