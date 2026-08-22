// src/analysis/dead_code/mod.rs

mod analyzer;
pub mod filters;
mod modules;
mod report;
mod scorer;
mod types;
mod whitelist;
// reachability.rs has been removed - using roots::ReachabilityAnalyzer instead

pub use analyzer::{
    AnalysisSummary, DeadCodeAnalysis, DeadCodeAnalyzer, DeadFunction, FunctionImpact, RemovalCost,
};
pub use modules::{DeadFile, DeadImport, DeadModule, DeadModuleReport};
pub use report::DeadCodeReportGenerator;
pub use scorer::{ConfidenceLevel, ConfidenceScorer, DeadScore, ScoreFactor, ScoreWeights};
pub use types::{DeadType, DeadTypeReport};
pub use whitelist::WHITELIST;

pub use filters::{filter_reason, is_framework_file, is_never_dead};

use crate::analysis::dynamic_refs::DynamicRefDetector;
use crate::analysis::roots::{ReachabilityAnalyzer, RootDetectionConfig, RootDetector};
use crate::analysis::verdict_source::state::{VerdictConfig, VerdictEngine};
use crate::graph::call_graph::CallGraph;
use crate::graph::dependency_graph::DependencyGraph;
use crate::graph::import_graph::ImportGraph;
use crate::graph::traits::GraphMetrics;
use crate::graph::type_graph::TypeGraph;
use crate::parser::tree_sitter::ParsedFile;

#[derive(Debug, Clone, Default)]
pub struct DeadStats {
    pub total: usize,
    pub dead: usize,
    pub alive: usize,
}

pub struct DeadCodeDetector;

impl DeadCodeDetector {
    pub fn get_dead_stats(call_graph: &CallGraph, files: &[ParsedFile]) -> DeadStats {
        let root_config = RootDetectionConfig::default();
        let root_set = RootDetector::detect_roots(call_graph, files, &root_config);
        let reachability = ReachabilityAnalyzer::compute_reachability(call_graph, &root_set);
        let engine = VerdictEngine::new(VerdictConfig::default());
        let verdicts = engine.evaluate_all(call_graph, &reachability);
        let dead_count = engine.filter_dead(&verdicts).len();

        DeadStats {
            total: call_graph.node_count(),
            dead: dead_count,
            alive: call_graph.node_count() - dead_count,
        }
    }

    pub fn find_dead_modules(files: &[ParsedFile]) -> Vec<String> {
        files
            .iter()
            .filter(|f| !f.functions.is_empty() && !f.functions.iter().any(|func| func.is_public))
            .map(|f| f.path.clone())
            .collect()
    }

    pub fn analyze(
        call_graph: &CallGraph,
        type_graph: &TypeGraph,
        import_graph: &ImportGraph,
        _dependency_graph: &DependencyGraph,
        files: &[ParsedFile],
        _git_analysis: Option<&crate::analysis::git_analysis::GitAnalysis>,
    ) -> DeadCodeAnalysis {
        let root_config = RootDetectionConfig::default();
        let root_set = RootDetector::detect_roots(call_graph, files, &root_config);
        let reachability = ReachabilityAnalyzer::compute_reachability(call_graph, &root_set);

        let dynamic_detector = DynamicRefDetector::new();
        let dynamic_refs = dynamic_detector.detect_all(call_graph, files);

        let verdict_engine =
            VerdictEngine::new(VerdictConfig::default()).with_dynamic_refs(dynamic_refs);

        let verdicts = verdict_engine.evaluate_all(call_graph, &reachability);
        let dead_verdicts: Vec<_> = verdict_engine.filter_dead(&verdicts);

        let mut analyzer = DeadCodeAnalyzer::new();
        let dead_functions = analyzer.import_verdicts(&dead_verdicts, call_graph);

        analyzer.analyze_structural_components(
            dead_functions,
            reachability,
            call_graph,
            type_graph,
            import_graph,
        )
    }

    pub fn generate_report(analysis: &DeadCodeAnalysis) -> String {
        DeadCodeReportGenerator::generate_report(analysis)
    }

    pub fn dead_code_ratio(call_graph: &CallGraph, files: &[ParsedFile]) -> f64 {
        let stats = Self::get_dead_stats(call_graph, files);
        if stats.total == 0 {
            0.0
        } else {
            stats.dead as f64 / stats.total as f64
        }
    }
}
