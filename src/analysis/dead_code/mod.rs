// src/analysis/dead_code/mod.rs

mod analyzer;
pub mod filters;
mod modules;
mod reachability;
mod report;
mod scorer;
mod types;
mod whitelist;

pub use analyzer::{
    AnalysisSummary, DeadCodeAnalysis, DeadCodeAnalyzer, DeadFunction, FunctionImpact, RemovalCost,
};
pub use modules::{DeadFile, DeadImport, DeadModule, DeadModuleReport};
pub use reachability::ReachabilityReport;
pub use report::DeadCodeReportGenerator;
pub use scorer::{ConfidenceLevel, ConfidenceScorer, DeadScore, ScoreFactor, ScoreWeights};
pub use types::{DeadType, DeadTypeReport};
pub use whitelist::WHITELIST;

pub use filters::{filter_reason, is_framework_file, is_never_dead};

use crate::graph::call_graph::CallGraph;
use crate::graph::traits::GraphMetrics;
use crate::parser::tree_sitter::ParsedFile;

/// Dead code statistics
#[derive(Debug, Clone, Default)]
pub struct DeadStats {
    pub total: usize,
    pub dead: usize,
    pub alive: usize,
}

pub struct DeadCodeDetector;

impl DeadCodeDetector {
    /// ⚠️ DEPRECATED: This is a naive implementation that only checks fan_in == 0 && !is_public.
    /// It produces many false positives (trait impls, framework methods, etc.).
    /// Use VerdictEngine instead for accurate dead code detection.
    ///
    /// This is kept only for debug/backward compatibility and should not be used
    /// in production reports.
    #[deprecated(
        since = "0.2.0",
        note = "Use VerdictEngine for accurate dead code detection. This naive check will be removed in a future version."
    )]
    #[allow(dead_code)]
    pub fn find_unused_functions_naive_debug_only(call_graph: &CallGraph) -> Vec<String> {
        let mut unused = Vec::new();
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            if func.fan_in == 0 && !func.is_public {
                unused.push(func.full_path.clone());
            }
        }
        unused
    }

    /// Get dead code statistics using the verdict-based approach.
    /// This is the recommended way to detect dead code.
    pub fn get_dead_stats(call_graph: &CallGraph, files: &[ParsedFile]) -> DeadStats {
        use crate::analysis::roots::{ReachabilityAnalyzer, RootDetectionConfig, RootDetector};
        use crate::analysis::verdict::{VerdictConfig, VerdictEngine};

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

    #[allow(deprecated)]
    pub fn analyze(
        call_graph: &CallGraph,
        type_graph: &crate::graph::type_graph::TypeGraph,
        import_graph: &crate::graph::import_graph::ImportGraph,
        dependency_graph: &crate::graph::dependency_graph::DependencyGraph,
        files: &[ParsedFile],
        git_analysis: Option<&crate::analysis::git_analysis::GitAnalysis>,
    ) -> DeadCodeAnalysis {
        use crate::analysis::dynamic_refs::DynamicRefDetector;
        use crate::analysis::roots::{ReachabilityAnalyzer, RootDetectionConfig, RootDetector};
        use crate::analysis::verdict::{VerdictConfig, VerdictEngine};

        let root_config = RootDetectionConfig::default();
        let root_set = RootDetector::detect_roots(call_graph, files, &root_config);
        let reachability = ReachabilityAnalyzer::compute_reachability(call_graph, &root_set);

        let mut verdict_engine = VerdictEngine::new(VerdictConfig::default());
        let dynamic_detector = DynamicRefDetector::new();
        let dynamic_refs = dynamic_detector.detect_all(call_graph, files);
        verdict_engine = verdict_engine.with_dynamic_refs(dynamic_refs);

        let verdicts = verdict_engine.evaluate_all(call_graph, &reachability);
        let dead_verdicts: Vec<_> = verdict_engine.filter_dead(&verdicts);

        let mut impact_analyzer = DeadCodeAnalyzer::new_for_impact_only();
        let dead_functions = impact_analyzer.import_verdicts(&dead_verdicts, call_graph);

        let legacy = impact_analyzer.analyze(
            call_graph,
            type_graph,
            import_graph,
            dependency_graph,
            files,
            git_analysis,
        );

        DeadCodeAnalysis {
            functions: dead_functions.clone(),
            types: legacy.types,
            modules: legacy.modules,
            reachability,
            summary: AnalysisSummary {
                total_functions: call_graph.node_count(),
                dead_functions: dead_functions.len(),
                dead_types: legacy.summary.dead_types,
                dead_modules: legacy.summary.dead_modules,
                dead_files: legacy.summary.dead_files,
                avg_confidence: if dead_functions.is_empty() {
                    0.0
                } else {
                    dead_functions.iter().map(|f| f.score.score).sum::<f64>()
                        / dead_functions.len() as f64
                },
                estimated_loc_removable: dead_functions
                    .iter()
                    .map(|f| f.impact.lines_of_code)
                    .sum(),
            },
        }
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

    pub fn dead_code_ratio(call_graph: &CallGraph, files: &[ParsedFile]) -> f64 {
        let stats = Self::get_dead_stats(call_graph, files);
        if stats.total == 0 {
            0.0
        } else {
            stats.dead as f64 / stats.total as f64
        }
    }
}
