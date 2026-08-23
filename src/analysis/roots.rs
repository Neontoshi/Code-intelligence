// src/analysis/roots.rs

use crate::graph::call_graph::CallGraph;
use crate::parser::tree_sitter::ParsedFile;
use crate::FunctionNode;
use std::collections::{HashMap, HashSet};

pub type FunctionId = String;

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

    /// Detect application entry points using contextual evidence
    fn detect_application_roots(call_graph: &CallGraph) -> HashSet<FunctionId> {
        let mut roots = HashSet::new();

        // ⭐ Only treat as entry point if there's supporting context
        let app_entry_names = ["main", "async_main", "run", "start", "init", "setup"];

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];

            // Must be exactly "main" or "async_main"
            if func.name == "main" || func.name == "async_main" {
                roots.insert(func.full_path.clone());
                continue;
            }

            // For generic names like "run", "start", "init", "setup"
            if app_entry_names.contains(&func.name.as_str()) {
                // Only treat as root if there's supporting evidence:
                // 1. It's the only function with that name in the project
                // 2. It has no callers (true entry point)
                // 3. It's in a bin/ or main.rs file
                // 4. It has #[tokio::main] or similar attribute

                let is_entry = Self::is_likely_entry_point(func, call_graph);
                if is_entry {
                    roots.insert(func.full_path.clone());
                }
            }
        }

        roots
    }

    ///Check if a function is likely a true entry point
    fn is_likely_entry_point(func: &FunctionNode, call_graph: &CallGraph) -> bool {
        // Check 1: No callers (true entry point)
        let idx = call_graph.name_index.get(&func.full_path);
        if let Some(&idx) = idx {
            let callers = call_graph.get_callers(idx);
            if callers.is_empty() {
                // Check 2: In bin/ or main.rs file
                if func.file.contains("/bin/")
                    || func.file.ends_with("main.rs")
                    || func.file.contains("/src/bin/")
                {
                    return true;
                }

                // Check 3: Has async attribute (likely tokio::main)
                if func.is_async {
                    return true;
                }

                // Check 4: Has doc comment with entry point indicators
                if let Some(doc) = &func.doc_comment {
                    if doc.contains("#[tokio::main]")
                        || doc.contains("#[async_std::main]")
                        || doc.contains("entry point")
                    {
                        return true;
                    }
                }
            }
        }

        false
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

    // Update the detect_export_roots function
    fn detect_export_roots(call_graph: &CallGraph) -> HashSet<FunctionId> {
        let mut roots = HashSet::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];

            // React components are roots (even if not public)
            if func.file.ends_with(".tsx") || func.file.ends_with(".jsx") {
                let is_component = func
                    .name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false);
                let is_hook = func.name.starts_with("use");
                if is_component || is_hook {
                    roots.insert(func.full_path.clone());
                    continue;
                }
            }

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

    // Update the detect_ffi_roots function
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

            // 3. Check for extern "C" in doc comment
            if let Some(doc) = &func.doc_comment {
                if doc.contains("extern \"C\"")
                    || doc.contains("#[no_mangle]")
                    || doc.contains("#[export_name]")
                    || doc.contains("#[link_name]")
                {
                    roots.insert(func.full_path.clone());
                }
            }

            // 4. Check for #[no_mangle] attribute via function name pattern
            if func.name.starts_with("_") && func.name.contains("c_") {
                roots.insert(func.full_path.clone());
            }
        }

        roots
    }

    fn detect_framework_roots(call_graph: &CallGraph, files: &[ParsedFile]) -> HashSet<FunctionId> {
        let mut roots = HashSet::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];

            // ⭐ React components (TSX/JSX) - contextual detection
            if func.file.ends_with(".tsx") || func.file.ends_with(".jsx") {
                // Must be in components/ or pages/ directory OR exported
                let is_component = func
                    .name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false);
                let is_hook = func.name.starts_with("use");
                let is_exported = func.is_public;

                // ⭐ NEW: Check if in a component directory
                let in_component_dir = func.file.contains("/components/")
                    || func.file.contains("/pages/")
                    || func.file.contains("/hooks/");

                if (is_component || is_hook) && (in_component_dir || is_exported) {
                    roots.insert(func.full_path.clone());
                    continue;
                }
            }

            // Go init functions - only in main packages or with init pattern
            if func.name == "init" && func.file.ends_with(".go") {
                // Check if it's in a main package or has no callers
                let idx = call_graph.name_index.get(&func.full_path);
                if let Some(&idx) = idx {
                    let callers = call_graph.get_callers(idx);
                    if callers.is_empty() {
                        roots.insert(func.full_path.clone());
                    }
                    continue;
                }
            }

            // Java/Spring - check function annotations
            if func.file.ends_with(".java") {
                if let Some(file) = files.iter().find(|f| f.path == func.file) {
                    if let Some(func_info) = file.functions.iter().find(|fi| fi.name == func.name) {
                        for decorator in &func_info.decorators {
                            let decorator_lower = decorator.to_lowercase();
                            if decorator_lower.contains("getmapping")
                                || decorator_lower.contains("postmapping")
                                || decorator_lower.contains("putmapping")
                                || decorator_lower.contains("deletemapping")
                                || decorator_lower.contains("requestmapping")
                                || decorator_lower.contains("restcontroller")
                                || decorator_lower.contains("controller")
                                || decorator_lower.contains("service")
                                || decorator_lower.contains("repository")
                                || decorator_lower.contains("component")
                            {
                                roots.insert(func.full_path.clone());
                                break;
                            }
                        }
                    }
                }
            }

            // Python Flask/FastAPI - check decorators
            if let Some(file) = files.iter().find(|f| f.path == func.file) {
                if let Some(func_info) = file.functions.iter().find(|fi| fi.name == func.name) {
                    for decorator in &func_info.decorators {
                        let decorator_lower = decorator.to_lowercase();
                        if decorator_lower.contains("app.route")
                            || decorator_lower.contains("router.")
                            || decorator_lower.contains("blueprint.")
                            || decorator_lower.contains("click.command")
                            || decorator_lower.contains("pytest")
                            || decorator_lower.contains("app.get")
                            || decorator_lower.contains("app.post")
                            || decorator_lower.contains("app.put")
                            || decorator_lower.contains("app.delete")
                        {
                            roots.insert(func.full_path.clone());
                            break;
                        }
                    }
                }
            }
        }

        roots
    }
}

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

        let path_to_idx: HashMap<&str, petgraph::graph::NodeIndex> = call_graph
            .node_indices()
            .map(|idx| (call_graph[idx].full_path.as_str(), idx))
            .collect();

        let mut processed = HashSet::new();

        while let Some((current, root)) = queue.pop_front() {
            if !processed.insert(current.clone()) {
                continue;
            }

            if let Some(&idx) = path_to_idx.get(current.as_str()) {
                for callee in call_graph.get_callees(idx) {
                    let callee_path = &callee.full_path;
                    if reachable.insert(callee_path.clone()) {
                        reachable_from
                            .entry(callee_path.clone())
                            .or_default()
                            .push(root.clone());
                        queue.push_back((callee_path.clone(), root.clone()));
                    } else {
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
