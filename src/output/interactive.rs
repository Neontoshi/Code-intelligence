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

    fn collect_nodes(call_graph: &CallGraph) -> Vec<serde_json::Value> {
        use crate::analysis::{
            dead_code::is_never_dead,
            roots::{ReachabilityAnalyzer, RootDetectionConfig, RootDetector},
            verdict_source::{VerdictConfig, VerdictEngine},
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

    fn collect_stats(call_graph: &CallGraph, files: &[ParsedFile]) -> serde_json::Value {
        use crate::analysis::{
            dead_code::is_never_dead,
            roots::{ReachabilityAnalyzer, RootDetectionConfig, RootDetector},
            verdict_source::{VerdictConfig, VerdictEngine},
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
        let nodes_json = serde_json::to_string(nodes).unwrap_or_else(|_| "[]".to_string());
        let edges_json = serde_json::to_string(edges).unwrap_or_else(|_| "[]".to_string());
        let stats_json = serde_json::to_string(stats).unwrap_or_else(|_| "{}".to_string());

        format!(
            r###"<!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>{project_name} - Call Graph Architecture</title>
        <link rel="preconnect" href="https://fonts.googleapis.com">
        <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
        <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600;700&family=Plus+Jakarta+Sans:wght@400;500;600;700;800&display=swap" rel="stylesheet">
        <style>
            :root {{
                --bg-canvas: #090a0f;
                --bg-grid: rgba(255, 255, 255, 0.03);
                --panel-bg: rgba(13, 16, 27, 0.78);
                --panel-border: rgba(255, 255, 255, 0.07);
                --panel-border-bright: rgba(56, 189, 248, 0.35);

                --text-primary: #f8fafc;
                --text-secondary: #94a3b8;
                --text-muted: #475569;

                --color-dead: #f43f5e;
                --color-dead-glow: rgba(244, 63, 94, 0.25);
                --color-public: #38bdf8;
                --color-important: #f59e0b;
                --color-trait: #10b981;
                --color-test: #a855f7;
                --color-internal: #64748b;

                --edge-default: rgba(100, 116, 139, 0.25);
                --edge-active: #38bdf8;
                --edge-dead: rgba(244, 63, 94, 0.4);
            }}

            * {{ margin: 0; padding: 0; box-sizing: border-box; }}

            body {{
                font-family: 'Plus Jakarta Sans', sans-serif;
                background: var(--bg-canvas);
                color: var(--text-primary);
                height: 100vh;
                width: 100vw;
                overflow: hidden;
                user-select: none;
                -webkit-font-smoothing: antialiased;
            }}

            #viewport {{
                width: 100vw;
                height: 100vh;
                background-image:
                    radial-gradient(var(--bg-grid) 1px, transparent 1px),
                    linear-gradient(to bottom, rgba(9, 10, 15, 0.4), rgba(9, 10, 15, 0.95));
                background-size: 28px 28px, 100% 100%;
                cursor: grab;
            }}
            #viewport:active {{ cursor: grabbing; }}

            /* Glass Panels */
            .glass {{
                background: var(--panel-bg);
                backdrop-filter: blur(20px);
                -webkit-backdrop-filter: blur(20px);
                border: 1px solid var(--panel-border);
                border-radius: 14px;
                box-shadow: 0 20px 40px -15px rgba(0, 0, 0, 0.7);
            }}

            /* App Header & Top Bar */
            #topbar {{
                position: absolute;
                top: 20px;
                left: 20px;
                width: 380px;
                z-index: 100;
                padding: 16px;
                display: flex;
                flex-direction: column;
                gap: 14px;
            }}

            .brand-row {{
                display: flex;
                align-items: center;
                justify-content: space-between;
            }}
            .brand-title {{
                font-size: 15px;
                font-weight: 800;
                letter-spacing: -0.02em;
                display: flex;
                align-items: center;
                gap: 8px;
                color: #fff;
            }}
            .brand-badge {{
                font-family: 'JetBrains Mono', monospace;
                font-size: 10px;
                padding: 2px 7px;
                border-radius: 6px;
                background: rgba(56, 189, 248, 0.12);
                color: var(--color-public);
                border: 1px solid rgba(56, 189, 248, 0.25);
            }}

            .stats-strip {{
                display: grid;
                grid-template-columns: repeat(4, 1fr);
                gap: 6px;
            }}
            .stat-item {{
                background: rgba(0, 0, 0, 0.35);
                border: 1px solid var(--panel-border);
                border-radius: 8px;
                padding: 6px 8px;
                display: flex;
                flex-direction: column;
            }}
            .stat-item .key {{
                font-size: 9px;
                font-weight: 700;
                text-transform: uppercase;
                letter-spacing: 0.05em;
                color: var(--text-muted);
            }}
            .stat-item .val {{
                font-family: 'JetBrains Mono', monospace;
                font-size: 13px;
                font-weight: 700;
                margin-top: 1px;
            }}

            .search-container {{
                position: relative;
            }}
            #search-input {{
                width: 100%;
                background: rgba(0, 0, 0, 0.4);
                border: 1px solid var(--panel-border);
                border-radius: 8px;
                padding: 9px 12px 9px 34px;
                color: #fff;
                font-size: 12px;
                font-family: 'JetBrains Mono', monospace;
                outline: none;
            }}
            #search-input:focus {{
                border-color: var(--panel-border-bright);
                box-shadow: 0 0 0 2px rgba(56, 189, 248, 0.15);
            }}
            .search-icon {{
                position: absolute;
                left: 11px;
                top: 50%;
                transform: translateY(-50%);
                color: var(--text-muted);
                font-size: 12px;
                pointer-events: none;
            }}

            /* Filter Chips */
            .filter-row {{
                display: flex;
                flex-wrap: wrap;
                gap: 6px;
            }}
            .chip {{
                font-size: 11px;
                font-weight: 600;
                padding: 4px 10px;
                border-radius: 20px;
                background: rgba(255, 255, 255, 0.04);
                border: 1px solid var(--panel-border);
                color: var(--text-secondary);
                cursor: pointer;
                display: flex;
                align-items: center;
                gap: 5px;
            }}
            .chip:hover {{
                background: rgba(255, 255, 255, 0.08);
                color: var(--text-primary);
            }}
            .chip.active {{
                background: rgba(56, 189, 248, 0.15);
                border-color: rgba(56, 189, 248, 0.4);
                color: #fff;
            }}
            .chip-dot {{
                width: 6px;
                height: 6px;
                border-radius: 50%;
            }}

            /* Inspector Sidebar */
            #inspector {{
                position: absolute;
                top: 20px;
                right: 20px;
                width: 360px;
                max-height: calc(100vh - 40px);
                z-index: 100;
                padding: 18px;
                display: none;
                flex-direction: column;
                gap: 14px;
                overflow-y: auto;
            }}
            #inspector.visible {{
                display: flex;
            }}

            .inspector-head {{
                display: flex;
                justify-content: space-between;
                align-items: flex-start;
                border-bottom: 1px solid var(--panel-border);
                padding-bottom: 12px;
            }}
            .func-name {{
                font-family: 'JetBrains Mono', monospace;
                font-size: 15px;
                font-weight: 700;
                word-break: break-all;
                color: var(--color-public);
            }}
            .func-path {{
                font-size: 11px;
                color: var(--text-muted);
                margin-top: 2px;
                word-break: break-all;
            }}
            .btn-close {{
                background: transparent;
                border: none;
                color: var(--text-muted);
                font-size: 18px;
                cursor: pointer;
                padding: 2px 6px;
                border-radius: 6px;
            }}
            .btn-close:hover {{ background: rgba(255,255,255,0.08); color: #fff; }}

            .badge-list {{
                display: flex;
                flex-wrap: wrap;
                gap: 6px;
            }}
            .tag-pill {{
                font-family: 'JetBrains Mono', monospace;
                font-size: 10px;
                font-weight: 600;
                padding: 2px 7px;
                border-radius: 4px;
                background: rgba(255, 255, 255, 0.05);
                border: 1px solid var(--panel-border);
                color: var(--text-secondary);
            }}

            .metric-grid {{
                display: grid;
                grid-template-columns: repeat(2, 1fr);
                gap: 8px;
            }}
            .metric-card {{
                background: rgba(0, 0, 0, 0.3);
                border: 1px solid var(--panel-border);
                border-radius: 8px;
                padding: 8px 10px;
            }}
            .metric-card .m-key {{
                font-size: 10px;
                color: var(--text-muted);
                text-transform: uppercase;
                font-weight: 700;
            }}
            .metric-card .m-val {{
                font-family: 'JetBrains Mono', monospace;
                font-size: 15px;
                font-weight: 700;
                margin-top: 2px;
            }}

            .neighbor-section h4 {{
                font-size: 11px;
                text-transform: uppercase;
                letter-spacing: 0.05em;
                color: var(--text-muted);
                margin-bottom: 6px;
            }}
            .neighbor-list {{
                display: flex;
                flex-direction: column;
                gap: 4px;
                max-height: 140px;
                overflow-y: auto;
            }}
            .neighbor-chip {{
                padding: 6px 8px;
                border-radius: 6px;
                background: rgba(0,0,0,0.25);
                border: 1px solid var(--panel-border);
                font-family: 'JetBrains Mono', monospace;
                font-size: 11px;
                cursor: pointer;
                display: flex;
                justify-content: space-between;
                align-items: center;
            }}
            .neighbor-chip:hover {{
                background: rgba(56, 189, 248, 0.1);
                border-color: var(--panel-border-bright);
                color: #fff;
            }}

            /* Floating HUD Controls */
            #hud {{
                position: absolute;
                bottom: 20px;
                right: 20px;
                z-index: 100;
                display: flex;
                flex-direction: column;
                gap: 8px;
                align-items: flex-end;
            }}
            .hud-btn-group {{
                display: flex;
                gap: 4px;
                padding: 4px;
            }}
            .hud-btn {{
                width: 32px;
                height: 32px;
                border-radius: 8px;
                background: transparent;
                border: none;
                color: var(--text-secondary);
                font-size: 13px;
                font-weight: 700;
                cursor: pointer;
                display: flex;
                align-items: center;
                justify-content: center;
            }}
            .hud-btn:hover {{
                background: rgba(255, 255, 255, 0.1);
                color: #fff;
            }}

            /* Tooltip */
            #tooltip {{
                position: absolute;
                z-index: 1000;
                padding: 8px 12px;
                border-radius: 8px;
                background: rgba(9, 10, 15, 0.95);
                border: 1px solid var(--panel-border-bright);
                font-family: 'JetBrains Mono', monospace;
                font-size: 11px;
                pointer-events: none;
                display: none;
                box-shadow: 0 10px 25px rgba(0,0,0,0.6);
            }}

            /* SVG Styles */
            path.edge-link {{
                fill: none;
                stroke: var(--edge-default);
                stroke-width: 1.2px;
            }}
            path.edge-link.active {{
                stroke: var(--edge-active);
                stroke-width: 2.2px;
            }}

            .node-group {{
                cursor: pointer;
            }}
            .node-circle {{
                stroke-width: 2px;
            }}
            .node-group:hover .node-circle {{
                filter: drop-shadow(0 0 10px currentColor);
            }}
            .node-label {{
                font-family: 'Plus Jakarta Sans', sans-serif;
                font-size: 10px;
                font-weight: 600;
                fill: var(--text-primary);
                text-anchor: middle;
                dominant-baseline: central;
                pointer-events: none;
                text-shadow: 0 1px 3px rgba(0,0,0,0.9), 0 0 8px rgba(0,0,0,0.8);
            }}

            #loading {{
                position: absolute;
                inset: 0;
                background: var(--bg-canvas);
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                z-index: 3000;
                gap: 14px;
            }}
            .ring-spinner {{
                width: 44px;
                height: 44px;
                border: 3px solid rgba(255,255,255,0.08);
                border-top-color: var(--color-public);
                border-radius: 50%;
            }}
        </style>
    </head>
    <body>
        <div id="loading">
            <div class="ring-spinner"></div>
            <div style="font-size: 13px; font-weight: 600; color: var(--text-secondary);">Synthesizing graph models...</div>
        </div>

        <div id="viewport"></div>
        <div id="tooltip"></div>

        <!-- Top Left Dashboard -->
        <div id="topbar" class="glass">
            <div class="brand-row">
                <div class="brand-title">
                    <span>⚡</span> {project_name}
                </div>
                <span class="brand-badge">D3 Engine</span>
            </div>

            <div class="stats-strip">
                <div class="stat-item">
                    <span class="key">Nodes</span>
                    <span class="val" id="stat-fn">0</span>
                </div>
                <div class="stat-item">
                    <span class="key">Calls</span>
                    <span class="val" id="stat-edges">0</span>
                </div>
                <div class="stat-item">
                    <span class="key" style="color: var(--color-dead);">Dead</span>
                    <span class="val" style="color: var(--color-dead);" id="stat-dead">0</span>
                </div>
                <div class="stat-item">
                    <span class="key">Files</span>
                    <span class="val" id="stat-files">0</span>
                </div>
            </div>

            <div class="search-container">
                <span class="search-icon">🔍</span>
                <input type="text" id="search-input" placeholder="Search functions, paths, files..." autocomplete="off" />
            </div>

            <div class="filter-row">
                <button class="chip active" data-filter="all">All</button>
                <button class="chip" data-filter="dead"><span class="chip-dot" style="background: var(--color-dead)"></span> Dead</button>
                <button class="chip" data-filter="public"><span class="chip-dot" style="background: var(--color-public)"></span> Public</button>
                <button class="chip" data-filter="trait"><span class="chip-dot" style="background: var(--color-trait)"></span> Trait</button>
                <button class="chip" data-filter="test"><span class="chip-dot" style="background: var(--color-test)"></span> Tests</button>
            </div>
        </div>

        <!-- Right-Hand Inspector -->
        <div id="inspector" class="glass">
            <div class="inspector-head">
                <div>
                    <div class="func-name" id="insp-name">-</div>
                    <div class="func-path" id="insp-path">-</div>
                </div>
                <button class="btn-close" onclick="closeInspector()">&times;</button>
            </div>

            <div class="badge-list" id="insp-badges"></div>

            <div class="metric-grid">
                <div class="metric-card">
                    <div class="m-key">Cyclomatic Cplx</div>
                    <div class="m-val" id="insp-cplx">-</div>
                </div>
                <div class="metric-card">
                    <div class="m-key">Importance Score</div>
                    <div class="m-val" id="insp-imp">-</div>
                </div>
                <div class="metric-card">
                    <div class="m-key">Fan In (Callers)</div>
                    <div class="m-val" id="insp-fanin">-</div>
                </div>
                <div class="metric-card">
                    <div class="m-key">Fan Out (Callees)</div>
                    <div class="m-val" id="insp-fanout">-</div>
                </div>
            </div>

            <div class="neighbor-section">
                <h4>Incoming Callers</h4>
                <div class="neighbor-list" id="insp-callers"></div>
            </div>

            <div class="neighbor-section">
                <h4>Outgoing Calls</h4>
                <div class="neighbor-list" id="insp-callees"></div>
            </div>
        </div>

        <!-- Floating HUD Actions -->
        <div id="hud">
            <div class="glass hud-btn-group">
                <button class="hud-btn" id="btn-zoom-in" title="Zoom In">+</button>
                <button class="hud-btn" id="btn-zoom-out" title="Zoom Out">&minus;</button>
                <button class="hud-btn" id="btn-zoom-reset" title="Fit to Screen">&#x26F6;</button>
            </div>
        </div>

        <script src="https://d3js.org/d3.v7.min.js"></script>
        <script>
            const nodesData = {nodes_json};
            const edgesData = {edges_json};
            const statsData = {stats_json};

            document.getElementById('stat-fn').textContent = statsData.total_functions;
            document.getElementById('stat-edges').textContent = statsData.total_edges;
            document.getElementById('stat-dead').textContent = statsData.dead_functions;
            document.getElementById('stat-files').textContent = statsData.total_files;

            const width = window.innerWidth;
            const height = window.innerHeight;

            const svg = d3.select("#viewport")
                .append("svg")
                .attr("width", width)
                .attr("height", height);

            const defs = svg.append("defs");
            defs.append("marker")
                .attr("id", "arrow-default")
                .attr("viewBox", "0 -5 10 10")
                .attr("refX", 20)
                .attr("refY", 0)
                .attr("markerWidth", 4)
                .attr("markerHeight", 4)
                .attr("orient", "auto")
                .append("path")
                .attr("d", "M0,-5L10,0L0,5")
                .attr("fill", "rgba(100, 116, 139, 0.4)");

            defs.append("marker")
                .attr("id", "arrow-active")
                .attr("viewBox", "0 -5 10 10")
                .attr("refX", 20)
                .attr("refY", 0)
                .attr("markerWidth", 5)
                .attr("markerHeight", 5)
                .attr("orient", "auto")
                .append("path")
                .attr("d", "M0,-5L10,0L0,5")
                .attr("fill", "#38bdf8");

            const g = svg.append("g");

            const zoom = d3.zoom()
                .scaleExtent([0.15, 3.5])
                .on("zoom", (event) => {{
                    g.attr("transform", event.transform);
                    const k = event.transform.k;
                    g.selectAll(".node-label")
                        .style("opacity", d => (k > 0.8 || d.importance > 0.6 || d.is_dead) ? 1 : 0);
                }});

            svg.call(zoom);

            const nodeById = new Map(nodesData.map(n => [n.id, n]));
            const callersMap = new Map();
            const calleesMap = new Map();

            edgesData.forEach(e => {{
                const s = typeof e.source === 'object' ? e.source.id : e.source;
                const t = typeof e.target === 'object' ? e.target.id : e.target;
                if (!calleesMap.has(s)) calleesMap.set(s, []);
                if (!callersMap.has(t)) callersMap.set(t, []);
                calleesMap.get(s).push(t);
                callersMap.get(t).push(s);
            }});

            const simulation = d3.forceSimulation(nodesData)
                .force("link", d3.forceLink(edgesData).id(d => d.id).distance(90))
                .force("charge", d3.forceManyBody().strength(-300).distanceMax(600))
                .force("center", d3.forceCenter(width / 2, height / 2))
                .force("collision", d3.forceCollide().radius(d => Math.max(10, d.size / 2) + 14));

            const link = g.append("g")
                .selectAll("path")
                .data(edgesData)
                .enter().append("path")
                .attr("class", "edge-link")
                .attr("marker-end", "url(#arrow-default)");

            const node = g.append("g")
                .selectAll("g")
                .data(nodesData)
                .enter().append("g")
                .attr("class", "node-group")
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

            node.append("circle")
                .attr("class", "node-circle")
                .attr("r", d => Math.max(7, d.size / 2))
                .attr("fill", d => d.color)
                .attr("stroke", d => d3.rgb(d.color).darker(0.9))
                .attr("color", d => d.color);

            node.append("text")
                .attr("class", "node-label")
                .attr("dy", d => Math.max(7, d.size / 2) + 12)
                .text(d => d.label);

            const tooltip = document.getElementById("tooltip");

            node.on("mouseover", (event, d) => {{
                tooltip.style.display = "block";
                tooltip.innerHTML = `<strong>${{d.label}}</strong><br><span style="color: #64748b;">${{d.file.split('/').pop()}}:${{d.line}}</span>`;
                tooltip.style.left = (event.pageX + 14) + "px";
                tooltip.style.top = (event.pageY - 12) + "px";
            }}).on("mousemove", (event) => {{
                tooltip.style.left = (event.pageX + 14) + "px";
                tooltip.style.top = (event.pageY - 12) + "px";
            }}).on("mouseout", () => {{
                tooltip.style.display = "none";
            }});

            let selectedNode = null;

            node.on("click", (event, d) => {{
                event.stopPropagation();
                focusNode(d);
            }});

            svg.on("click", () => {{
                resetFocus();
            }});

            function focusNode(d) {{
                selectedNode = d;
                showInspector(d);

                const activeNeighbors = new Set();
                activeNeighbors.add(d.id);

                (callersMap.get(d.id) || []).forEach(id => activeNeighbors.add(id));
                (calleesMap.get(d.id) || []).forEach(id => activeNeighbors.add(id));

                link
                    .classed("active", l => l.source.id === d.id || l.target.id === d.id)
                    .attr("marker-end", l => (l.source.id === d.id || l.target.id === d.id) ? "url(#arrow-active)" : "url(#arrow-default)")
                    .style("opacity", l => (l.source.id === d.id || l.target.id === d.id) ? 1 : 0.08);

                node.style("opacity", n => activeNeighbors.has(n.id) ? 1 : 0.12);
            }}

            function resetFocus() {{
                selectedNode = null;
                closeInspector();
                node.style("opacity", 1);
                link
                    .classed("active", false)
                    .attr("marker-end", "url(#arrow-default)")
                    .style("opacity", 0.6);
            }}

            function showInspector(d) {{
                const insp = document.getElementById('inspector');
                insp.classList.add('visible');

                document.getElementById('insp-name').textContent = d.label;
                document.getElementById('insp-path').textContent = d.full_path;
                document.getElementById('insp-cplx').textContent = d.complexity;
                document.getElementById('insp-imp').textContent = Number(d.importance).toFixed(2);
                document.getElementById('insp-fanin').textContent = d.fan_in;
                document.getElementById('insp-fanout').textContent = d.fan_out;

                const badgeContainer = document.getElementById('insp-badges');
                badgeContainer.innerHTML = '';

                const badges = [
                    d.is_dead ? {{ label: 'Dead Code', bg: 'var(--color-dead)', color: '#fff' }} : null,
                    d.is_public ? {{ label: 'pub', bg: 'rgba(56,189,248,0.2)', color: 'var(--color-public)' }} : null,
                    d.is_async ? {{ label: 'async', bg: 'rgba(168,85,247,0.2)', color: 'var(--color-test)' }} : null,
                    d.is_trait_method ? {{ label: 'trait method', bg: 'rgba(16,185,129,0.2)', color: 'var(--color-trait)' }} : null,
                    d.is_test ? {{ label: 'test', bg: 'rgba(168,85,247,0.2)', color: 'var(--color-test)' }} : null,
                    {{ label: `L${{d.layer}}`, bg: 'rgba(255,255,255,0.06)', color: 'var(--text-secondary)' }},
                ].filter(Boolean);

                badges.forEach(b => {{
                    const el = document.createElement('span');
                    el.className = 'tag-pill';
                    el.textContent = b.label;
                    el.style.background = b.bg;
                    el.style.color = b.color;
                    badgeContainer.appendChild(el);
                }});

                renderNeighborList('insp-callers', callersMap.get(d.id) || []);
                renderNeighborList('insp-callees', calleesMap.get(d.id) || []);
            }}

            function renderNeighborList(elementId, neighborIds) {{
                const listEl = document.getElementById(elementId);
                listEl.innerHTML = '';
                if (neighborIds.length === 0) {{
                    listEl.innerHTML = '<div style="font-size: 11px; color: var(--text-muted); padding: 4px;">None</div>';
                    return;
                }}
                neighborIds.forEach(id => {{
                    const neighbor = nodeById.get(id);
                    if (!neighbor) return;
                    const item = document.createElement('div');
                    item.className = 'neighbor-chip';
                    item.innerHTML = `<span>${{neighbor.label}}</span><span style="color: var(--text-muted); font-size: 9px;">${{neighbor.file.split('/').pop()}}</span>`;
                    item.onclick = (e) => {{
                        e.stopPropagation();
                        focusNode(neighbor);
                    }};
                    listEl.appendChild(item);
                }});
            }}

            function closeInspector() {{
                document.getElementById('inspector').classList.remove('visible');
            }}

            simulation.on("tick", () => {{
                link.attr("d", d => {{
                    const dx = d.target.x - d.source.x;
                    const dy = d.target.y - d.source.y;
                    const dr = Math.sqrt(dx * dx + dy * dy) * 1.5;
                    return `M${{d.source.x}},${{d.source.y}}A${{dr}},${{dr}} 0 0,1 ${{d.target.x}},${{d.target.y}}`;
                }});

                node.attr("transform", d => `translate(${{d.x}},${{d.y}})`);
            }});

            const searchInput = document.getElementById('search-input');
            searchInput.addEventListener('input', (e) => {{
                const query = e.target.value.toLowerCase().trim();
                if (!query) {{
                    resetFocus();
                    return;
                }}
                node.style("opacity", d =>
                    (d.label.toLowerCase().includes(query) || d.file.toLowerCase().includes(query) || d.full_path.toLowerCase().includes(query)) ? 1 : 0.08
                );
                link.style("opacity", 0.04);
            }});

            document.querySelectorAll('.filter-row .chip').forEach(chip => {{
                chip.addEventListener('click', () => {{
                    document.querySelectorAll('.filter-row .chip').forEach(c => c.classList.remove('active'));
                    chip.classList.add('active');

                    const filter = chip.dataset.filter;
                    if (filter === 'all') {{
                        node.style("opacity", 1);
                        link.style("opacity", 0.6);
                        return;
                    }}

                    node.style("opacity", d => {{
                        if (filter === 'dead') return d.is_dead ? 1 : 0.08;
                        if (filter === 'public') return d.is_public ? 1 : 0.08;
                        if (filter === 'trait') return d.is_trait_method ? 1 : 0.08;
                        if (filter === 'test') return d.is_test ? 1 : 0.08;
                        return 1;
                    }});
                    link.style("opacity", 0.05);
                }});
            }});

            document.getElementById('btn-zoom-in').addEventListener('click', () => svg.call(zoom.scaleBy, 1.3));
            document.getElementById('btn-zoom-out').addEventListener('click', () => svg.call(zoom.scaleBy, 0.7));
            document.getElementById('btn-zoom-reset').addEventListener('click', () => {{
                svg.call(zoom.transform, d3.zoomIdentity.translate(0, 0).scale(1));
            }});

            window.addEventListener('resize', () => {{
                const w = window.innerWidth;
                const h = window.innerHeight;
                svg.attr("width", w).attr("height", h);
                simulation.force("center", d3.forceCenter(w / 2, h / 2)).restart();
            }});

            document.getElementById('loading').style.display = 'none';
        </script>
    </body>
    </html>"###
        )
    }
}
