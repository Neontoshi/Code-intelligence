// src/analysis/dead_code/analyzer.rs
use super::modules::{DeadModuleReport, ModuleDeadCodeDetector};
use super::scorer::{ConfidenceLevel, DeadScore, ScoreFactor};
use super::types::{DeadTypeReport, TypeDeadCodeDetector};
use crate::analysis::roots::ReachabilityMap;
use crate::analysis::verdict_source::Verdict;
use crate::graph::call_graph::{CallGraph, FunctionNode};
use crate::graph::import_graph::ImportGraph;
use crate::graph::traits::GraphMetrics;
use crate::graph::type_graph::TypeGraph;
use std::collections::HashMap;

#[derive(Clone)]
pub struct DeadCodeAnalysis {
    pub functions: Vec<DeadFunction>,
    pub types: DeadTypeReport,
    pub modules: DeadModuleReport,
    pub reachability: ReachabilityMap,
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
    pub is_binary_only: bool,
    pub is_internal_call: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionImpact {
    pub lines_of_code: usize,
    pub dependencies: Vec<String>,
    pub complexity: f64,
    pub estimated_removal_impact: String,
    pub removal_cost: RemovalCost,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemovalCost {
    Low,
    Medium,
    High,
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
    cache: HashMap<String, DeadCodeAnalysis>,
}

impl DeadCodeAnalyzer {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn import_verdicts(
        &mut self,
        verdicts: &[&Verdict],
        call_graph: &CallGraph,
    ) -> Vec<DeadFunction> {
        let mut dead_functions = Vec::new();

        for verdict in verdicts.iter().filter(|v| v.is_dead()) {
            let idx = call_graph
                .node_indices()
                .find(|idx| call_graph[*idx].full_path == verdict.full_path);

            if let Some(idx) = idx {
                let func = &call_graph[idx];
                let impact = self.calculate_impact(func, call_graph);
                let is_binary_only = self.is_binary_only_function(func);
                let is_internal_call = func.fan_in == 0 && !is_binary_only && !func.is_public;

                dead_functions.push(DeadFunction {
                    full_path: verdict.full_path.clone(),
                    name: verdict.function_name.clone(),
                    file: func.file.clone(),
                    line: func.line,
                    score: DeadScore {
                        score: verdict.confidence,
                        level: if verdict.confidence > 0.95 {
                            ConfidenceLevel::Guaranteed
                        } else if verdict.confidence > 0.85 {
                            ConfidenceLevel::VeryLikely
                        } else {
                            ConfidenceLevel::Probably
                        },
                        factors: verdict
                            .signals
                            .iter()
                            .map(|s| ScoreFactor {
                                name: s.name.clone(),
                                weight: s.weight,
                                contribution: if s.direction
                                    == crate::analysis::verdict_source::SignalDirection::SupportsDead
                                {
                                    s.weight
                                } else {
                                    -s.weight
                                },
                                explanation: s.explanation.clone(),
                            })
                            .collect(),
                    },
                    impact,
                    removal_order: 0,
                    is_binary_only,
                    is_internal_call,
                });
            }
        }

        dead_functions.sort_by(|a, b| b.score.score.total_cmp(&a.score.score));
        for (i, func) in dead_functions.iter_mut().enumerate() {
            func.removal_order = i + 1;
        }

        dead_functions
    }

    pub fn analyze_structural_components(
        &mut self,
        dead_functions: Vec<DeadFunction>,
        reachability: ReachabilityMap,
        call_graph: &CallGraph,
        type_graph: &TypeGraph,
        import_graph: &ImportGraph,
    ) -> DeadCodeAnalysis {
        let type_report = TypeDeadCodeDetector::detect_dead_types(type_graph, call_graph);
        let module_report = ModuleDeadCodeDetector::detect_dead_modules(import_graph);

        let total_loc: usize = dead_functions.iter().map(|f| f.impact.lines_of_code).sum();
        let avg_confidence = if dead_functions.is_empty() {
            0.0
        } else {
            dead_functions.iter().map(|f| f.score.score).sum::<f64>() / dead_functions.len() as f64
        };

        let summary = AnalysisSummary {
            total_functions: call_graph.node_count(),
            dead_functions: dead_functions.len(),
            dead_types: type_report.unused_structs.len()
                + type_report.unused_enums.len()
                + type_report.unused_traits.len()
                + type_report.unused_type_aliases.len(),
            dead_modules: module_report.unused_modules.len(),
            dead_files: module_report.unused_files.len(),
            avg_confidence,
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

    fn is_binary_only_function(&self, func: &FunctionNode) -> bool {
        func.file.contains("/bin/")
            || func.file.starts_with("src/bin/")
            || func.file.ends_with("main.rs")
            || func.file.contains("/benches/")
            || func.file.contains("/cli/")
            || func.file.contains("/cmd/")
    }

    fn calculate_impact(&self, func: &FunctionNode, call_graph: &CallGraph) -> FunctionImpact {
        let idx = call_graph.name_index.get(&func.full_path).copied();
        let mut dependencies = Vec::new();
        let mut complexity = func.complexity;

        if let Some(idx) = idx {
            for callee in call_graph.get_callees(idx) {
                dependencies.push(callee.full_path.clone());
                complexity += callee.complexity * 0.1;
            }
        }

        let lines_of_code = if func.body_end_line > func.body_start_line {
            func.body_end_line - func.body_start_line + 1
        } else {
            1
        };

        let (estimated_removal_impact, removal_cost) = if lines_of_code >= 50 || complexity >= 15.0
        {
            (
                format!(
                    "High impact - {} LOC, complexity {:.1}",
                    lines_of_code, complexity
                ),
                RemovalCost::High,
            )
        } else if lines_of_code >= 20 || complexity >= 7.0 {
            (
                format!(
                    "Medium impact - {} LOC, complexity {:.1}",
                    lines_of_code, complexity
                ),
                RemovalCost::Medium,
            )
        } else {
            (
                format!(
                    "Low impact - {} LOC, complexity {:.1}",
                    lines_of_code, complexity
                ),
                RemovalCost::Low,
            )
        };

        FunctionImpact {
            lines_of_code,
            dependencies,
            complexity,
            estimated_removal_impact,
            removal_cost,
        }
    }

    pub fn generate_report(&self, analysis: &DeadCodeAnalysis) -> String {
        super::report::DeadCodeReportGenerator::generate_report(analysis)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for DeadCodeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
