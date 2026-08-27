// src/analysis/roots/root_set.rs

//! Root set and reachability analysis

use crate::analysis::roots::FunctionId;
use crate::graph::call_graph::CallGraph;
use std::collections::{HashMap, HashSet, VecDeque};

/// Configuration for root detection
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

/// Set of root functions
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
    pub fn add_application(&mut self, id: FunctionId) {
        self.application.insert(id);
    }

    pub fn add_test(&mut self, id: FunctionId) {
        self.tests.insert(id);
    }

    pub fn add_export(&mut self, id: FunctionId) {
        self.exports.insert(id);
    }

    pub fn add_framework(&mut self, id: FunctionId) {
        self.framework.insert(id);
    }

    pub fn add_ffi(&mut self, id: FunctionId) {
        self.ffi.insert(id);
    }

    pub fn add_many_application(&mut self, ids: HashSet<FunctionId>) {
        self.application.extend(ids);
    }

    pub fn add_many_tests(&mut self, ids: HashSet<FunctionId>) {
        self.tests.extend(ids);
    }

    pub fn add_many_exports(&mut self, ids: HashSet<FunctionId>) {
        self.exports.extend(ids);
    }

    pub fn add_many_framework(&mut self, ids: HashSet<FunctionId>) {
        self.framework.extend(ids);
    }

    pub fn add_many_ffi(&mut self, ids: HashSet<FunctionId>) {
        self.ffi.extend(ids);
    }

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

/// Main root detector
pub struct RootDetector;

impl RootDetector {
    /// Detect all roots in the project
    pub fn detect_roots(
        call_graph: &CallGraph,
        files: &[crate::parser::tree_sitter::ParsedFile],
        config: &RootDetectionConfig,
    ) -> RootSet {
        use super::RootDetectorOrchestrator;
        let orchestrator = RootDetectorOrchestrator::new();
        orchestrator.detect_all_roots(call_graph, files, config)
    }
}

/// Reachability map
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

/// Reachability analyzer
pub struct ReachabilityAnalyzer;

impl ReachabilityAnalyzer {
    /// Compute reachability from roots
    pub fn compute_reachability(call_graph: &CallGraph, roots: &RootSet) -> ReachabilityMap {
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
