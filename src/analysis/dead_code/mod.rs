// src/analysis/dead_code/mod.rs

mod analyzer;
mod modules;
mod reachability;
mod report;
mod scorer;
mod types;
mod whitelist; // ⭐ NEW - Add this line

// Re-export from analyzer only (it has the most complete definitions)

pub use analyzer::{
    AnalysisSummary, DeadCodeAnalysis, DeadCodeAnalyzer, DeadFunction, FunctionImpact, RemovalCost,
};
pub use modules::{DeadFile, DeadImport, DeadModule, DeadModuleReport};
pub use reachability::ReachabilityReport;
pub use report::DeadCodeReportGenerator;
pub use scorer::{ConfidenceLevel, ConfidenceScorer, DeadScore, ScoreFactor, ScoreWeights};
pub use types::{DeadType, DeadTypeReport};
pub use whitelist::WHITELIST; // ⭐ NEW - Export the whitelist

use crate::graph::call_graph::CallGraph;
use crate::graph::traits::GraphMetrics;
use crate::parser::tree_sitter::ParsedFile;

pub struct DeadCodeDetector;

impl DeadCodeDetector {
    /// Legacy function for backward compatibility
    pub fn find_unused_functions(call_graph: &CallGraph) -> Vec<String> {
        let _analyzer = DeadCodeAnalyzer::new();
        // We need to pass type_graph, import_graph, dependency_graph
        // For now, use the old logic
        let mut unused = Vec::new();
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            if func.fan_in == 0 && !func.is_public {
                unused.push(func.full_path.clone());
            }
        }
        unused
    }

    /// New comprehensive analysis
    pub fn analyze(
        call_graph: &CallGraph,
        type_graph: &crate::graph::type_graph::TypeGraph,
        import_graph: &crate::graph::import_graph::ImportGraph,
        dependency_graph: &crate::graph::dependency_graph::DependencyGraph,
        files: &[ParsedFile],
        git_analysis: Option<&crate::analysis::git_analysis::GitAnalysis>,
    ) -> DeadCodeAnalysis {
        let mut analyzer = DeadCodeAnalyzer::new();
        analyzer.analyze(
            call_graph,
            type_graph,
            import_graph,
            dependency_graph,
            files,
            git_analysis,
        )
    }

    pub fn generate_report(analysis: &DeadCodeAnalysis) -> String {
        DeadCodeReportGenerator::generate_report(analysis)
    }

    pub fn find_dead_modules(files: &[ParsedFile]) -> Vec<String> {
        files
            .iter()
            .filter(|f| !f.functions.is_empty() && !f.functions.iter().any(|func| func.is_public))
            .map(|f| f.path.clone())
            .collect()
    }

    pub fn dead_code_ratio(call_graph: &CallGraph, _files: &[ParsedFile]) -> f64 {
        let total = call_graph.node_count();
        if total == 0 {
            return 0.0;
        }
        let unused = Self::find_unused_functions(call_graph);
        unused.len() as f64 / total as f64
    }
}
