// src/analysis/roots.rs

use crate::graph::call_graph::CallGraph;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::{HashMap, HashSet};

/// A unique identifier for a function
pub type FunctionId = String;

/// Set of root functions categorized by type
#[derive(Debug, Clone, Default)]
pub struct RootSet {
    /// Application entry points (main, run, start, etc.)
    pub application: HashSet<FunctionId>,

    /// Test functions (test_*, bench_*, etc.)
    pub tests: HashSet<FunctionId>,

    /// Public API exports (pub functions in libs)
    pub exports: HashSet<FunctionId>,

    /// Framework callbacks (React components, Spring handlers, etc.)
    pub framework: HashSet<FunctionId>,

    /// FFI exports (no_mangle, extern, etc.)
    pub ffi: HashSet<FunctionId>,
}

impl RootSet {
    /// Get all roots as a single set
    pub fn all(&self) -> HashSet<FunctionId> {
        let mut all = HashSet::new();
        all.extend(self.application.iter().cloned());
        all.extend(self.tests.iter().cloned());
        all.extend(self.exports.iter().cloned());
        all.extend(self.framework.iter().cloned());
        all.extend(self.ffi.iter().cloned());
        all
    }

    /// Check if a function is a root
    pub fn is_root(&self, id: &FunctionId) -> bool {
        self.all().contains(id)
    }

    /// Get counts by category
    pub fn counts(&self) -> Vec<(&'static str, usize)> {
        vec![
            ("Application", self.application.len()),
            ("Tests", self.tests.len()),
            ("Exports", self.exports.len()),
            ("Framework", self.framework.len()),
            ("FFI", self.ffi.len()),
        ]
    }
}

// Root Detector

#[derive(Debug, Clone)]
pub struct RootDetectionConfig {
    /// Include public functions as roots (for library analysis)
    pub include_exports: bool,

    /// Include test functions as roots
    pub include_tests: bool,

    /// Include framework callbacks (React, Spring, etc.)
    pub include_framework: bool,

    /// Include FFI exports
    pub include_ffi: bool,
}

impl Default for RootDetectionConfig {
    fn default() -> Self {
        Self {
            include_exports: true,
            include_tests: true,
            include_framework: true,
            include_ffi: true,
        }
    }
}

pub struct RootDetector;

impl RootDetector {
    /// Detect all roots in the project
    pub fn detect_roots(
        call_graph: &CallGraph,
        files: &[ParsedFile],
        config: &RootDetectionConfig,
    ) -> RootSet {
        let mut roots = RootSet::default();

        // 1. Application entry points
        roots.application = Self::detect_application_roots(call_graph);

        // 2. Test functions
        if config.include_tests {
            roots.tests = Self::detect_test_roots(call_graph);
        }

        // 3. Public exports (library API)
        if config.include_exports {
            roots.exports = Self::detect_export_roots(call_graph);
        }

        // 4. Framework callbacks
        if config.include_framework {
            roots.framework = Self::detect_framework_roots(call_graph, files);
        }

        // 5. FFI exports
        if config.include_ffi {
            roots.ffi = Self::detect_ffi_roots(call_graph);
        }

        roots
    }

    // Detection Methods

    /// Detect application entry points
    fn detect_application_roots(call_graph: &CallGraph) -> HashSet<FunctionId> {
        let mut roots = HashSet::new();

        let app_entry_names = ["main", "async_main", "run", "start", "init", "setup"];

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            if app_entry_names.contains(&func.name.as_str()) {
                roots.insert(func.full_path.clone());
            }
        }

        roots
    }

    /// Detect test functions
    fn detect_test_roots(call_graph: &CallGraph) -> HashSet<FunctionId> {
        let mut roots = HashSet::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            let is_test = func.name.starts_with("test_")
                || func.name.starts_with("Test")
                || func.name.starts_with("bench_")
                || func.name.starts_with("Benchmark")
                || func.file.contains("/tests/")
                || func.file.ends_with("_test.rs")
                || func.file.ends_with("_test.go");

            if is_test {
                roots.insert(func.full_path.clone());
            }
        }

        roots
    }

    fn detect_export_roots(call_graph: &CallGraph) -> HashSet<FunctionId> {
        let mut roots = HashSet::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];

            // Public functions with no callers are likely API exports
            if func.is_public && func.fan_in == 0 {
                roots.insert(func.full_path.clone());
            }

            // Go exported functions (capitalized) are EXPORTS, not FFI
            if func.file.ends_with(".go") {
                let is_exported = func
                    .name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false);
                if is_exported {
                    roots.insert(func.full_path.clone());
                }
            }
        }

        roots
    }

    /// Detect framework-specific roots
    fn detect_framework_roots(call_graph: &CallGraph, files: &[ParsedFile]) -> HashSet<FunctionId> {
        let mut roots = HashSet::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];

            // React components (TSX/JSX) - function-level detection
            if func.file.ends_with(".tsx") || func.file.ends_with(".jsx") {
                let is_component = func
                    .name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false);
                let is_hook = func.name.starts_with("use");
                let is_export = func.is_public;

                if is_component || is_hook || is_export {
                    roots.insert(func.full_path.clone());
                }
            }

            // Go init functions
            if func.name == "init" && func.file.ends_with(".go") {
                roots.insert(func.full_path.clone());
            }

            // Java/Spring - check function annotations, not just file path
            if func.file.contains("/controllers/")
                || func.file.contains("/handlers/")
                || func.file.contains("/services/")
            {
                let is_spring_handler = func.name.contains("handle")
                    || func.name.contains("get")
                    || func.name.contains("post")
                    || func.name.contains("put")
                    || func.name.contains("delete");

                let has_spring_annotation = func
                    .doc_comment
                    .as_ref()
                    .map(|d| {
                        d.contains("@GetMapping")
                            || d.contains("@PostMapping")
                            || d.contains("@RequestMapping")
                            || d.contains("@RestController")
                    })
                    .unwrap_or(false);

                if func.is_public && (is_spring_handler || has_spring_annotation) {
                    roots.insert(func.full_path.clone());
                }
            }

            // Python decorators - use the stored decorators from parser
            if let Some(file) = files.iter().find(|f| f.path == func.file) {
                if let Some(func_info) = file.functions.iter().find(|fi| fi.name == func.name) {
                    for decorator in &func_info.decorators {
                        if decorator.contains("app.route")
                            || decorator.contains("router.")
                            || decorator.contains("blueprint.")
                            || decorator.contains("click.command")
                            || decorator.contains("pytest")
                            || decorator.contains("app.get")
                            || decorator.contains("app.post")
                            || decorator.contains("app.put")
                            || decorator.contains("app.delete")
                        {
                            roots.insert(func.full_path.clone());
                            break;
                        }
                    }
                }
            }

            // ⭐ Java Spring annotations - use decorators from parser
            if func.file.ends_with(".java") {
                if let Some(file) = files.iter().find(|f| f.path == func.file) {
                    if let Some(func_info) = file.functions.iter().find(|fi| fi.name == func.name) {
                        for decorator in &func_info.decorators {
                            if decorator.contains("GetMapping")
                                || decorator.contains("PostMapping")
                                || decorator.contains("PutMapping")
                                || decorator.contains("DeleteMapping")
                                || decorator.contains("RequestMapping")
                                || decorator.contains("RestController")
                                || decorator.contains("Service")
                                || decorator.contains("Repository")
                                || decorator.contains("Component")
                            {
                                roots.insert(func.full_path.clone());
                                break;
                            }
                        }
                    }
                }
            }
        }

        roots
    }

    fn detect_ffi_roots(call_graph: &CallGraph) -> HashSet<FunctionId> {
        let mut roots = HashSet::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];

            // 1. Function name suggests FFI
            if func.name.contains("extern") || func.name.contains("ffi") {
                roots.insert(func.full_path.clone());
            }

            // 2. File path suggests FFI
            if func.file.contains("/ffi/") || func.file.contains("/extern/") {
                roots.insert(func.full_path.clone());
            }

            // 3. Check for extern "C" in the doc comment (not reliable but better than nothing)
            if let Some(doc) = &func.doc_comment {
                if doc.contains("extern \"C\"")
                    || doc.contains("#[no_mangle]")
                    || doc.contains("#[export_name]")
                {
                    if func.name.contains("extern") || func.file.contains("/ffi/") {
                        roots.insert(func.full_path.clone());
                    }
                }
            }
        }

        roots
    }
}

// Reachability Analysis

#[derive(Debug, Clone)]
pub struct ReachabilityMap {
    /// Functions reachable from roots
    pub reachable: HashSet<FunctionId>,
    /// Functions NOT reachable from roots
    pub unreachable: HashSet<FunctionId>,
    /// For each reachable function, the roots that reach it
    pub reachable_from: HashMap<FunctionId, Vec<FunctionId>>,
}

impl ReachabilityMap {
    pub fn is_reachable(&self, id: &FunctionId) -> bool {
        self.reachable.contains(id)
    }

    pub fn is_unreachable(&self, id: &FunctionId) -> bool {
        self.unreachable.contains(id)
    }

    pub fn reachable_count(&self) -> usize {
        self.reachable.len()
    }

    pub fn unreachable_count(&self) -> usize {
        self.unreachable.len()
    }
}

pub struct ReachabilityAnalyzer;

impl ReachabilityAnalyzer {
    /// Compute reachability from roots
    pub fn compute_reachability(call_graph: &CallGraph, roots: &RootSet) -> ReachabilityMap {
        use std::collections::{HashMap, HashSet, VecDeque};

        let root_ids = roots.all();
        let mut reachable = HashSet::new();
        let mut reachable_from: HashMap<FunctionId, Vec<FunctionId>> = HashMap::new();
        let mut queue = VecDeque::new();

        // Initialize with all roots
        for root_id in &root_ids {
            if !reachable.contains(root_id) {
                reachable.insert(root_id.clone());
                reachable_from
                    .entry(root_id.clone())
                    .or_default()
                    .push(root_id.clone());
                queue.push_back((root_id.clone(), root_id.clone()));
            }
        }

        // Build a fast lookup for function full_path -> NodeIndex
        let path_to_idx: HashMap<String, petgraph::graph::NodeIndex> = call_graph
            .node_indices()
            .map(|idx| (call_graph[idx].full_path.clone(), idx))
            .collect();

        // BFS traversal - safe from stack overflow
        let mut processed = HashSet::new();
        let max_functions = 10000;

        while let Some((current, root)) = queue.pop_front() {
            // Safety limit to prevent infinite loops
            if reachable.len() > max_functions {
                eprintln!(
                    "⚠️ Reachability analysis reached safety limit ({} functions)",
                    max_functions
                );
                break;
            }

            // Skip if already processed
            if processed.contains(&current) {
                continue;
            }
            processed.insert(current.clone());

            // Get the node index for this function
            if let Some(&idx) = path_to_idx.get(&current) {
                // Get all callees
                for callee in call_graph.get_callees(idx) {
                    let callee_path = &callee.full_path;
                    if !reachable.contains(callee_path) {
                        reachable.insert(callee_path.clone());
                        reachable_from
                            .entry(callee_path.clone())
                            .or_default()
                            .push(root.clone());
                        queue.push_back((callee_path.clone(), root.clone()));
                    } else {
                        // Already reachable, but add this root as another source
                        reachable_from
                            .entry(callee_path.clone())
                            .or_default()
                            .push(root.clone());
                    }
                }
            }
        }

        // Compute unreachable
        let mut unreachable = HashSet::new();
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            if !reachable.contains(&func.full_path) {
                unreachable.insert(func.full_path.clone());
            }
        }

        eprintln!(
            "📊 Reachability: {} reachable, {} unreachable",
            reachable.len(),
            unreachable.len()
        );

        ReachabilityMap {
            reachable,
            unreachable,
            reachable_from,
        }
    }

    /// Legacy method - kept for compatibility
    pub fn find_reachable_functions(
        call_graph: &CallGraph,
        roots: &RootSet,
    ) -> HashSet<FunctionId> {
        let map = Self::compute_reachability(call_graph, roots);
        map.reachable
    }
}
