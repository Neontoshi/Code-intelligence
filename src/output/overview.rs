// src/output/overview.rs

use crate::graph::call_graph::CallGraph;
use std::collections::HashMap;

pub struct LayerSummary {
    pub name: String,
    pub count: usize,
    pub description: String,
    pub color: String,
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
        let function_nodes = Self::collect_function_nodes(call_graph);
        let function_edges = Self::collect_function_edges(call_graph);
        Self::build_html(
            &layers,
            &edges,
            &function_nodes,
            &function_edges,
            project_name,
        )
    }

    fn layer_color(layer: &str) -> String {
        match layer {
            "handler" => "#38bdf8".to_string(),
            "service" => "#10b981".to_string(),
            "repository" => "#f59e0b".to_string(),
            "middleware" => "#a855f7".to_string(),
            "config" => "#06b6d4".to_string(),
            "worker" => "#f97316".to_string(),
            "blockchain" => "#f43f5e".to_string(),
            "observability" => "#ec4899".to_string(),
            "auth" => "#e11d48".to_string(),
            "utility" => "#64748b".to_string(),
            "api" => "#6366f1".to_string(),
            "cli" => "#475569".to_string(),
            "test" => "#d946ef".to_string(),
            "core" => "#475569".to_string(),
            "root" => "#eab308".to_string(),
            other => Self::hash_color(other),
        }
    }

    fn hash_color(name: &str) -> String {
        let mut hash: u32 = 5381;
        for b in name.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(b as u32);
        }
        let hue = hash % 360;
        format!("hsl({}, 70%, 54%)", hue)
    }

    fn layer_description(layer: &str) -> String {
        match layer {
            "handler" => "Receives requests and kicks off the work".to_string(),
            "service" => "Business logic — the actual rules of the app".to_string(),
            "repository" => "Reads and writes data storage".to_string(),
            "middleware" => "Runs on every request before it reaches handlers".to_string(),
            "config" => "Settings and environment setup".to_string(),
            "worker" => "Background jobs and scheduled tasks".to_string(),
            "blockchain" => "On-chain / smart contract interactions".to_string(),
            "observability" => "Logging, metrics, tracing".to_string(),
            "auth" => "Login, permissions, access control".to_string(),
            "utility" => "Shared helper functions".to_string(),
            "api" => "External-facing interface".to_string(),
            "cli" => "Command-line entry points".to_string(),
            "test" => "Test code".to_string(),
            "core" => "Core application code".to_string(),
            "root" => "Top-level project files".to_string(),
            other => format!("Code under the `{}` module", other),
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

    fn collect_function_nodes(call_graph: &CallGraph) -> Vec<serde_json::Value> {
        call_graph
            .node_indices()
            .map(|idx| {
                let func = &call_graph[idx];
                let layer = if func.layer.is_empty() {
                    "unknown".to_string()
                } else {
                    func.layer.clone()
                };
                serde_json::json!({
                    "id": idx.index(),
                    "name": func.name,
                    "file": func.file,
                    "line": func.line,
                    "layer": layer,
                    "importance": func.importance_score,
                })
            })
            .collect()
    }

    fn collect_function_edges(call_graph: &CallGraph) -> Vec<serde_json::Value> {
        let mut edges = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for idx in call_graph.node_indices() {
            let source = idx.index();
            for callee in call_graph.get_callees(idx) {
                if let Some(&callee_idx) = call_graph.name_index.get(&callee.full_path) {
                    let target = callee_idx.index();
                    if seen.insert((source, target)) {
                        edges.push(serde_json::json!({ "source": source, "target": target }));
                    }
                }
            }
        }
        edges
    }

    fn build_html(
        layers: &[LayerSummary],
        edges: &[LayerEdge],
        function_nodes: &[serde_json::Value],
        function_edges: &[serde_json::Value],
        project_name: &str,
    ) -> String {
        let nodes_json = serde_json::to_string(function_nodes).unwrap();
        let fn_edges_json = serde_json::to_string(function_edges).unwrap();

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
<title>{project_name} • Architecture Overview</title>
<link rel="preconnect" href="https://fonts.googleapis.com">
<link href="https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;500;600&display=swap" rel="stylesheet">
<style>
    :root {{
        --bg: #07090e;
        --card-bg: rgba(14, 18, 27, 0.75);
        --card-border: rgba(255, 255, 255, 0.08);
        --card-border-glow: rgba(56, 189, 248, 0.25);
        --text-main: #f8fafc;
        --text-muted: #94a3b8;
        --accent: #38bdf8;
    }}
    * {{ margin: 0; padding: 0; box-sizing: border-box; }}
    body {{
        font-family: 'Plus Jakarta Sans', -apple-system, BlinkMacSystemFont, sans-serif;
        background: var(--bg);
        color: var(--text-main);
        height: 100vh;
        width: 100vw;
        overflow: hidden;
        user-select: none;
    }}

    #viewport {{
        width: 100vw;
        height: 100vh;
        background-color: var(--bg);
        background-image:
            radial-gradient(at 50% 0%, rgba(56, 189, 248, 0.06) 0px, transparent 50%),
            radial-gradient(at 100% 100%, rgba(168, 85, 247, 0.04) 0px, transparent 50%),
            linear-gradient(rgba(255, 255, 255, 0.02) 1px, transparent 1px),
            linear-gradient(90deg, rgba(255, 255, 255, 0.02) 1px, transparent 1px);
        background-size: 100% 100%, 100% 100%, 36px 36px, 36px 36px;
    }}

    /* Compact Header Bar */
    #hud-header {{
        position: absolute;
        top: 16px;
        left: 16px;
        z-index: 10;
        display: flex;
        align-items: center;
        gap: 10px;
        background: var(--card-bg);
        backdrop-filter: blur(20px);
        -webkit-backdrop-filter: blur(20px);
        border: 1px solid var(--card-border);
        border-radius: 12px;
        padding: 8px 14px;
        box-shadow: 0 12px 30px rgba(0, 0, 0, 0.45);
        pointer-events: none;
    }}
    .hud-icon {{
        display: flex;
        align-items: center;
        justify-content: center;
        width: 28px;
        height: 28px;
        background: rgba(56, 189, 248, 0.12);
        border: 1px solid rgba(56, 189, 248, 0.3);
        border-radius: 8px;
        color: var(--accent);
        font-size: 14px;
    }}
    #hud-header h1 {{
        font-size: 14px;
        font-weight: 800;
        letter-spacing: -0.01em;
    }}
    #hud-header p {{
        font-size: 11.5px;
        color: var(--text-muted);
    }}

    #back-btn {{
        position: absolute;
        top: 16px;
        right: 16px;
        z-index: 20;
        display: none;
        align-items: center;
        gap: 6px;
        background: rgba(14, 18, 27, 0.85);
        backdrop-filter: blur(16px);
        border: 1px solid rgba(56, 189, 248, 0.35);
        color: #fff;
        padding: 8px 14px;
        border-radius: 10px;
        font-family: inherit;
        font-size: 12px;
        font-weight: 700;
        cursor: pointer;
        transition: all 0.2s;
        box-shadow: 0 10px 24px rgba(0, 0, 0, 0.4);
    }}
    #back-btn:hover {{
        background: rgba(56, 189, 248, 0.2);
        border-color: var(--accent);
    }}

    #tooltip {{
        position: absolute;
        z-index: 1000;
        padding: 12px 14px;
        border-radius: 10px;
        background: rgba(10, 13, 20, 0.95);
        backdrop-filter: blur(16px);
        border: 1px solid var(--card-border-glow);
        font-size: 12px;
        pointer-events: none;
        display: none;
        max-width: 300px;
        box-shadow: 0 20px 40px rgba(0, 0, 0, 0.7);
    }}
    #tooltip .t-title {{
        font-weight: 700;
        font-size: 12.5px;
        margin-bottom: 4px;
        color: var(--text-main);
        font-family: 'JetBrains Mono', monospace;
    }}
    #tooltip .t-desc {{
        color: var(--text-muted);
        line-height: 1.4;
        margin-bottom: 6px;
    }}
    #tooltip .t-fn {{
        font-size: 11px;
        color: var(--accent);
        font-family: 'JetBrains Mono', monospace;
        margin-top: 2px;
    }}

    .layer-box {{ cursor: pointer; }}
    .layer-box rect {{
        stroke-width: 1.5px;
        rx: 12px;
        transition: filter 0.2s;
    }}
    .layer-box:hover rect {{
        filter: drop-shadow(0 0 12px rgba(255, 255, 255, 0.2)) brightness(1.2);
    }}
    .layer-label {{
        font-weight: 800;
        font-size: 12px;
        fill: #fff;
        text-anchor: middle;
        pointer-events: none;
    }}
    .layer-count {{
        font-size: 10px;
        font-weight: 600;
        fill: rgba(255, 255, 255, 0.75);
        text-anchor: middle;
        pointer-events: none;
    }}

    .edge-path {{
        fill: none;
        stroke: rgba(148, 163, 184, 0.22);
    }}
    .fn-edge {{
        stroke: rgba(148, 163, 184, 0.12);
        fill: none;
        stroke-width: 1.2px;
    }}
    .fn-edge.active {{
        stroke: var(--accent) !important;
        stroke-width: 2px !important;
        stroke-opacity: 1 !important;
    }}

    .fn-node circle {{
        stroke-width: 1.5px;
        cursor: pointer;
    }}
    .fn-node:hover circle {{
        filter: drop-shadow(0 0 8px #fff) brightness(1.3);
    }}
    .fn-node text {{
        font-family: 'JetBrains Mono', monospace;
        font-size: 10px;
        fill: rgba(255, 255, 255, 0.85);
        text-anchor: middle;
        pointer-events: none;
        text-shadow: 0 2px 6px rgba(0, 0, 0, 0.95);
    }}

    .ext-port {{ cursor: pointer; }}
    .ext-port rect {{ rx: 12px; stroke-width: 1.5px; }}
    .ext-port:hover rect {{ filter: drop-shadow(0 0 10px rgba(255, 255, 255, 0.2)) brightness(1.2); }}
    .ext-port text {{ fill: #fff; text-anchor: middle; pointer-events: none; font-weight: 700; }}
    .ext-edge {{ fill: none; stroke-dasharray: 4 4; opacity: 0.3; }}
    .ext-edge.active {{ opacity: 1; stroke-dasharray: none; stroke-width: 2px !important; }}

    .dimmed {{ opacity: 0.1 !important; }}
</style>
</head>
<body>
    <div id="hud-header">
        <div class="hud-icon">⚡</div>
        <div>
            <h1>{project_name}</h1>
            <p id="subtitle">Architecture overview — hover to inspect, click to explore</p>
        </div>
    </div>

    <button id="back-btn" onclick="resetView()">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="19" y1="12" x2="5" y2="12"></line><polyline points="12 19 5 12 12 5"></polyline></svg>
        Back
    </button>

    <div id="viewport"></div>
    <div id="tooltip"></div>

    <script src="https://d3js.org/d3.v7.min.js"></script>
    <script>
        const layersData = {layers_json};
        const layerEdgesData = {edges_json};
        const nodesData = {nodes_json};
        const fnEdgesData = {fn_edges_json};

        const nodeById = new Map(nodesData.map(n => [n.id, n]));

        const width = window.innerWidth;
        const height = window.innerHeight;
        const svg = d3.select("#viewport").append("svg")
            .attr("width", width).attr("height", height);

        const defs = svg.append("defs");
        defs.append("marker")
            .attr("id", "arrow-default")
            .attr("viewBox", "0 -5 10 10")
            .attr("refX", 18).attr("refY", 0)
            .attr("markerWidth", 5).attr("markerHeight", 5)
            .attr("orient", "auto")
            .append("path")
            .attr("d", "M0,-5L10,0L0,5")
            .attr("fill", "rgba(148,163,184,0.6)");

        const zoomG = svg.append("g");
        const zoom = d3.zoom().scaleExtent([0.3, 3]).on("zoom", (e) => {{
            zoomG.attr("transform", e.transform);
        }});
        svg.call(zoom);

        const gOverview = zoomG.append("g").attr("id", "g-overview");
        const gDetail = zoomG.append("g").attr("id", "g-detail").style("display", "none");

        const tooltip = document.getElementById("tooltip");
        const backBtn = document.getElementById("back-btn");
        const subtitle = document.getElementById("subtitle");

        let activeSim = null;

        // ---------- Overview: Balanced Ring Layout ----------
        const cx = width / 2;
        // Shift center slightly down to clear HUD, but preserve equal top/bottom clearance
        const cy = height / 2 + 15;
        const maxCount = Math.max(...layersData.map(l => l.count));

        layersData.forEach(l => {{
            l.w = 78 + Math.sqrt(l.count / maxCount) * 60;
            l.h = 48;
        }});

        // Constrain radius within safe screen bounds to prevent top HUD collision and bottom cutoff
        const maxRadiusY = (height / 2) - 80;
        const maxRadiusX = (width / 2) - 100;
        const idealRadius = Math.min(maxRadiusX, maxRadiusY);

        const totalW = layersData.reduce((s, l) => s + l.w + 24, 0);
        const ringRadius = Math.min(idealRadius, Math.max(180, totalW / (2 * Math.PI)));

        let cum = 0;
        layersData.forEach(l => {{
            const frac = (l.w + 24) / totalW;
            l.angle = cum * 2 * Math.PI + (frac * 2 * Math.PI) / 2 - Math.PI / 2;
            cum += frac;
            l.x = cx + ringRadius * Math.cos(l.angle);
            l.y = cy + ringRadius * Math.sin(l.angle);
        }});
        const layerByName = new Map(layersData.map(l => [l.name, l]));
        const maxWeight = Math.max(1, ...layerEdgesData.map(e => e.weight));

        gOverview.selectAll(".edge-path")
            .data(layerEdgesData.filter(e => layerByName.has(e.from) && layerByName.has(e.to)))
            .enter().append("path")
            .attr("class", "edge-path")
            .attr("stroke-width", e => 1.2 + (e.weight / maxWeight) * 4)
            .attr("d", e => {{
                const a = layerByName.get(e.from), b = layerByName.get(e.to);
                const mx = (a.x + b.x) / 2, my = (a.y + b.y) / 2;
                return `M${{a.x}},${{a.y}} Q${{mx}},${{my}} ${{b.x}},${{b.y}}`;
            }});

        const layerNode = gOverview.selectAll(".layer-box")
            .data(layersData)
            .enter().append("g")
            .attr("class", "layer-box")
            .attr("transform", l => `translate(${{l.x - l.w/2}},${{l.y - l.h/2}})`);

        layerNode.append("rect")
            .attr("width", l => l.w).attr("height", l => l.h)
            .attr("fill", l => l.color)
            .attr("fill-opacity", 0.88)
            .attr("stroke", l => d3.rgb(l.color).brighter(0.4));

        layerNode.append("text").attr("class", "layer-label")
            .attr("x", l => l.w / 2).attr("y", l => l.h / 2 - 3)
            .text(l => l.name);

        layerNode.append("text").attr("class", "layer-count")
            .attr("x", l => l.w / 2).attr("y", l => l.h / 2 + 13)
            .text(l => `${{l.count}} functions`);

        layerNode.on("mousemove", (event, l) => {{
            tooltip.style.display = "block";
            tooltip.style.left = (event.pageX + 16) + "px";
            tooltip.style.top = (event.pageY - 10) + "px";
            tooltip.innerHTML = `
                <div class="t-title">${{l.name}}</div>
                <div class="t-desc">${{l.description}}</div>
                ${{l.top_functions.map(f => `<div class="t-fn">• ${{f}}</div>`).join('')}}
                <div class="t-fn" style="margin-top:6px; color: var(--accent);">Click to explore internal functions &rarr;</div>
            `;
        }}).on("mouseout", () => {{ tooltip.style.display = "none"; }})
          .on("click", (event, l) => showDetail(l.name));

        // ---------- Detail View ----------
        function showDetail(layerName) {{
            if (activeSim) activeSim.stop();
            svg.call(zoom.transform, d3.zoomIdentity);

            tooltip.style.display = "none";
            gOverview.style("display", "none");
            gDetail.style("display", null);
            gDetail.selectAll("*").remove();
            backBtn.style.display = "flex";
            subtitle.textContent = `${{layerName}} — hover nodes to trace calls; click external ports to jump`;

            const members = nodesData.filter(n => n.layer === layerName);
            const memberIds = new Set(members.map(n => n.id));
            const internal = fnEdgesData.filter(e => memberIds.has(e.source) && memberIds.has(e.target));

            const portAgg = new Map();
            fnEdgesData.forEach(e => {{
                const sIn = memberIds.has(e.source), tIn = memberIds.has(e.target);
                if (sIn && !tIn) {{
                    const other = nodeById.get(e.target);
                    if (other) {{
                        const p = portAgg.get(other.layer) || {{ layer: other.layer, out: 0, in: 0 }};
                        p.out++;
                        portAgg.set(other.layer, p);
                    }}
                }} else if (!sIn && tIn) {{
                    const other = nodeById.get(e.source);
                    if (other) {{
                        const p = portAgg.get(other.layer) || {{ layer: other.layer, out: 0, in: 0 }};
                        p.in++;
                        portAgg.set(other.layer, p);
                    }}
                }}
            }});

            const ports = Array.from(portAgg.values());
            const dcx = width / 2, dcy = height / 2 + 15;
            const outerR = Math.min((height / 2) - 80, (width / 2) - 100);

            ports.forEach((p, i) => {{
                const a = (i / Math.max(1, ports.length)) * 2 * Math.PI - Math.PI / 2;
                p.x = dcx + outerR * Math.cos(a);
                p.y = dcy + outerR * Math.sin(a);
            }});

            const maxImp = Math.max(0.01, ...members.map(m => m.importance));
            members.forEach(m => {{
                m.r = 4.5 + (m.importance / maxImp) * 8;
                m.showLabel = members.length <= 25 || m.importance >= maxImp * 0.5;
            }});

            const memberById = new Map(members.map(m => [m.id, m]));

            const maxPortCalls = Math.max(1, ...ports.map(p => p.out + p.in));
            const extEdgeGroup = gDetail.append("g").attr("class", "ext-edges");
            extEdgeGroup.selectAll(".ext-edge")
                .data(ports)
                .enter().append("path")
                .attr("class", "ext-edge")
                .attr("stroke", p => layerByName.has(p.layer) ? layerByName.get(p.layer).color : "#94a3b8")
                .attr("stroke-width", p => 1.5 + ((p.out + p.in) / maxPortCalls) * 3.5)
                .attr("d", p => `M${{dcx}},${{dcy}} L${{p.x}},${{p.y}}`);

            const internalEdgeGroup = gDetail.append("g").attr("class", "internal-edges");
            const fnEdges = internalEdgeGroup.selectAll(".fn-edge")
                .data(internal)
                .enter().append("path")
                .attr("class", "fn-edge")
                .attr("marker-end", "url(#arrow-default)");

            const layerColor = layerByName.has(layerName) ? layerByName.get(layerName).color : "#94a3b8";
            const nodeGroup = gDetail.append("g").attr("class", "fn-nodes");
            const fnNodes = nodeGroup.selectAll(".fn-node")
                .data(members)
                .enter().append("g")
                .attr("class", "fn-node");

            fnNodes.append("circle")
                .attr("r", m => m.r)
                .attr("fill", layerColor)
                .attr("fill-opacity", 0.9)
                .attr("stroke", "#fff")
                .attr("stroke-opacity", 0.4);

            fnNodes.append("text")
                .attr("dy", m => m.r + 11)
                .style("display", m => m.showLabel ? "block" : "none")
                .text(m => m.name.length > 16 ? m.name.slice(0, 14) + "…" : m.name);

            const portGroup = gDetail.append("g").attr("class", "ext-ports");
            const portNodes = portGroup.selectAll(".ext-port")
                .data(ports)
                .enter().append("g")
                .attr("class", "ext-port")
                .attr("transform", p => `translate(${{p.x - 55}},${{p.y - 18}})`);

            portNodes.append("rect")
                .attr("width", 110).attr("height", 36)
                .attr("fill", p => layerByName.has(p.layer) ? layerByName.get(p.layer).color : "#94a3b8")
                .attr("fill-opacity", 0.92)
                .attr("stroke", p => d3.rgb(layerByName.has(p.layer) ? layerByName.get(p.layer).color : "#94a3b8").brighter(0.4));

            portNodes.append("text")
                .attr("x", 55).attr("y", 15)
                .attr("font-size", "11px")
                .text(p => p.layer);

            portNodes.append("text")
                .attr("x", 55).attr("y", 28)
                .style("font-size", "9px").style("opacity", 0.9)
                .text(p => `${{p.out > 0 ? '→ ' + p.out : ''}} ${{p.in > 0 ? '← ' + p.in : ''}} calls`);

            portNodes.on("click", (event, p) => showDetail(p.layer));

            activeSim = d3.forceSimulation(members)
                .force("center", d3.forceCenter(dcx, dcy))
                .force("charge", d3.forceManyBody().strength(members.length > 100 ? -20 : -50))
                .force("collide", d3.forceCollide().radius(m => m.r + 6))
                .force("radial", d3.forceRadial(outerR * 0.55, dcx, dcy).strength(0.75))
                .on("tick", () => {{
                    fnNodes.attr("transform", m => `translate(${{m.x}},${{m.y}})`);
                    fnEdges.attr("d", e => {{
                        const a = memberById.get(e.source), b = memberById.get(e.target);
                        if (!a || !b) return "";
                        return `M${{a.x}},${{a.y}} L${{b.x}},${{b.y}}`;
                    }});
                }});

            fnNodes.on("mouseenter", (event, m) => {{
                const connectedTargets = new Set();
                const connectedSources = new Set();

                fnEdges.classed("active", e => {{
                    const isSrc = (e.source.id || e.source) === m.id;
                    const isTgt = (e.target.id || e.target) === m.id;
                    if (isSrc) connectedTargets.add(e.target.id || e.target);
                    if (isTgt) connectedSources.add(e.source.id || e.source);
                    return isSrc || isTgt;
                }}).classed("dimmed", e => {{
                    const isSrc = (e.source.id || e.source) === m.id;
                    const isTgt = (e.target.id || e.target) === m.id;
                    return !(isSrc || isTgt);
                }});

                fnNodes.classed("dimmed", n => n.id !== m.id && !connectedTargets.has(n.id) && !connectedSources.has(n.id));
                d3.select(event.currentTarget).select("text").style("display", "block");

                tooltip.style.display = "block";
                tooltip.style.left = (event.pageX + 16) + "px";
                tooltip.style.top = (event.pageY - 10) + "px";
                tooltip.innerHTML = `
                    <div class="t-title">${{m.name}}()</div>
                    <div class="t-desc">${{m.file}}:${{m.line}}</div>
                    <div class="t-fn">Score: ${{Number(m.importance).toFixed(3)}}</div>
                `;
            }}).on("mouseleave", (event, m) => {{
                fnEdges.classed("active", false).classed("dimmed", false);
                fnNodes.classed("dimmed", false);
                if (!m.showLabel) d3.select(event.currentTarget).select("text").style("display", "none");
                tooltip.style.display = "none";
            }});
        }}

        window.resetView = function() {{
            if (activeSim) activeSim.stop();
            gDetail.style("display", "none");
            gOverview.style("display", null);
            backBtn.style.display = "none";
            subtitle.textContent = "Architecture overview: hover to inspect, click to explore";
            svg.call(zoom.transform, d3.zoomIdentity);
        }};
    </script>
</body>
</html>"###
        )
    }
}
