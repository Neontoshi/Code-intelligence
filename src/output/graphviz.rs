use crate::graph::call_graph::CallGraph;
use petgraph::graph::NodeIndex;
use std::collections::{HashMap, HashSet};

/// Controls how much of the call graph actually gets rendered.
///
/// The old generator only filtered by `importance_score > 0.3`, which on a
/// codebase this size still let ~1,500 nodes through. `max_nodes` puts a
/// hard ceiling on output size no matter how big the codebase gets.
pub struct GraphVizConfig {
    /// Hard cap on nodes rendered — the single biggest lever for file size.
    pub max_nodes: usize,
    /// Floor below which a node isn't even considered a candidate.
    pub min_importance: f64,
    /// Truncate long function names so boxes stay a readable size.
    pub max_label_len: usize,
    /// Collapse duplicate A->B edges (e.g. multiple call sites) into one.
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
    /// Rank all candidate nodes by importance and keep only the top
    /// `max_nodes`. This is what actually bounds file size — a raw
    /// threshold filter alone doesn't, since a big codebase can still have
    /// thousands of nodes above any fixed cutoff.
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

            // If adding this word exceeds the line limit, start a new line
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
            "#e74c3c" // critical
        } else if importance > 0.5 {
            "#f39c12" // important
        } else {
            "#3498db" // regular
        }
    }

    fn resolve(call_graph: &CallGraph, full_path: &str) -> Option<NodeIndex> {
        call_graph
            .node_indices()
            .find(|&i| call_graph[i].full_path == full_path)
    }

    /// Default entry point: bounded, importance-colored graph. This is the
    /// one to reach for by default — it's the direct replacement for the
    /// old `generate()`, just capped in size.
    pub fn generate(call_graph: &CallGraph) -> String {
        Self::generate_with_config(call_graph, &GraphVizConfig::default())
    }

    pub fn generate_with_config(call_graph: &CallGraph, config: &GraphVizConfig) -> String {
        let selected = Self::select_nodes(call_graph, config);
        let selected_set: HashSet<NodeIndex> = selected.iter().copied().collect();

        let mut dot = String::new();
        dot.push_str("digraph CallGraph {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  concentrate=true;\n"); // merges visually-parallel edge paths
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
                    // A callee outside the top-N no longer gets dropped —
                    // it just doesn't get "important" styling. Otherwise a
                    // selected node's real calls to low-importance functions
                    // were invisible, making it look like it called nothing.
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

    /// Focused view: only the neighborhood around one function, out to
    /// `depth` call-hops. Use this instead of the full graph when you just
    /// want to understand one part of the system — output stays small
    /// regardless of overall codebase size.
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
        dot.push_str("  size=\"14,10\";\n"); // uniform scale-to-fit only — ratio=compress added no benefit in testing and risks uneven scaling across disconnected components
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

    /// Bird's-eye view: one node per architectural layer (handler, service,
    /// repository, etc.) instead of per-function, with edge weight = number
    /// of cross-layer calls. Zero per-function noise — use this to sanity
    /// check overall architecture shape before drilling into a layer.
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
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\\n({} functions)\"];\n",
                layer, layer, count
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
}
