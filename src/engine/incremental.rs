// src/engine/incremental.rs

use crate::graph::call_graph::CallGraph;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::SystemTime;

/// Tracks file changes for incremental analysis
#[derive(Debug, Clone)]
pub struct FileTracker {
    /// File path -> (modified_time, content_hash)
    files: HashMap<PathBuf, (SystemTime, String)>,
    /// Functions that were previously analyzed
    function_cache: HashMap<String, IncrementalFunction>,
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
}

#[derive(Debug, Clone, Default)]
pub struct IncrementalResult {
    pub changed_files: Vec<PathBuf>,
    pub affected_functions: Vec<String>,
    pub removed_functions: Vec<String>,
    pub added_functions: Vec<String>,
    pub modified_functions: Vec<String>,
    pub cache_hit: bool,
}

impl FileTracker {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            function_cache: HashMap::new(),
        }
    }

    /// Check which files have changed
    pub fn detect_changes(&mut self, files: &[ParsedFile]) -> Vec<PathBuf> {
        let mut changed = Vec::new();

        for file in files {
            let path = PathBuf::from(&file.path);
            let hash = self.hash_file(&file.source);

            if let Some((_, old_hash)) = self.files.get(&path) {
                if old_hash != &hash {
                    changed.push(path.clone());
                }
            } else {
                changed.push(path.clone());
            }

            self.files.insert(path, (SystemTime::now(), hash));
        }

        changed
    }

    /// Get functions affected by file changes
    pub fn get_affected_functions(
        &self,
        call_graph: &CallGraph,
        changed_files: &[PathBuf],
    ) -> HashSet<String> {
        let mut affected = HashSet::new();

        // Get all functions in changed files
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            let file_path = PathBuf::from(&func.file);

            if changed_files.contains(&file_path) {
                affected.insert(func.full_path.clone());

                // Also add callers and callees
                for caller in call_graph.get_callers(idx) {
                    affected.insert(caller.full_path.clone());
                }
                for callee in call_graph.get_callees(idx) {
                    affected.insert(callee.full_path.clone());
                }
            }
        }

        affected
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
