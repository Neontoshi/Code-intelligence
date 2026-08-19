// src/output/interactive.rs

use crate::graph::call_graph::{CallGraph, FunctionNode};
use crate::graph::traits::GraphMetrics;
use crate::parser::tree_sitter::ParsedFile;
use serde_json::json;

pub struct InteractiveGraph;

impl InteractiveGraph {
    pub fn generate(call_graph: &CallGraph, files: &[ParsedFile], project_name: &str) -> String {
        let nodes_data = Self::collect_nodes(call_graph);
        let edges_data = Self::collect_edges(call_graph);
        let stats = Self::collect_stats(call_graph, files);

        Self::build_html_d3(&nodes_data, &edges_data, &stats, project_name)
    }

    pub fn generate_limited(
        call_graph: &CallGraph,
        files: &[ParsedFile],
        project_name: &str,
        max_nodes: usize,
    ) -> String {
        let nodes_data = Self::collect_nodes_limited(call_graph, max_nodes);
        let edges_data = Self::collect_edges_limited(call_graph, &nodes_data);
        let stats = Self::collect_stats(call_graph, files);

        Self::build_html_d3(&nodes_data, &edges_data, &stats, project_name)
    }

    fn collect_nodes(call_graph: &CallGraph) -> Vec<serde_json::Value> {
        use crate::analysis::{
            dead_code::is_never_dead,
            roots::{ReachabilityAnalyzer, RootDetectionConfig, RootDetector},
            verdict::{VerdictConfig, VerdictEngine},
        };

        // 1. Detect roots
        let root_config = RootDetectionConfig::default();
        let root_set = RootDetector::detect_roots(call_graph, &[], &root_config);

        // 2. Compute reachability
        let reachability = ReachabilityAnalyzer::compute_reachability(call_graph, &root_set);

        // 3. Create verdict engine
        let verdict_engine = VerdictEngine::new(VerdictConfig::default());

        // 4. Generate verdicts for all functions (or use existing ones)
        let verdicts = verdict_engine.evaluate_all(call_graph, &reachability);

        // Build a map of full_path -> is_dead
        let mut dead_map = std::collections::HashMap::new();
        for verdict in verdicts {
            if verdict.is_dead() {
                dead_map.insert(verdict.full_path.clone(), true);
            }
        }

        let mut nodes = Vec::new();
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];

            // Check if the verdict engine says it's dead AND it passes the filters
            let is_dead = dead_map.contains_key(&func.full_path) && !is_never_dead(func);

            nodes.push(json!({
                "id": idx.index(),
                "label": func.name,
                "full_path": func.full_path,
                "file": func.file,
                "line": func.line,
                "is_public": func.is_public,
                "is_async": func.is_async,
                "complexity": func.complexity,
                "importance": func.importance_score,
                "fan_in": func.fan_in,
                "fan_out": func.fan_out,
                "layer": func.layer,
                "is_dead": is_dead,
                "is_test": func.is_test,
                "is_trait_method": func.is_trait_method,
                "size": Self::calculate_node_size(func),
                "color": Self::calculate_node_color(func, is_dead),
            }));
        }

        nodes
    }

    fn collect_nodes_limited(call_graph: &CallGraph, max_nodes: usize) -> Vec<serde_json::Value> {
        use crate::analysis::{
            dead_code::is_never_dead,
            roots::{ReachabilityAnalyzer, RootDetectionConfig, RootDetector},
            verdict::{VerdictConfig, VerdictEngine},
        };

        // 1. Detect roots
        let root_config = RootDetectionConfig::default();
        let root_set = RootDetector::detect_roots(call_graph, &[], &root_config);

        // 2. Compute reachability
        let reachability = ReachabilityAnalyzer::compute_reachability(call_graph, &root_set);

        // 3. Create verdict engine
        let verdict_engine = VerdictEngine::new(VerdictConfig::default());

        // 4. Generate verdicts
        let verdicts = verdict_engine.evaluate_all(call_graph, &reachability);

        // Build a map of full_path -> is_dead
        let mut dead_map = std::collections::HashMap::new();
        for verdict in verdicts {
            if verdict.is_dead() {
                dead_map.insert(verdict.full_path.clone(), true);
            }
        }

        let mut nodes = Vec::new();
        let selected = call_graph.top_important_nodes(max_nodes, 0.0);
        let selected_set: std::collections::HashSet<_> = selected.iter().copied().collect();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            let is_selected = selected_set.contains(&idx);
            let is_dead = dead_map.contains_key(&func.full_path) && !is_never_dead(func);

            nodes.push(json!({
                "id": idx.index(),
                "label": func.name,
                "full_path": func.full_path,
                "file": func.file,
                "line": func.line,
                "is_public": func.is_public,
                "is_async": func.is_async,
                "complexity": func.complexity,
                "importance": func.importance_score,
                "fan_in": func.fan_in,
                "fan_out": func.fan_out,
                "layer": func.layer,
                "is_dead": is_dead,
                "is_test": func.is_test,
                "is_trait_method": func.is_trait_method,
                "size": Self::calculate_node_size(func),
                "color": if is_selected {
                    Self::calculate_node_color(func, is_dead)
                } else {
                    "#2d2d44".to_string()
                },
                "hidden": !is_selected,
            }));
        }

        nodes
    }

    fn collect_edges(call_graph: &CallGraph) -> Vec<serde_json::Value> {
        let mut edges = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for idx in call_graph.node_indices() {
            let source = idx.index();
            for callee in call_graph.get_callees(idx) {
                if let Some(callee_idx) = call_graph.name_index.get(&callee.full_path) {
                    let target = callee_idx.index();
                    let key = (source, target);
                    if !seen.contains(&key) {
                        seen.insert(key);
                        edges.push(json!({
                            "source": source,
                            "target": target,
                        }));
                    }
                }
            }
        }

        edges
    }

    fn collect_edges_limited(
        call_graph: &CallGraph,
        nodes_data: &[serde_json::Value],
    ) -> Vec<serde_json::Value> {
        let selected_ids: std::collections::HashSet<u64> = nodes_data
            .iter()
            .filter_map(|node| node["id"].as_u64())
            .collect();

        let mut edges = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for idx in call_graph.node_indices() {
            let source = idx.index() as u64;
            if !selected_ids.contains(&source) {
                continue;
            }

            for callee in call_graph.get_callees(idx) {
                if let Some(callee_idx) = call_graph.name_index.get(&callee.full_path) {
                    let target = callee_idx.index() as u64;
                    if !selected_ids.contains(&target) {
                        continue;
                    }
                    let key = (source, target);
                    if !seen.contains(&key) {
                        seen.insert(key);
                        edges.push(json!({
                            "source": source,
                            "target": target,
                        }));
                    }
                }
            }
        }

        edges
    }

    fn collect_stats(call_graph: &CallGraph, files: &[ParsedFile]) -> serde_json::Value {
        use crate::analysis::{
            dead_code::is_never_dead,
            roots::{ReachabilityAnalyzer, RootDetectionConfig, RootDetector},
            verdict::{VerdictConfig, VerdictEngine},
        };

        // 1. Detect roots
        let root_config = RootDetectionConfig::default();
        let root_set = RootDetector::detect_roots(call_graph, &[], &root_config);

        // 2. Compute reachability
        let reachability = ReachabilityAnalyzer::compute_reachability(call_graph, &root_set);

        // 3. Create verdict engine
        let verdict_engine = VerdictEngine::new(VerdictConfig::default());

        // 4. Generate verdicts
        let verdicts = verdict_engine.evaluate_all(call_graph, &reachability);

        // Build a map of full_path -> is_dead
        let mut dead_map = std::collections::HashMap::new();
        for verdict in verdicts {
            if verdict.is_dead() {
                dead_map.insert(verdict.full_path.clone(), true);
            }
        }

        // Count dead functions using verdict engine + filters
        let total_functions = call_graph.node_count();
        let dead_functions = call_graph
            .node_indices()
            .filter(|idx| {
                let func = &call_graph[*idx];
                dead_map.contains_key(&func.full_path) && !is_never_dead(func)
            })
            .count();
        let test_functions = call_graph
            .node_indices()
            .filter(|idx| call_graph[*idx].is_test)
            .count();
        let public_functions = call_graph
            .node_indices()
            .filter(|idx| call_graph[*idx].is_public)
            .count();
        let async_functions = call_graph
            .node_indices()
            .filter(|idx| call_graph[*idx].is_async)
            .count();

        let mut languages = std::collections::HashSet::new();
        for file in files {
            languages.insert(file.language.clone());
        }

        json!({
            "total_functions": total_functions,
            "dead_functions": dead_functions,
            "test_functions": test_functions,
            "public_functions": public_functions,
            "async_functions": async_functions,
            "total_edges": call_graph.edge_count(),
            "total_files": files.len(),
            "languages": languages.into_iter().collect::<Vec<String>>(),
        })
    }

    fn calculate_node_size(func: &FunctionNode) -> usize {
        let base = 10;
        let importance_factor = (func.importance_score * 20.0) as usize;
        let complexity_factor = (func.complexity / 5.0) as usize;
        base + importance_factor + complexity_factor
    }

    fn calculate_node_color(func: &FunctionNode, is_dead: bool) -> String {
        if func.is_test {
            "#8e44ad".to_string() // Purple - test functions
        } else if func.is_trait_method {
            "#2ecc71".to_string() // Green - trait methods
        } else if is_dead {
            "#e74c3c".to_string() // Red - truly dead code (using verdict engine)
        } else if func.is_public {
            "#3498db".to_string() // Blue - public API
        } else if func.importance_score > 0.7 {
            "#f39c12".to_string() // Orange - important
        } else {
            "#95a5a6".to_string() // Gray - normal
        }
    }

    fn build_html_d3(
        nodes: &[serde_json::Value],
        edges: &[serde_json::Value],
        stats: &serde_json::Value,
        project_name: &str,
    ) -> String {
        let nodes_json = serde_json::to_string(nodes).unwrap();
        let edges_json = serde_json::to_string(edges).unwrap();
        let stats_json = serde_json::to_string(stats).unwrap();

        format!(
            r###"<!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Call Graph - {project_name}</title>
        <link rel="preconnect" href="https://fonts.googleapis.com">
        <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
        <link href="https://fonts.googleapis.com/css2?family=Fira+Code:wght@400;500;600&family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
        <style>
            :root {{
                --bg-base: #0f111a;
                --bg-surface: rgba(22, 27, 44, 0.85);
                --bg-surface-hover: rgba(30, 38, 64, 0.95);
                --border-subtle: rgba(255, 255, 255, 0.08);
                --border-focus: #38bdf8;
                --text-primary: #f8fafc;
                --text-secondary: #94a3b8;
                --text-muted: #64748b;

                --color-dead: #f43f5e;
                --color-public: #38bdf8;
                --color-important: #fbbf24;
                --color-trait: #34d399;
                --color-test: #c084fc;
                --color-normal: #64748b;
                --edge-default: rgba(148, 163, 184, 0.2);
                --edge-highlight: #38bdf8;
            }}

            * {{ margin: 0; padding: 0; box-sizing: border-box; }}

            body {{
                font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
                background: var(--bg-base);
                color: var(--text-primary);
                height: 100vh;
                width: 100vw;
                overflow: hidden;
                -webkit-font-smoothing: antialiased;
            }}

            #container {{
                width: 100vw;
                height: 100vh;
                cursor: grab;
            }}
            #container:active {{
                cursor: grabbing;
            }}

            /* Loading Screen */
            #loading {{
                position: absolute;
                inset: 0;
                background: var(--bg-base);
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                z-index: 2000;
                gap: 16px;
            }}
            .spinner {{
                width: 48px;
                height: 48px;
                border: 3px solid var(--border-subtle);
                border-top-color: var(--color-public);
                border-radius: 50%;
                animation: spin 0.8s cubic-bezier(0.4, 0, 0.2, 1) infinite;
            }}
            @keyframes spin {{ to {{ transform: rotate(360deg); }} }}

            /* Glassmorphism Panels */
            .glass-panel {{
                background: var(--bg-surface);
                backdrop-filter: blur(16px);
                -webkit-backdrop-filter: blur(16px);
                border: 1px solid var(--border-subtle);
                border-radius: 12px;
                box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.5), 0 8px 10px -6px rgba(0, 0, 0, 0.5);
            }}

            /* Header / Controls */
            #controls {{
                position: absolute;
                top: 24px;
                left: 24px;
                width: 320px;
                z-index: 100;
                padding: 16px;
                display: flex;
                flex-direction: column;
                gap: 14px;
            }}

            .brand {{
                display: flex;
                align-items: center;
                justify-content: space-between;
            }}
            .brand h1 {{
                font-size: 15px;
                font-weight: 700;
                letter-spacing: -0.02em;
                color: var(--text-primary);
                white-space: nowrap;
                overflow: hidden;
                text-overflow: ellipsis;
            }}

            .stats-grid {{
                display: grid;
                grid-template-columns: repeat(3, 1fr);
                gap: 8px;
            }}
            .stat-card {{
                background: rgba(0, 0, 0, 0.25);
                border: 1px solid var(--border-subtle);
                padding: 8px;
                border-radius: 8px;
                display: flex;
                flex-direction: column;
                gap: 2px;
            }}
            .stat-card .label {{
                font-size: 10px;
                text-transform: uppercase;
                font-weight: 600;
                letter-spacing: 0.05em;
                color: var(--text-muted);
            }}
            .stat-card .val {{
                font-size: 14px;
                font-weight: 700;
                font-family: 'Fira Code', monospace;
            }}

            .search-box {{
                position: relative;
                width: 100%;
            }}
            #search {{
                width: 100%;
                padding: 9px 12px;
                background: rgba(0, 0, 0, 0.35);
                border: 1px solid var(--border-subtle);
                border-radius: 8px;
                color: var(--text-primary);
                font-size: 13px;
                outline: none;
                transition: all 0.2s ease;
            }}
            #search:focus {{
                border-color: var(--border-focus);
                box-shadow: 0 0 0 2px rgba(56, 189, 248, 0.2);
            }}

            /* Legend */
            #legend {{
                position: absolute;
                bottom: 24px;
                left: 24px;
                padding: 12px 16px;
                z-index: 100;
                display: flex;
                gap: 16px;
                align-items: center;
            }}
            .legend-item {{
                display: flex;
                align-items: center;
                gap: 6px;
                font-size: 11px;
                font-weight: 500;
                color: var(--text-secondary);
            }}
            .legend-dot {{
                width: 8px;
                height: 8px;
                border-radius: 50%;
            }}

            /* Details Sidebar */
            #inspector {{
                position: absolute;
                top: 24px;
                right: 24px;
                width: 320px;
                z-index: 100;
                padding: 16px;
                display: none;
                flex-direction: column;
                gap: 12px;
            }}
            #inspector.active {{
                display: flex;
            }}
            .inspector-header {{
                display: flex;
                justify-content: space-between;
                align-items: flex-start;
                border-bottom: 1px solid var(--border-subtle);
                padding-bottom: 10px;
            }}
            .node-title {{
                font-family: 'Fira Code', monospace;
                font-size: 14px;
                font-weight: 600;
                word-break: break-all;
                color: var(--color-public);
            }}
            .close-btn {{
                background: none;
                border: none;
                color: var(--text-muted);
                cursor: pointer;
                font-size: 16px;
                line-height: 1;
            }}
            .close-btn:hover {{ color: var(--text-primary); }}

            .meta-row {{
                display: flex;
                justify-content: space-between;
                font-size: 12px;
                padding: 4px 0;
            }}
            .meta-label {{ color: var(--text-muted); }}
            .meta-value {{
                font-family: 'Fira Code', monospace;
                font-weight: 500;
                color: var(--text-primary);
            }}

            /* Floating Nav Controls */
            #zoom-controls {{
                position: absolute;
                bottom: 24px;
                right: 24px;
                z-index: 100;
                display: flex;
                flex-direction: column;
                gap: 6px;
            }}
            .btn-icon {{
                width: 36px;
                height: 36px;
                border-radius: 8px;
                background: var(--bg-surface);
                border: 1px solid var(--border-subtle);
                color: var(--text-primary);
                display: flex;
                align-items: center;
                justify-content: center;
                cursor: pointer;
                transition: all 0.2s ease;
                font-size: 14px;
                font-weight: bold;
            }}
            .btn-icon:hover {{
                background: var(--bg-surface-hover);
                border-color: var(--border-focus);
            }}

            /* D3 SVG Elements */
            .node-circle {{
                stroke-width: 2px;
                cursor: pointer;
                transition: stroke 0.2s, stroke-width 0.2s;
            }}
            .node-circle:hover, .node-circle.selected {{
                stroke: #ffffff !important;
                stroke-width: 3px !important;
            }}
            .node-label {{
                font-family: 'Inter', sans-serif;
                font-weight: 500;
                fill: #f8fafc;
                pointer-events: none;
                text-anchor: middle;
                dominant-baseline: central;
                text-shadow: 0 1px 4px rgba(0, 0, 0, 0.8), 0 0 2px rgba(0, 0, 0, 0.9);
            }}
            line.link {{
                stroke: var(--edge-default);
                stroke-opacity: 0.6;
                transition: stroke 0.2s, stroke-opacity 0.2s;
            }}
            line.link.highlighted {{
                stroke: var(--edge-highlight);
                stroke-opacity: 1;
                stroke-width: 2.5px;
            }}
        </style>
    </head>
    <body>
        <div id="loading">
            <div class="spinner"></div>
            <div style="font-size: 14px; color: var(--text-secondary);">Initializing Call Graph...</div>
        </div>

        <div id="container"></div>

        <div id="controls" class="glass-panel">
            <div class="brand">
                <h1 title="{project_name}">📦 {project_name}</h1>
            </div>
            <div class="stats-grid">
                <div class="stat-card">
                    <span class="label">Functions</span>
                    <span class="val" id="stat-fn">0</span>
                </div>
                <div class="stat-card">
                    <span class="label">Edges</span>
                    <span class="val" id="stat-edges">0</span>
                </div>
                <div class="stat-card">
                    <span class="label" style="color: var(--color-dead);">Dead</span>
                    <span class="val" style="color: var(--color-dead);" id="stat-dead">0</span>
                </div>
            </div>
            <div class="search-box">
                <input type="text" id="search" placeholder="Search functions or files..." autocomplete="off" />
            </div>
        </div>

        <div id="legend" class="glass-panel">
            <div class="legend-item"><span class="legend-dot" style="background: var(--color-dead);"></span> Dead</div>
            <div class="legend-item"><span class="legend-dot" style="background: var(--color-public);"></span> Public API</div>
            <div class="legend-item"><span class="legend-dot" style="background: var(--color-important);"></span> Important</div>
            <div class="legend-item"><span class="legend-dot" style="background: var(--color-trait);"></span> Trait Method</div>
            <div class="legend-item"><span class="legend-dot" style="background: var(--color-test);"></span> Test</div>
            <div class="legend-item"><span class="legend-dot" style="background: var(--color-normal);"></span> Internal</div>
        </div>

        <div id="inspector" class="glass-panel">
            <div class="inspector-header">
                <div class="node-title" id="insp-title">-</div>
                <button class="close-btn" onclick="closeInspector()">&times;</button>
            </div>
            <div class="meta-row">
                <span class="meta-label">File</span>
                <span class="meta-value" id="insp-file">-</span>
            </div>
            <div class="meta-row">
                <span class="meta-label">Line</span>
                <span class="meta-value" id="insp-line">-</span>
            </div>
            <div class="meta-row">
                <span class="meta-label">Cyclomatic Complexity</span>
                <span class="meta-value" id="insp-complexity">-</span>
            </div>
            <div class="meta-row">
                <span class="meta-label">Fan-in / Fan-out</span>
                <span class="meta-value" id="insp-fan">-</span>
            </div>
            <div class="meta-row">
                <span class="meta-label">Dead Status</span>
                <span class="meta-value" id="insp-status">-</span>
            </div>
        </div>

        <div id="zoom-controls">
            <button class="btn-icon" id="zoom-in" title="Zoom In">+</button>
            <button class="btn-icon" id="zoom-out" title="Zoom Out">&minus;</button>
            <button class="btn-icon" id="zoom-reset" title="Reset View">&#x21bb;</button>
        </div>

        <script src="https://d3js.org/d3.v7.min.js"></script>
        <script>
            const nodesData = {nodes_json};
            const edgesData = {edges_json};
            const statsData = {stats_json};

            // Populate initial summary stats
            document.getElementById('stat-fn').textContent = statsData.total_functions;
            document.getElementById('stat-edges').textContent = statsData.total_edges;
            document.getElementById('stat-dead').textContent = statsData.dead_functions;

            const width = window.innerWidth;
            const height = window.innerHeight;

            const svg = d3.select("#container")
                .append("svg")
                .attr("width", width)
                .attr("height", height);

            // Arrow Marker Definitions
            svg.append("defs").selectAll("marker")
                .data(["end"])
                .enter().append("marker")
                .attr("id", "arrow")
                .attr("viewBox", "0 -5 10 10")
                .attr("refX", 22)
                .attr("refY", 0)
                .attr("markerWidth", 5)
                .attr("markerHeight", 5)
                .attr("orient", "auto")
                .append("path")
                .attr("d", "M0,-5L10,0L0,5")
                .attr("fill", "rgba(148, 163, 184, 0.4)");

            const g = svg.append("g");

            const zoom = d3.zoom()
                .scaleExtent([0.1, 4])
                .on("zoom", (event) => {{
                    g.attr("transform", event.transform);
                    // Adjust label visibility dynamically based on zoom depth
                    g.selectAll(".node-label")
                        .style("opacity", event.transform.k < 0.6 ? 0 : 1);
                }});

            svg.call(zoom);

            // Physics Simulation
            const simulation = d3.forceSimulation(nodesData)
                .force("link", d3.forceLink(edgesData).id(d => d.id).distance(120))
                .force("charge", d3.forceManyBody().strength(-350))
                .force("center", d3.forceCenter(width / 2, height / 2))
                .force("collide", d3.forceCollide().radius(d => Math.max(12, d.size / 2) + 12));

            // Links
            const link = g.append("g")
                .selectAll("line")
                .data(edgesData)
                .enter().append("line")
                .attr("class", "link")
                .attr("marker-end", "url(#arrow)");

            // Nodes
            const node = g.append("g")
                .selectAll("circle")
                .data(nodesData)
                .enter().append("circle")
                .attr("class", "node-circle")
                .attr("r", d => Math.max(8, d.size / 2))
                .attr("fill", d => d.color)
                .attr("stroke", d => d3.rgb(d.color).darker(0.8))
                .call(d3.drag()
                    .on("start", (event, d) => {{
                        if (!event.active) simulation.alphaTarget(0.3).restart();
                        d.fx = d.x;
                        d.fy = d.y;
                    }})
                    .on("drag", (event, d) => {{
                        d.fx = event.x;
                        d.fy = event.y;
                    }})
                    .on("end", (event, d) => {{
                        if (!event.active) simulation.alphaTarget(0);
                        d.fx = null;
                        d.fy = null;
                    }}));

            // Labels
            const label = g.append("g")
                .selectAll("text")
                .data(nodesData)
                .enter().append("text")
                .attr("class", "node-label")
                .attr("dy", d => Math.max(8, d.size / 2) + 12)
                .text(d => d.label)
                .style("font-size", "11px");

            // Interaction Handling
            node.on("click", (event, d) => {{
                event.stopPropagation();
                showInspector(d);

                // Highlight neighborhood
                const connectedNodeIds = new Set();
                connectedNodeIds.add(d.id);

                link.classed("highlighted", l => {{
                    const isConnected = l.source.id === d.id || l.target.id === d.id;
                    if (isConnected) {{
                        connectedNodeIds.add(l.source.id);
                        connectedNodeIds.add(l.target.id);
                    }}
                    return isConnected;
                }});

                node.style("opacity", n => connectedNodeIds.has(n.id) ? 1 : 0.15);
                label.style("opacity", n => connectedNodeIds.has(n.id) ? 1 : 0.15);
            }});

            svg.on("click", () => {{
                closeInspector();
                node.style("opacity", 1);
                label.style("opacity", 1);
                link.classed("highlighted", false);
            }});

            // Inspector logic
            function showInspector(d) {{
                const insp = document.getElementById('inspector');
                insp.classList.add('active');
                document.getElementById('insp-title').textContent = d.label;
                document.getElementById('insp-file').textContent = d.file.split('/').slice(-2).join('/');
                document.getElementById('insp-line').textContent = d.line;
                document.getElementById('insp-complexity').textContent = d.complexity;
                document.getElementById('insp-fan').textContent = `${{d.fan_in}} / ${{d.fan_out}}`;

                const statusEl = document.getElementById('insp-status');
                statusEl.textContent = d.is_dead ? "Dead Code" : "Reachable / Active";
                statusEl.style.color = d.is_dead ? "var(--color-dead)" : "var(--color-trait)";
            }}

            function closeInspector() {{
                document.getElementById('inspector').classList.remove('active');
            }}

            // Tick loop
            simulation.on("tick", () => {{
                link
                    .attr("x1", d => d.source.x)
                    .attr("y1", d => d.source.y)
                    .attr("x2", d => d.target.x)
                    .attr("y2", d => d.target.y);

                node
                    .attr("cx", d => d.x)
                    .attr("cy", d => d.y);

                label
                    .attr("x", d => d.x)
                    .attr("y", d => d.y);
            }});

            // Search Filter
            const searchInput = document.getElementById('search');
            searchInput.addEventListener('input', (e) => {{
                const query = e.target.value.toLowerCase().trim();
                if (!query) {{
                    node.style("opacity", 1);
                    label.style("opacity", 1);
                    link.style("opacity", 0.6);
                    return;
                }}

                node.style("opacity", d => (d.label.toLowerCase().includes(query) || d.file.toLowerCase().includes(query)) ? 1 : 0.08);
                label.style("opacity", d => (d.label.toLowerCase().includes(query) || d.file.toLowerCase().includes(query)) ? 1 : 0.08);
                link.style("opacity", 0.05);
            }});

            // Zoom Toolbar Controls
            document.getElementById('zoom-in').addEventListener('click', () => svg.transition().duration(300).call(zoom.scaleBy, 1.3));
            document.getElementById('zoom-out').addEventListener('click', () => svg.transition().duration(300).call(zoom.scaleBy, 0.7));
            document.getElementById('zoom-reset').addEventListener('click', () => svg.transition().duration(300).call(zoom.transform, d3.zoomIdentity));

            // Window resize
            window.addEventListener('resize', () => {{
                const w = window.innerWidth;
                const h = window.innerHeight;
                svg.attr("width", w).attr("height", h);
                simulation.force("center", d3.forceCenter(w / 2, h / 2)).restart();
            }});

            // Remove loader
            document.getElementById('loading').style.display = 'none';
        </script>
    </body>
    </html>"###
        )
    }
}
