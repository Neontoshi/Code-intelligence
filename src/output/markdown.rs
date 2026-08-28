use crate::graph::call_graph::CallGraph;
use crate::graph::traits::GraphMetrics;
use crate::parser::tree_sitter::ParsedFile;

pub struct MarkdownOutput;

impl MarkdownOutput {
    pub fn generate(call_graph: &CallGraph, files: &[ParsedFile]) -> String {
        let mut output = String::new();

        output.push_str("# Project Overview\n\n");

        // Statistics
        output.push_str("## 📊 Statistics\n\n");
        output.push_str(&format!(
            "- **Total Functions**: {}\n",
            call_graph.node_count()
        ));
        output.push_str(&format!("- **Total Files**: {}\n", files.len()));
        output.push_str(&format!(
            "- **Total Relationships**: {}\n",
            call_graph.edge_count()
        ));
        output.push_str("\n");

        // Languages
        let mut languages = std::collections::HashMap::new();
        for file in files {
            *languages.entry(file.language.clone()).or_insert(0) += 1;
        }

        if !languages.is_empty() {
            output.push_str("### Languages\n\n");
            for (lang, count) in languages {
                output.push_str(&format!("- **{}**: {} files\n", lang, count));
            }
            output.push('\n');
        }

        // Entry Points
        let entry_points: Vec<_> = call_graph
            .node_indices()
            .filter(|idx| {
                let func = &call_graph[*idx];
                func.is_public && call_graph.get_callers(*idx).is_empty()
            })
            .map(|idx| &call_graph[idx])
            .collect();

        if !entry_points.is_empty() {
            output.push_str("## 🚀 Entry Points\n\n");
            for func in entry_points {
                output.push_str(&format!("- **{}**", func.name));
                if let Some(doc) = &func.doc_comment {
                    output.push_str(&format!(" - {}", doc.lines().next().unwrap_or("")));
                }
                output.push('\n');
            }
            output.push('\n');
        }

        // Most Important Functions
        output.push_str("## 🔥 Most Important Functions\n\n");

        let mut functions: Vec<_> = call_graph
            .node_indices()
            .map(|idx| (idx, call_graph[idx].importance_score))
            .collect();
        functions.sort_by(|a, b| b.1.total_cmp(&a.1));

        for (idx, score) in functions.iter().take(10) {
            let func = &call_graph[*idx];
            let emoji = if *score > 0.8 {
                "🔥"
            } else if *score > 0.5 {
                "📌"
            } else {
                "📄"
            };
            output.push_str(&format!(
                "- {} **{}** (importance: {:.2})\n",
                emoji, func.name, score
            ));
            output.push_str(&format!("  - File: {}\n", func.file));
            output.push_str(&format!("  - Line: {}\n", func.line));
            let callees = call_graph.get_callees(*idx);
            if !callees.is_empty() {
                let callee_names: Vec<_> = callees.iter().map(|f| f.name.as_str()).collect();
                output.push_str(&format!("  - Calls: {}\n", callee_names.join(", ")));
            }
            output.push('\n');
        }

        output.push_str("## 📞 Call Graph Summary\n\n");
        output.push_str("```mermaid\n");
        output.push_str("graph TD\n");

        // Limit the Mermaid graph to the most important nodes to keep it readable.
        let selected = call_graph.top_important_nodes(60, 0.3);
        let mut added = std::collections::HashSet::new();
        for &idx in &selected {
            let func = &call_graph[idx];
            for callee in call_graph.get_callees(idx) {
                let edge_id = format!("{}->{}", func.full_path, callee.full_path);
                if !added.contains(&edge_id) {
                    let source_label = func.name.replace(' ', "_");
                    let target_label = callee.name.replace(' ', "_");
                    output.push_str(&format!(
                        "    {}[{}] --> {}[{}]\n",
                        source_label, func.name, target_label, callee.name
                    ));
                    added.insert(edge_id);
                }
            }
        }

        output.push_str("## 🏗️ Architecture Layers\n\n");
        let layers = crate::output::json::JsonOutput::detect_layers(files);
        for layer in layers {
            if let (Some(name), Some(files)) = (layer.get("name"), layer.get("files")) {
                if let (Some(name_str), Some(files_arr)) = (name.as_str(), files.as_array()) {
                    output.push_str(&format!("### {}\n\n", name_str));
                    for file in files_arr {
                        if let Some(file_str) = file.as_str() {
                            output.push_str(&format!("- {}\n", file_str));
                        }
                    }
                    output.push('\n');
                }
            }
        }

        output
    }

    pub fn generate_readme(call_graph: &CallGraph, files: &[ParsedFile]) -> String {
        let mut output = String::new();

        output.push_str("# Project Documentation\n\n");
        output.push_str("## Overview\n\n");
        output.push_str(&format!(
            "This project contains **{}** functions across **{}** files.\n\n",
            call_graph.node_count(),
            files.len()
        ));

        output.push_str("## Key Functions\n\n");

        let mut functions: Vec<_> = call_graph
            .node_indices()
            .map(|idx| (idx, call_graph[idx].importance_score))
            .collect();
        functions.sort_by(|a, b| b.1.total_cmp(&a.1));

        for (idx, score) in functions.iter().take(5) {
            let func = &call_graph[*idx];
            output.push_str(&format!("### {}\n\n", func.name));
            if let Some(doc) = &func.doc_comment {
                output.push_str(&format!("{}\n\n", doc));
            }
            output.push_str(&format!("- **File**: {}\n", func.file));
            output.push_str(&format!("- **Line**: {}\n", func.line));
            output.push_str(&format!("- **Importance**: {:.2}\n", score));
            output.push('\n');
        }

        output
    }
}
