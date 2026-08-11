// src/engine/stages.rs

use crate::analysis::context::ProjectMetrics;
use crate::analysis::features::FeatureExtractor;
use crate::engine::indexer::RichIndexes;
use crate::graph::call_graph::CallGraph;
use crate::graph::project_graph::ProjectGraph;
use crate::parser::tree_sitter::ParsedFile;
use std::path::PathBuf;

/// Stage 1: Raw file collection
pub struct RawProject {
    pub root: PathBuf,
    pub files: Vec<PathBuf>,
}

/// Stage 2: Parsed project
pub struct ParsedProject {
    pub root: PathBuf,
    pub files: Vec<ParsedFile>,
}

/// Stage 3: Analyzed project with graph
pub struct AnalyzedProject {
    pub root: PathBuf,
    pub files: Vec<ParsedFile>,
    pub call_graph: CallGraph,
    pub project_graph: ProjectGraph,
    pub cycle_detection_skipped: bool,
    pub cycle_detection_node_count: usize,
}

/// Stage 4: Optimized project with features and indexes
pub struct OptimizedProject {
    pub root: PathBuf,
    pub files: Vec<ParsedFile>,
    pub call_graph: CallGraph,
    pub project_graph: ProjectGraph,
    pub features: FeatureExtractor,
    pub rich_indexes: RichIndexes,
    pub metrics: ProjectMetrics,
}
