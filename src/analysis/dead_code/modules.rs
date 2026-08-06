// src/analysis/dead_code/modules.rs

use crate::graph::import_graph::ImportGraph;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DeadModuleReport {
    pub unused_modules: Vec<DeadModule>,
    pub unused_files: Vec<DeadFile>,
    pub unused_imports: Vec<DeadImport>,
}

#[derive(Debug, Clone)]
pub struct DeadModule {
    pub name: String,
    pub path: PathBuf,
    pub confidence: f64,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct DeadFile {
    pub path: PathBuf,
    pub confidence: f64,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct DeadImport {
    pub module: String,
    pub imported_by: String,
    pub line: usize,
    pub confidence: f64,
}

pub struct ModuleDeadCodeDetector;

impl ModuleDeadCodeDetector {
    pub fn detect_dead_modules(_import_graph: &ImportGraph) -> DeadModuleReport {
        // TODO: Implement once ImportGraph supports iteration
        DeadModuleReport {
            unused_modules: Vec::new(),
            unused_files: Vec::new(),
            unused_imports: Vec::new(),
        }
    }
}
