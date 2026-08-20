// src/output/overview.rs

use crate::graph::call_graph::CallGraph;
use std::collections::HashMap;

pub struct LayerSummary {
    pub name: String,
    pub count: usize,
    pub description: &'static str,
    pub color: &'static str,
    pub top_functions: Vec<String>,
}

pub struct LayerEdge {
    pub from: String,
    pub to: String,
    pub weight: usize,
}

pub struct OverviewGraph;

impl OverviewGraph {
    pub fn generate(call_graph: &CallGraph, project_name: &str) -> String {
        let (layers, edges) = Self::collect_layer_data(call_graph);
        Self::build_html(&layers, &edges, project_name)
    }

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

    fn layer_description(layer: &str) -> &'static str {
        match layer {
            "handler" => "Receives requests and kicks off the work",
            "service" => "Business logic — the actual rules of the app",
            "repository" => "Reads and writes data storage",
            "middleware" => "Runs on every request before it reaches handlers",
            "config" => "Settings and environment setup",
            "worker" => "Background jobs and scheduled tasks",
            "blockchain" => "On-chain / smart contract interactions",
            "observability" => "Logging, metrics, tracing",
            "auth" => "Login, permissions, access control",
            "utility" => "Shared helper functions",
            "api" => "External-facing interface",
            "cli" => "Command-line entry points",
            "test" => "Test code",
            "core" => "Core application code",
            "root" => "Top-level project files",
            _ => "Uncategorized code",
        }
    }

    fn collect_layer_data(call_graph: &CallGraph) -> (Vec<LayerSummary>, Vec<LayerEdge>) {
        let mut layer_counts: HashMap<String, usize> = HashMap::new();
        let mut layer_top: HashMap<String, Vec<(String, f64)>> = HashMap::new();
        let mut layer_edges: HashMap<(String, String), usize> = HashMap::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            let layer = if func.layer.is_empty() {
                "unknown".to_string()
            } else {
                func.layer.clone()
            };

            *layer_counts.entry(layer.clone()).or_insert(0) += 1;
            layer_top
                .entry(layer.clone())
                .or_default()
                .push((func.name.clone(), func.importance_score));

            for callee in call_graph.get_callees(idx) {
                if let Some(&callee_idx) = call_graph.name_index.get(&callee.full_path) {
                    let callee_layer = call_graph[callee_idx].layer.clone();
                    let callee_layer = if callee_layer.is_empty() {
                        "unknown".to_string()
                    } else {
                        callee_layer
                    };
                    if callee_layer != layer {
                        *layer_edges
                            .entry((layer.clone(), callee_layer))
                            .or_insert(0) += 1;
                    }
                }
            }
        }

        let mut layers: Vec<LayerSummary> = layer_counts
            .into_iter()
            .map(|(name, count)| {
                let mut top = layer_top.remove(&name).unwrap_or_default();
                top.sort_by(|a, b| b.1.total_cmp(&a.1));
                let top_functions = top.into_iter().take(3).map(|(n, _)| n).collect();

                LayerSummary {
                    color: Self::layer_color(&name),
                    description: Self::layer_description(&name),
                    name,
                    count,
                    top_functions,
                }
            })
            .collect();
        layers.sort_by(|a, b| b.count.cmp(&a.count));

        let edges = layer_edges
            .into_iter()
            .map(|((from, to), weight)| LayerEdge { from, to, weight })
            .collect();

        (layers, edges)
    }

    fn build_html(layers: &[LayerSummary], edges: &[LayerEdge], project_name: &str) -> String {
        let layers_json = serde_json::to_string(
            &layers
                .iter()
                .map(|l| {
                    serde_json::json!({
                        "name": l.name,
                        "count": l.count,
                        "description": l.description,
                        "color": l.color,
                        "top_functions": l.top_functions,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();

        let edges_json = serde_json::to_string(
            &edges
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "from": e.from,
                        "to": e.to,
                        "weight": e.weight,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();

        format!(
            r###"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{project_name} - Architecture Overview</title>
<link rel="preconnect" href="https://fonts.googleapis.com">
<link href="https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@400;500;600;700;800&display=swap" rel="stylesheet">
<style>
    :root {{
        --bg: #090a0f;
        --panel: rgba(13, 16, 27, 0.85);
        --border: rgba(255, 255, 255, 0.08);
        --text: #f8fafc;
        --muted: #94a3b8;
    }}
    * {{ margin: 0; padding: 0; box-sizing: border-box; }}
    body {{
        font-family: 'Plus Jakarta Sans', sans-serif;
        background: var(--bg);
        color: var(--text);
        height: 100vh; width: 100vw;
        overflow: hidden;
    }}
    #title {{
        position: absolute; top: 24px; left: 24px; z-index: 10;
    }}
    #title h1 {{ font-size: 20px; font-weight: 800; }}
    #title p {{ font-size: 13px; color: var(--muted); margin-top: 4px; }}
    #viewport {{ width: 100vw; height: 100vh; }}
    .layer-box {{ cursor: pointer; }}
    .layer-box rect {{
        stroke-width: 2px;
        rx: 14px;
    }}
    .layer-box:hover rect {{ filter: brightness(1.25); }}
    .layer-label {{
        font-weight: 700; font-size: 14px; fill: #fff;
        text-anchor: middle; pointer-events: none;
    }}
    .layer-count {{
        font-size: 11px; fill: rgba(255,255,255,0.75);
        text-anchor: middle; pointer-events: none;
    }}
    .edge-path {{ fill: none; stroke: rgba(148,163,184,0.35); }}
    #tooltip {{
        position: absolute; z-index: 1000; padding: 10px 14px;
        border-radius: 10px; background: rgba(9,10,15,0.96);
        border: 1px solid var(--border); font-size: 12px;
        pointer-events: none; display: none; max-width: 260px;
        box-shadow: 0 10px 25px rgba(0,0,0,0.6);
    }}
    #tooltip .t-title {{ font-weight: 700; margin-bottom: 4px; }}
    #tooltip .t-desc {{ color: var(--muted); margin-bottom: 6px; }}
    #tooltip .t-fn {{ font-size: 11px; color: var(--muted); }}
</style>
</head>
<body>
    <div id="title">
        <h1>⚡ {project_name}</h1>
        <p>Architecture overview — hover a block to see what it does</p>
    </div>
    <div id="viewport"></div>
    <div id="tooltip"></div>

    <script src="https://d3js.org/d3.v7.min.js"></script>
    <script>
        const layers = {layers_json};
        const edges = {edges_json};

        const width = window.innerWidth;
        const height = window.innerHeight;
        const svg = d3.select("#viewport").append("svg")
            .attr("width", width).attr("height", height);
        const g = svg.append("g");

        svg.call(d3.zoom().scaleExtent([0.4, 2.5]).on("zoom", (e) => {{
            g.attr("transform", e.transform);
        }}));

        // Fixed circular layout — deterministic, no jitter, easy to read
        const cx = width / 2, cy = height / 2 + 20;
        const radius = Math.min(width, height) * 0.32;
        const maxCount = Math.max(...layers.map(l => l.count));

        layers.forEach((l, i) => {{
            const angle = (i / layers.length) * 2 * Math.PI - Math.PI / 2;
            l.x = cx + radius * Math.cos(angle);
            l.y = cy + radius * Math.sin(angle);
            l.w = 90 + (l.count / maxCount) * 90;
            l.h = 56;
        }});
        const byName = new Map(layers.map(l => [l.name, l]));

        const maxWeight = Math.max(1, ...edges.map(e => e.weight));

        g.selectAll(".edge-path")
            .data(edges.filter(e => byName.has(e.from) && byName.has(e.to)))
            .enter().append("path")
            .attr("class", "edge-path")
            .attr("stroke-width", e => 1 + (e.weight / maxWeight) * 5)
            .attr("d", e => {{
                const a = byName.get(e.from), b = byName.get(e.to);
                const mx = (a.x + b.x) / 2, my = (a.y + b.y) / 2 - 40;
                return `M${{a.x}},${{a.y}} Q${{mx}},${{my}} ${{b.x}},${{b.y}}`;
            }});

        const node = g.selectAll(".layer-box")
            .data(layers)
            .enter().append("g")
            .attr("class", "layer-box")
            .attr("transform", l => `translate(${{l.x - l.w/2}},${{l.y - l.h/2}})`);

        node.append("rect")
            .attr("width", l => l.w).attr("height", l => l.h)
            .attr("fill", l => l.color)
            .attr("stroke", l => d3.rgb(l.color).darker(0.8));

        node.append("text").attr("class", "layer-label")
            .attr("x", l => l.w/2).attr("y", l => l.h/2 - 4)
            .text(l => l.name);
        node.append("text").attr("class", "layer-count")
            .attr("x", l => l.w/2).attr("y", l => l.h/2 + 14)
            .text(l => `${{l.count}} functions`);

        const tooltip = document.getElementById("tooltip");
        node.on("mousemove", (event, l) => {{
            tooltip.style.display = "block";
            tooltip.style.left = (event.pageX + 16) + "px";
            tooltip.style.top = (event.pageY - 10) + "px";
            tooltip.innerHTML = `
                <div class="t-title">${{l.name}}</div>
                <div class="t-desc">${{l.description}}</div>
                ${{l.top_functions.map(f => `<div class="t-fn">• ${{f}}</div>`).join('')}}
            `;
        }}).on("mouseout", () => {{ tooltip.style.display = "none"; }});
    </script>
</body>
</html>"###
        )
    }
}
