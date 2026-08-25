// src/engine/incremental.rs

use crate::graph::call_graph::CallGraph;
use crate::graph::traits::GraphMetrics;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

/// Tracks file changes for incremental analysis
#[derive(Debug, Clone)]
pub struct FileTracker {
    /// File path -> (modified_time, content_hash)
    files: HashMap<PathBuf, (SystemTime, String)>,
    /// Functions that were previously analyzed
    function_cache: HashMap<String, IncrementalFunction>,
    /// Reverse dependency map: function -> list of functions that call it
    reverse_deps: HashMap<String, HashSet<String>>,
    /// Forward dependency map: function -> list of functions it calls
    forward_deps: HashMap<String, HashSet<String>>,
}

#[derive(Debug, Clone)]
pub struct IncrementalFunction {
    pub full_path: String,
    pub name: String,
    pub file: PathBuf,
    pub line: usize,
    pub fan_in: usize,
    pub fan_out: usize,
    pub complexity: f64,
    pub is_public: bool,
    pub is_async: bool,
    pub trait_impl: Option<String>,
    pub calls: Vec<String>,
    pub callers: Vec<String>,
    pub dirty: bool,
}

#[derive(Debug, Clone, Default)]
pub struct IncrementalResult {
    pub changed_files: Vec<PathBuf>,
    pub affected_functions: Vec<String>,
    pub removed_functions: Vec<String>,
    pub added_functions: Vec<String>,
    pub modified_functions: Vec<String>,
    pub cache_hit: bool,
    /// Rebuild instructions for the pipeline
    pub rebuild_scope: RebuildScope,
}

/// Defines what needs to be rebuilt
#[derive(Debug, Clone, Default)]
pub struct RebuildScope {
    /// Functions that need full re-analysis
    pub functions_to_reanalyze: HashSet<String>,
    /// Files that need re-parsing
    pub files_to_reparse: HashSet<PathBuf>,
    /// Edges that need to be re-evaluated
    pub edges_to_rebuild: HashSet<(String, String)>,
    /// Whether the entire graph needs rebuilding
    pub full_rebuild: bool,
}

impl FileTracker {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            function_cache: HashMap::new(),
            reverse_deps: HashMap::new(),
            forward_deps: HashMap::new(),
        }
    }

    pub fn detect_changes(&mut self, files: &[ParsedFile]) -> Vec<PathBuf> {
        let mut changed = Vec::new();
        for file in files {
            let path = PathBuf::from(&file.path);
            let hash = self.hash_file(&file.source);
            if let Some((_, old_hash)) = self.files.get(&path) {
                if old_hash != &hash {
                    changed.push(path.clone());
                    // Track that this file changed for dependency invalidation
                    self.track_dependency_change(&path);
                }
            } else {
                changed.push(path.clone());
                self.track_dependency_change(&path);
            }
            self.files.insert(path, (SystemTime::now(), hash));
        }
        changed
    }

    /// Track dependency changes - invalidate callers of changed functions
    fn track_dependency_change(&mut self, changed_file: &Path) {
        // Find all functions in this file
        let changed_funcs: Vec<String> = self
            .function_cache
            .iter()
            .filter(|(_, f)| PathBuf::from(&f.file) == changed_file)
            .map(|(path, _)| path.clone())
            .collect();

        // Mark them as dirty
        for func_path in changed_funcs {
            if let Some(cached) = self.function_cache.get_mut(&func_path) {
                cached.dirty = true;
            }
        }
    }

    /// Build dependency graph from call graph
    pub fn build_dependency_graph(&mut self, call_graph: &CallGraph) {
        self.reverse_deps.clear();
        self.forward_deps.clear();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            let full_path = func.full_path.clone();

            // Forward dependencies: functions this function calls
            let callees: HashSet<String> = call_graph
                .get_callees(idx)
                .iter()
                .map(|f| f.full_path.clone())
                .collect();

            self.forward_deps.insert(full_path.clone(), callees.clone());

            // Reverse dependencies: functions that call this function
            for callee in callees {
                self.reverse_deps
                    .entry(callee)
                    .or_default()
                    .insert(full_path.clone());
            }
        }
    }

    /// Get all functions affected by changes (full dependency propagation)
    pub fn get_affected_functions_full(
        &self,
        changed_files: &[PathBuf],
        call_graph: &CallGraph,
    ) -> HashSet<String> {
        let mut affected = HashSet::new();

        // 1. Direct changes: functions in changed files
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            let file_path = PathBuf::from(&func.file);
            if changed_files.contains(&file_path) {
                affected.insert(func.full_path.clone());
            }
        }

        // 2. Propagate through dependency graph (both directions)
        let mut queue: VecDeque<String> = affected.iter().cloned().collect();
        let mut processed = HashSet::new();

        while let Some(path) = queue.pop_front() {
            if processed.contains(&path) {
                continue;
            }
            processed.insert(path.clone());

            // Forward: functions that depend on this one (callers)
            if let Some(callers) = self.reverse_deps.get(&path) {
                for caller in callers {
                    if !affected.contains(caller) {
                        affected.insert(caller.clone());
                        queue.push_back(caller.clone());
                    }
                }
            }

            // Backward: functions that this one depends on (callees)
            if let Some(callees) = self.forward_deps.get(&path) {
                for callee in callees {
                    if !affected.contains(callee) {
                        affected.insert(callee.clone());
                        queue.push_back(callee.clone());
                    }
                }
            }
        }

        affected
    }

    /// Determine rebuild scope
    pub fn determine_rebuild_scope(
        &self,
        changed_files: &[PathBuf],
        call_graph: &CallGraph,
    ) -> RebuildScope {
        let mut scope = RebuildScope::default();

        // If too many files changed, do full rebuild
        if changed_files.len() > 10
            || changed_files.len() as f64 / call_graph.node_count() as f64 > 0.3
        {
            scope.full_rebuild = true;
            return scope;
        }

        // Get affected functions
        let affected = self.get_affected_functions_full(changed_files, call_graph);
        scope.functions_to_reanalyze = affected;

        // Find files that need re-parsing
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            if scope.functions_to_reanalyze.contains(&func.full_path) {
                scope.files_to_reparse.insert(PathBuf::from(&func.file));
            }
        }

        // Find edges that need rebuilding
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            if scope.functions_to_reanalyze.contains(&func.full_path) {
                for callee in call_graph.get_callees(idx) {
                    if scope.functions_to_reanalyze.contains(&callee.full_path) {
                        scope
                            .edges_to_rebuild
                            .insert((func.full_path.clone(), callee.full_path.clone()));
                    }
                }
            }
        }

        scope
    }

    /// Get affected functions (callers of changed functions) - simple version
    pub fn get_affected_functions(
        &self,
        call_graph: &CallGraph,
        changed_files: &[PathBuf],
    ) -> HashSet<String> {
        self.get_affected_functions_full(changed_files, call_graph)
    }

    /// Cache a function for incremental analysis
    pub fn cache_function(&mut self, func: &IncrementalFunction) {
        self.function_cache
            .insert(func.full_path.clone(), func.clone());
    }

    /// Get cached function if available
    pub fn get_cached(&self, full_path: &str) -> Option<&IncrementalFunction> {
        self.function_cache.get(full_path)
    }

    fn hash_file(&self, source: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(source.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

impl Default for FileTracker {
    fn default() -> Self {
        Self::new()
    }
}
