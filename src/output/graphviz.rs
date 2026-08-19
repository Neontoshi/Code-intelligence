use crate::graph::call_graph::CallGraph;
use petgraph::graph::NodeIndex;
use std::collections::{HashMap, HashSet};

/// Controls how much of the call graph actually gets rendered.
pub struct GraphVizConfig {
    pub max_nodes: usize,
    pub min_importance: f64,
    pub max_label_len: usize,
    pub concentrate_edges: bool,
}

impl Default for GraphVizConfig {
    fn default() -> Self {
        Self {
            max_nodes: 60,
            min_importance: 0.3,
            max_label_len: 28,
            concentrate_edges: true,
        }
    }
}

pub struct GraphVizOutput;

impl GraphVizOutput {
    fn select_nodes(call_graph: &CallGraph, config: &GraphVizConfig) -> Vec<NodeIndex> {
        call_graph.top_important_nodes(config.max_nodes, config.min_importance)
    }

    fn wrap_label(name: &str, max_line_len: usize) -> String {
        if name.chars().count() <= max_line_len {
            return name.to_string();
        }

        let mut result = String::new();
        let mut current_line_len = 0;

        for word in name.split_inclusive(&['_', '.', ':', '/', '-']) {
            let word_len = word.chars().count();
            if current_line_len + word_len > max_line_len && current_line_len > 0 {
                result.push('\n');
                current_line_len = 0;
            }
            result.push_str(word);
            current_line_len += word_len;
        }

        result
    }

    fn node_color(importance: f64) -> &'static str {
        if importance > 0.8 {
            "#e74c3c"
        } else if importance > 0.5 {
            "#f39c12"
        } else {
            "#3498db"
        }
    }

    /// ⭐ NEW: Color by layer
    fn layer_color(layer: &str) -> &'static str {
        match layer {
            "handler" => "#3498db",
            "service" => "#2ecc71",
            "repository" => "#f39c12",
            "middleware" => "#9b59b6",
            "config" => "#1abc9c",
            "worker" => "#e67e22",
            "blockchain" => "#e74c3c",
            "observability" => "#1a1a2e",
            "auth" => "#c0392b",
            "utility" => "#7f8c8d",
            "api" => "#2980b9",
            "cli" => "#2c3e50",
            "test" => "#8e44ad",
            "core" => "#34495e",
            _ => "#95a5a6",
        }
    }

    /// ⭐ NEW: Format node label with metrics
    fn format_node_label(
        func: &crate::graph::call_graph::FunctionNode,
        config: &GraphVizConfig,
    ) -> String {
        let mut label = Self::wrap_label(&func.name, config.max_label_len);
        label = format!(
            "{}\\n[fan_in: {}, complexity: {:.1}]",
            label, func.fan_in, func.complexity
        );
        label
    }

    fn resolve(call_graph: &CallGraph, full_path: &str) -> Option<NodeIndex> {
        call_graph.name_index.get(full_path).copied()
    }

    /// Default entry point: bounded, importance-colored graph
    pub fn generate(call_graph: &CallGraph) -> String {
        Self::generate_with_config(call_graph, &GraphVizConfig::default())
    }

    pub fn generate_with_config(call_graph: &CallGraph, config: &GraphVizConfig) -> String {
        let selected = Self::select_nodes(call_graph, config);
        let selected_set: HashSet<NodeIndex> = selected.iter().copied().collect();

        let mut dot = String::new();
        dot.push_str("digraph CallGraph {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  concentrate=true;\n");
        dot.push_str("  ranksep=0.6; nodesep=0.4;\n");
        dot.push_str("  size=\"14,10\";\n");
        dot.push_str("  node [shape=box, style=rounded, fontname=\"Courier New\", fontsize=11, margin=\"0.25,0.12\"];\n");
        dot.push_str("  edge [fontname=\"monospace\", fontsize=9, color=\"#7f8c8d\"];\n\n");

        for &idx in &selected {
            let func = &call_graph[idx];
            let label = Self::wrap_label(&func.name, config.max_label_len);
            let color = Self::node_color(func.importance_score);
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\", fillcolor=\"{}\", style=\"filled,rounded\"];\n",
                idx.index(),
                label,
                color
            ));
        }

        dot.push('\n');

        let mut edges_seen: HashSet<(usize, usize)> = HashSet::new();
        let mut extra_nodes_declared: HashSet<usize> = HashSet::new();
        for &idx in &selected {
            for callee in call_graph.get_callees(idx) {
                if let Some(callee_idx) = Self::resolve(call_graph, &callee.full_path) {
                    if !selected_set.contains(&callee_idx)
                        && extra_nodes_declared.insert(callee_idx.index())
                    {
                        let label = Self::wrap_label(&callee.name, config.max_label_len);
                        dot.push_str(&format!(
                            "  \"{}\" [label=\"{}\", fillcolor=\"#dcdde1\", style=\"filled,rounded\"];\n",
                            callee_idx.index(),
                            label
                        ));
                    }

                    let key = (idx.index(), callee_idx.index());
                    if config.concentrate_edges && !edges_seen.insert(key) {
                        continue;
                    }
                    dot.push_str(&format!(
                        "  \"{}\" -> \"{}\";\n",
                        idx.index(),
                        callee_idx.index()
                    ));
                }
            }
        }

        dot.push_str("}\n");
        dot
    }

    /// Focused view: only the neighborhood around one function
    pub fn generate_focused(call_graph: &CallGraph, entry_full_path: &str, depth: usize) -> String {
        let start_idx = match Self::resolve(call_graph, entry_full_path) {
            Some(idx) => idx,
            None => {
                return format!(
                    "digraph CallGraph {{ label=\"not found: {}\"; }}\n",
                    entry_full_path
                )
            }
        };

        let mut visited: HashSet<NodeIndex> = HashSet::new();
        visited.insert(start_idx);
        let mut frontier = vec![start_idx];

        for _ in 0..depth {
            let mut next_frontier = Vec::new();
            for &idx in &frontier {
                for callee in call_graph.get_callees(idx) {
                    if let Some(callee_idx) = Self::resolve(call_graph, &callee.full_path) {
                        if visited.insert(callee_idx) {
                            next_frontier.push(callee_idx);
                        }
                    }
                }
            }
            frontier = next_frontier;
        }

        let mut dot = String::new();
        dot.push_str("digraph CallGraph {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  size=\"14,10\";\n");
        dot.push_str("  node [shape=box, style=rounded, fontname=\"Courier New\", fontsize=11, margin=\"0.25,0.12\"];\n");
        dot.push_str("  edge [fontname=\"monospace\", fontsize=9, color=\"#7f8c8d\"];\n\n");

        for &idx in &visited {
            let func = &call_graph[idx];
            let label = Self::wrap_label(&func.name, 32);
            let color = if idx == start_idx {
                "#e74c3c"
            } else {
                Self::node_color(func.importance_score)
            };
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\", fillcolor=\"{}\", style=\"filled,rounded\"];\n",
                idx.index(),
                label,
                color
            ));
        }

        dot.push('\n');
        for &idx in &visited {
            for callee in call_graph.get_callees(idx) {
                if let Some(callee_idx) = Self::resolve(call_graph, &callee.full_path) {
                    if visited.contains(&callee_idx) {
                        dot.push_str(&format!(
                            "  \"{}\" -> \"{}\";\n",
                            idx.index(),
                            callee_idx.index()
                        ));
                    }
                }
            }
        }

        dot.push_str("}\n");
        dot
    }

    /// Bird's-eye view: one node per architectural layer
    pub fn generate_layer_summary(call_graph: &CallGraph) -> String {
        let mut layer_counts: HashMap<String, usize> = HashMap::new();
        let mut layer_edges: HashMap<(String, String), usize> = HashMap::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            *layer_counts.entry(func.layer.clone()).or_insert(0) += 1;

            for callee in call_graph.get_callees(idx) {
                if let Some(callee_idx) = Self::resolve(call_graph, &callee.full_path) {
                    let callee_layer = call_graph[callee_idx].layer.clone();
                    if callee_layer != func.layer {
                        *layer_edges
                            .entry((func.layer.clone(), callee_layer))
                            .or_insert(0) += 1;
                    }
                }
            }
        }

        let mut dot = String::new();
        dot.push_str("digraph CallGraph {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  size=\"14,10\";\n");
        dot.push_str("  node [shape=box, style=rounded, fontname=\"Courier New\", fontsize=11, margin=\"0.25,0.12\"];\n");
        dot.push_str("  edge [fontname=\"monospace\", fontsize=10, color=\"#636e72\"];\n\n");

        for (layer, count) in &layer_counts {
            let color = Self::layer_color(layer);
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\\n({} functions)\", fillcolor=\"{}\", style=\"filled,rounded\"];\n",
                layer, layer, count, color
            ));
        }

        dot.push('\n');
        for ((from, to), weight) in &layer_edges {
            let penwidth = (1.0 + (*weight as f64).log2()).max(1.0);
            dot.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"{}\", penwidth={:.1}];\n",
                from, to, weight, penwidth
            ));
        }

        dot.push_str("}\n");
        dot
    }

    /// ⭐ NEW: Two-phase graph - important functions + layer summaries
    pub fn generate_two_phase(call_graph: &CallGraph, config: &GraphVizConfig) -> String {
        let mut dot = String::new();
        dot.push_str("digraph CallGraph {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  concentrate=true;\n");
        dot.push_str("  ranksep=0.6; nodesep=0.4;\n");
        dot.push_str("  size=\"14,10\";\n");
        dot.push_str("  node [shape=box, style=rounded, fontname=\"Courier New\", fontsize=11, margin=\"0.25,0.12\"];\n");
        dot.push_str("  edge [fontname=\"monospace\", fontsize=9, color=\"#7f8c8d\"];\n\n");

        // Track which nodes are important
        let mut important_nodes = HashSet::new();
        let mut layer_groups: HashMap<String, Vec<NodeIndex>> = HashMap::new();

        // First pass: categorize nodes
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            if func.importance_score > 0.5 {
                important_nodes.insert(idx);
            } else {
                let layer = if func.layer.is_empty() {
                    "unknown".to_string()
                } else {
                    func.layer.clone()
                };
                layer_groups.entry(layer).or_default().push(idx);
            }
        }

        // Second pass: Render important nodes
        for &idx in &important_nodes {
            let func = &call_graph[idx];
            let label = Self::format_node_label(func, config);
            let color = Self::node_color(func.importance_score);
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\", fillcolor=\"{}\", style=\"filled,rounded\"];\n",
                idx.index(),
                label,
                color
            ));
        }

        // Render aggregated layer nodes
        for (layer, nodes) in &layer_groups {
            if !nodes.is_empty() {
                let color = Self::layer_color(layer);
                dot.push_str(&format!(
                    "  \"layer_{}\" [label=\"{}\\n({} functions)\", shape=box, style=dashed, fillcolor=\"{}\", color=\"#7f8c8d\"];\n",
                    layer, layer, nodes.len(), color
                ));
            }
        }

        dot.push('\n');

        // Edges between important nodes
        let mut edges_seen: HashSet<(usize, usize)> = HashSet::new();
        for &idx in &important_nodes {
            for callee in call_graph.get_callees(idx) {
                if let Some(callee_idx) = Self::resolve(call_graph, &callee.full_path) {
                    if important_nodes.contains(&callee_idx) {
                        let key = (idx.index(), callee_idx.index());
                        if !edges_seen.contains(&key) {
                            edges_seen.insert(key);
                            dot.push_str(&format!(
                                "  \"{}\" -> \"{}\";\n",
                                idx.index(),
                                callee_idx.index()
                            ));
                        }
                    }
                }
            }
        }

        // Edges from important nodes to layer aggregations
        for &idx in &important_nodes {
            for callee in call_graph.get_callees(idx) {
                if let Some(callee_idx) = Self::resolve(call_graph, &callee.full_path) {
                    if !important_nodes.contains(&callee_idx) {
                        let callee_func = &call_graph[callee_idx];
                        let layer = if callee_func.layer.is_empty() {
                            "unknown".to_string()
                        } else {
                            callee_func.layer.clone()
                        };
                        let layer_key = format!("layer_{}", layer);
                        dot.push_str(&format!(
                            "  \"{}\" -> \"{}\" [style=dashed, color=\"#95a5a6\"];\n",
                            idx.index(),
                            layer_key
                        ));
                    }
                }
            }
        }

        // Edges from layer aggregations to important nodes
        for (layer, nodes) in &layer_groups {
            for node_idx in nodes {
                for callee in call_graph.get_callees(*node_idx) {
                    if let Some(callee_idx) = Self::resolve(call_graph, &callee.full_path) {
                        if important_nodes.contains(&callee_idx) {
                            let layer_key = format!("layer_{}", layer);
                            dot.push_str(&format!(
                                "  \"{}\" -> \"{}\" [style=dotted, color=\"#95a5a6\"];\n",
                                layer_key,
                                callee_idx.index()
                            ));
                            break;
                        }
                    }
                }
            }
        }

        // Legend
        dot.push_str(&Self::generate_legend());

        dot.push_str("}\n");
        dot
    }

    /// Generate legend for two-phase graph
    fn generate_legend() -> String {
        let mut legend = String::new();
        legend.push_str("  subgraph cluster_legend {\n");
        legend.push_str("    label=\"Legend\";\n");
        legend.push_str("    style=filled;\n    fillcolor=\"#ecf0f1\";\n");
        legend.push_str("    node [shape=box, style=rounded, fontsize=10];\n");
        legend.push_str("    legend_important [label=\"Important function\", fillcolor=\"#f39c12\", style=\"filled\"];\n");
        legend.push_str("    legend_normal [label=\"Normal function\", fillcolor=\"#3498db\", style=\"filled\"];\n");
        legend.push_str("    legend_layer [label=\"Layer group\", fillcolor=\"#ecf0f1\", style=\"dashed\", color=\"#7f8c8d\"];\n");
        legend.push_str("    legend_solid [label=\"Call edge\", color=\"#7f8c8d\"];\n");
        legend.push_str(
            "    legend_dashed [label=\"To layer\", style=\"dashed\", color=\"#95a5a6\"];\n",
        );
        legend.push_str(
            "    legend_dotted [label=\"From layer\", style=\"dotted\", color=\"#95a5a6\"];\n",
        );
        legend.push_str("  }\n");
        legend
    }
}
