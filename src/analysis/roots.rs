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

        let app_entry_names = ["main", "async_main", "run", "start", "init", "setup"];

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];

            // Must be exactly "main", "async_main", or Flutter's lib/main.dart
            if func.name == "main" || func.name == "async_main" {
                roots.insert(func.full_path.clone());
                continue;
            }

            // PHP entry points (index.php, artisan, bin/console)
            if func.file.ends_with("index.php") || func.file.ends_with("artisan") {
                roots.insert(func.full_path.clone());
                continue;
            }

            // C# top-level entry files (Program.cs, Startup.cs)
            if func.file.ends_with("Program.cs") || func.file.ends_with("Startup.cs") {
                roots.insert(func.full_path.clone());
                continue;
            }

            // For generic names like "run", "start", "init", "setup"
            if app_entry_names.contains(&func.name.as_str()) {
                let is_entry = Self::is_likely_entry_point(func, call_graph);
                if is_entry {
                    roots.insert(func.full_path.clone());
                }
            }
        }

        roots
    }

    fn is_react_hook_name(name: &str) -> bool {
        if !name.starts_with("use") || name.len() <= 3 {
            return false;
        }
        name.chars()
            .nth(3)
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
    }

    /// Check if a function is likely a true entry point
    fn is_likely_entry_point(func: &FunctionNode, call_graph: &CallGraph) -> bool {
        let idx = call_graph.name_index.get(&func.full_path);
        if let Some(&idx) = idx {
            let callers = call_graph.get_callers(idx);
            if callers.is_empty() {
                if func.file.contains("/bin/")
                    || func.file.ends_with("main.rs")
                    || func.file.contains("/src/bin/")
                {
                    return true;
                }

                if func.is_async {
                    return true;
                }

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
                || func.name.starts_with("benchmark_")
                || func.name.starts_with("Benchmark")
                || func.file.contains("/tests/")
                || func.file.contains("/benches/")
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

            let is_jsx_file = func.file.ends_with(".tsx") || func.file.ends_with(".jsx");
            let is_ts_family =
                is_jsx_file || func.file.ends_with(".ts") || func.file.ends_with(".js");

            let is_component = is_jsx_file
                && func
                    .name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false);
            let is_hook = is_ts_family && Self::is_react_hook_name(&func.name);

            if is_component || is_hook {
                roots.insert(func.full_path.clone());
                continue;
            }

            let is_entry_barrel = func.file.ends_with("/index.ts")
                || func.file.ends_with("/index.js")
                || func.file.ends_with("/index.tsx")
                || func.file.ends_with("/index.jsx")
                || func.file.ends_with("/main.ts")
                || func.file.ends_with("/mod.ts")
                || func.file.ends_with("/lib.ts");

            if func.is_public && (is_entry_barrel || func.fan_in == 0) {
                roots.insert(func.full_path.clone());
            }

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

    fn detect_ffi_roots(call_graph: &CallGraph) -> HashSet<FunctionId> {
        let mut roots = HashSet::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];

            if func.name.contains("extern") || func.name.contains("ffi") {
                roots.insert(func.full_path.clone());
            }

            if func.file.contains("/ffi/") || func.file.contains("/extern/") {
                roots.insert(func.full_path.clone());
            }

            if let Some(doc) = &func.doc_comment {
                if doc.contains("extern \"C\"")
                    || doc.contains("#[no_mangle]")
                    || doc.contains("#[export_name]")
                    || doc.contains("#[link_name]")
                {
                    roots.insert(func.full_path.clone());
                }
            }

            if func.name.starts_with('_') && func.name.contains("c_") {
                roots.insert(func.full_path.clone());
            }
        }

        roots
    }

    fn detect_framework_roots(call_graph: &CallGraph, files: &[ParsedFile]) -> HashSet<FunctionId> {
        let mut roots = HashSet::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];

            let is_jsx_file = func.file.ends_with(".tsx") || func.file.ends_with(".jsx");
            let is_ts_family =
                is_jsx_file || func.file.ends_with(".ts") || func.file.ends_with(".js");

            if is_ts_family {
                let is_component = is_jsx_file
                    && (func
                        .name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                        || func.file.contains("/pages/")
                        || func.file.contains("/components/"));
                let is_hook = Self::is_react_hook_name(&func.name);

                let in_framework_dir = func.file.contains("/component")
                    || func.file.contains("/page")
                    || func.file.contains("/hooks/")
                    || func.file.contains("/stores/")
                    || func.file.contains("/services/");

                if is_hook || is_component || (func.is_public && in_framework_dir) {
                    roots.insert(func.full_path.clone());
                    continue;
                }
            }

            // Java/Spring
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

            // Python
            if func.file.ends_with(".py") {
                if func.file.ends_with("__main__.py")
                    || func.file.ends_with("manage.py")
                    || func.file.ends_with("wsgi.py")
                {
                    roots.insert(func.full_path.clone());
                    continue;
                }

                if let Some(file) = files.iter().find(|f| f.path == func.file) {
                    if let Some(func_info) = file.functions.iter().find(|fi| fi.name == func.name) {
                        for decorator in &func_info.decorators {
                            let d = decorator.to_lowercase();
                            if d.contains("route")
                                || d.contains("get")
                                || d.contains("post")
                                || d.contains("put")
                                || d.contains("delete")
                                || d.contains("patch")
                                || d.contains("router.")
                                || d.contains("blueprint.")
                                || d.contains("command")
                                || d.contains("fixture")
                                || d.contains("pytest")
                                || d.contains("task")
                                || d.contains("celery")
                            {
                                roots.insert(func.full_path.clone());
                                break;
                            }
                        }
                    }
                }
            }

            // Dart / Flutter
            if func.file.ends_with(".dart") {
                let is_widget = func.file.contains("/widgets/")
                    || func.file.contains("/pages/")
                    || func.file.contains("/screens/")
                    || func.file.contains("/views/");

                if is_widget && func.is_public {
                    roots.insert(func.full_path.clone());
                    continue;
                }
            }
            // C# / ASP.NET Core Attributes & MediatR Handlers
            if func.file.ends_with(".cs") {
                if let Some(file) = files.iter().find(|f| f.path == func.file) {
                    if let Some(func_info) = file.functions.iter().find(|fi| fi.name == func.name) {
                        for decorator in &func_info.decorators {
                            let d = decorator.to_lowercase();
                            if d.contains("httpget")
                                            || d.contains("httppost")
                                            || d.contains("httpput")
                                            || d.contains("httpdelete")
                                            || d.contains("route")
                                            || d.contains("apicontroller")
                                            || d.contains("authorize")
                                            || d.contains("fact")      // xUnit test
                                            || d.contains("test")
                            // NUnit test
                            {
                                roots.insert(func.full_path.clone());
                                break;
                            }
                        }
                    }
                }
            }

            // PHP Laravel / Symfony
            if func.file.ends_with(".php") {
                if let Some(file) = files.iter().find(|f| f.path == func.file) {
                    if let Some(func_info) = file.functions.iter().find(|fi| fi.name == func.name) {
                        for decorator in &func_info.decorators {
                            let d = decorator.to_lowercase();
                            if d.contains("route")
                                || d.contains("get")
                                || d.contains("post")
                                || d.contains("livewire")
                            {
                                roots.insert(func.full_path.clone());
                                break;
                            }
                        }
                    }
                }
            }

            // C++
            if func.file.ends_with(".cpp")
                || func.file.ends_with(".hpp")
                || func.file.ends_with(".h")
            {
                if let Some(doc) = &func.doc_comment {
                    if doc.contains("Q_INVOKABLE")
                        || doc.contains("EMSCRIPTEN_KEEPALIVE")
                        || doc.contains("JNIEXPORT")
                    {
                        roots.insert(func.full_path.clone());
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

    pub fn find_reachable_functions(
        call_graph: &CallGraph,
        roots: &RootSet,
    ) -> HashSet<FunctionId> {
        let map = Self::compute_reachability(call_graph, roots);
        map.reachable
    }
}
