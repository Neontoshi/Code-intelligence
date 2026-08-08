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

    /// New comprehensive analysis.
    ///
    /// Routes through VerdictEngine (roots -> reachability -> per-function
    /// verdicts -> import_verdicts), same as `dead_code_check`'s bin. The old
    /// path called the deprecated `DeadCodeAnalyzer::analyze()` directly,
    /// which is a no-op whenever `use_verdict_engine` is true (the default),
    /// so this wrapper was silently returning an empty analysis. No ML model
    /// is threaded through here since this signature never received one —
    /// static signals only. Callers who want ML-assisted verdicts should
    /// build a VerdictEngine directly with `.with_ml(...)`.
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

        // 1. Roots + reachability
        let root_config = RootDetectionConfig::default();
        let root_set = RootDetector::detect_roots(call_graph, files, &root_config);
        let reachability = ReachabilityAnalyzer::compute_reachability(call_graph, &root_set);

        // 2. Verdict engine (static signals only, no ML model here)
        let mut verdict_engine = VerdictEngine::new(VerdictConfig::default());
        let dynamic_detector = DynamicRefDetector::new();
        let dynamic_refs = dynamic_detector.detect_all(call_graph, files);
        verdict_engine = verdict_engine.with_dynamic_refs(dynamic_refs);

        let verdicts = verdict_engine.evaluate_all(call_graph, &reachability);
        let dead_verdicts: Vec<_> = verdict_engine.filter_dead(&verdicts);

        // 3. Import verdicts to get DeadFunction list with impact metadata
        let mut impact_analyzer = DeadCodeAnalyzer::new_for_impact_only();
        let dead_functions = impact_analyzer.import_verdicts(&dead_verdicts, call_graph);

        // 4. Reuse the impact-only analyzer for types/modules (no dead/alive
        // decisions made there — those come from verdicts above)
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

    pub fn dead_code_ratio(call_graph: &CallGraph, _files: &[ParsedFile]) -> f64 {
        let total = call_graph.node_count();
        if total == 0 {
            return 0.0;
        }
        let unused = Self::find_unused_functions(call_graph);
        unused.len() as f64 / total as f64
    }
}
