// src/analysis/dead_code/analyzer.rs

use crate::analysis::git_analysis::GitAnalysis;
use crate::graph::call_graph::{CallGraph, FunctionNode};
use crate::graph::dependency_graph::DependencyGraph;
use crate::graph::graph_traits::GraphMetrics; // Also add this for node_count()
use crate::graph::import_graph::ImportGraph;
use crate::graph::type_graph::TypeGraph;
use crate::parser::tree_sitter::ParsedFile;

use super::modules::{DeadModuleReport, ModuleDeadCodeDetector};
use super::reachability::{ReachabilityAnalyzer, ReachabilityReport};
use super::scorer::{ConfidenceLevel, ConfidenceScorer, DeadScore};
use super::types::{DeadTypeReport, TypeDeadCodeDetector};

#[derive(Debug, Clone)]
pub struct DeadCodeAnalysis {
    pub functions: Vec<DeadFunction>,
    pub types: DeadTypeReport,
    pub modules: DeadModuleReport,
    pub reachability: ReachabilityReport,
    pub summary: AnalysisSummary,
}

#[derive(Debug, Clone)]
pub struct DeadFunction {
    pub full_path: String,
    pub name: String,
    pub file: String,
    pub line: usize,
    pub score: DeadScore,
    pub impact: FunctionImpact,
    pub removal_order: usize,
}

#[derive(Debug, Clone)]
pub struct FunctionImpact {
    pub lines_of_code: usize,
    pub dependencies: Vec<String>,
    pub complexity: f64,
    pub estimated_removal_impact: String,
}

#[derive(Debug, Clone)]
pub struct AnalysisSummary {
    pub total_functions: usize,
    pub dead_functions: usize,
    pub dead_types: usize,
    pub dead_modules: usize,
    pub dead_files: usize,
    pub avg_confidence: f64,
    pub estimated_loc_removable: usize,
}

pub struct DeadCodeAnalyzer {
    scorer: ConfidenceScorer,
}

impl DeadCodeAnalyzer {
    pub fn new() -> Self {
        Self {
            scorer: ConfidenceScorer::new(),
        }
    }

    pub fn analyze(
        &self,
        call_graph: &CallGraph,
        type_graph: &TypeGraph,
        import_graph: &ImportGraph,
        _dependency_graph: &DependencyGraph,
        _files: &[ParsedFile],
        git_analysis: Option<&GitAnalysis>,
    ) -> DeadCodeAnalysis {
        // 1. Reachability Analysis
        let reachability = ReachabilityAnalyzer::analyze_reachability(call_graph);

        // 2. Function dead code analysis
        let mut dead_functions = Vec::new();
        let mut total_loc = 0;

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];

            // Skip functions with callers
            if func.fan_in > 0 {
                continue;
            }

            // Score the function
            let git_info =
                git_analysis.and_then(|g| g.files.get(&std::path::PathBuf::from(&func.file)));
            let score = self.scorer.score_function(func, git_info);

            // Only consider if confidence is high enough
            if matches!(
                score.level,
                ConfidenceLevel::Probably
                    | ConfidenceLevel::VeryLikely
                    | ConfidenceLevel::Guaranteed
            ) {
                // Calculate impact
                let impact = self.calculate_impact(func, call_graph);
                total_loc += impact.lines_of_code;

                dead_functions.push(DeadFunction {
                    full_path: func.full_path.clone(),
                    name: func.name.clone(),
                    file: func.file.clone(),
                    line: func.line,
                    score,
                    impact,
                    removal_order: 0, // Will be set later
                });
            }
        }

        // 3. Type dead code analysis
        let type_report = TypeDeadCodeDetector::detect_dead_types(type_graph, call_graph);

        // 4. Module dead code analysis
        let module_report = ModuleDeadCodeDetector::detect_dead_modules(import_graph);

        // 5. Sort by priority and assign removal order
        dead_functions.sort_by(|a, b| b.score.score.partial_cmp(&a.score.score).unwrap());

        for (i, func) in dead_functions.iter_mut().enumerate() {
            func.removal_order = i + 1;
        }

        // 6. Generate summary
        let summary = AnalysisSummary {
            total_functions: call_graph.node_count(),
            dead_functions: dead_functions.len(),
            dead_types: type_report.unused_structs.len()
                + type_report.unused_enums.len()
                + type_report.unused_traits.len()
                + type_report.unused_type_aliases.len(),
            dead_modules: module_report.unused_modules.len(),
            dead_files: module_report.unused_files.len(),
            avg_confidence: if dead_functions.is_empty() {
                0.0
            } else {
                dead_functions.iter().map(|f| f.score.score).sum::<f64>()
                    / dead_functions.len() as f64
            },
            estimated_loc_removable: total_loc,
        };

        DeadCodeAnalysis {
            functions: dead_functions,
            types: type_report,
            modules: module_report,
            reachability,
            summary,
        }
    }

    fn calculate_impact(&self, func: &FunctionNode, call_graph: &CallGraph) -> FunctionImpact {
        // Find the function index
        let mut idx = None;
        for i in call_graph.node_indices() {
            if call_graph[i].full_path == func.full_path {
                idx = Some(i);
                break;
            }
        }

        let mut dependencies = Vec::new();
        let mut complexity = func.complexity;

        if let Some(idx) = idx {
            for callee in call_graph.get_callees(idx) {
                dependencies.push(callee.full_path.clone());
                complexity += callee.complexity * 0.1;
            }
        }

        // Estimate LOC (rough)
        let lines_of_code = 20 + (func.complexity * 5.0) as usize;

        let estimated_removal_impact = if dependencies.is_empty() {
            "Low impact - self-contained function".to_string()
        } else if dependencies.len() <= 3 {
            format!(
                "Medium impact - affects {} dependencies",
                dependencies.len()
            )
        } else {
            format!("High impact - affects {} dependencies", dependencies.len())
        };

        FunctionImpact {
            lines_of_code,
            dependencies,
            complexity,
            estimated_removal_impact,
        }
    }
}

impl Default for DeadCodeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
