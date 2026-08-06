// src/analysis/dead_code/analyzer.rs

use crate::analysis::git_analysis::GitAnalysis;
use crate::graph::call_graph::{CallGraph, FunctionNode};
use crate::graph::dependency_graph::DependencyGraph;
use crate::graph::graph_traits::GraphMetrics;
use crate::graph::import_graph::ImportGraph;
use crate::graph::type_graph::TypeGraph;
use crate::parser::tree_sitter::ParsedFile;

use super::modules::{DeadModuleReport, ModuleDeadCodeDetector};
use super::reachability::{ReachabilityAnalyzer, ReachabilityReport};
use super::scorer::{ConfidenceLevel, ConfidenceScorer, DeadScore};
use super::types::{DeadTypeReport, TypeDeadCodeDetector};

use std::collections::HashMap;

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
    scorer: ConfidenceScorer,
    cache: HashMap<String, DeadCodeAnalysis>,
}

impl DeadCodeAnalyzer {
    pub fn new() -> Self {
        Self {
            scorer: ConfidenceScorer::new(),
            cache: HashMap::new(),
        }
    }

    /// Check if a function should be excluded from dead code analysis
    fn is_excluded_function(func: &FunctionNode) -> bool {
        // Skip trait implementations (they're required by traits)
        if func.trait_impl.is_some() {
            return true;
        }

        // Skip benchmark functions
        if func.file.contains("benches/") {
            return true;
        }

        // Skip test functions
        if func.name.starts_with("test_") || func.name.starts_with("bench_") {
            return true;
        }

        // Skip main entry points
        if func.name == "main"
            && (func.file.contains("src/bin/") || func.file.contains("src/main.rs"))
        {
            return true;
        }

        // Skip default implementations
        if func.name == "default" {
            return true;
        }

        // Skip trait method implementations
        let trait_methods = [
            "generate",
            "generate_stream",
            "model_name",
            "max_context_length",
            "is_available",
        ];
        if trait_methods.contains(&func.name.as_str()) && func.trait_impl.is_some() {
            return true;
        }

        // Skip functions in test modules
        if func.file.contains("/tests/") || func.file.ends_with("_test.rs") {
            return true;
        }

        // Skip functions that are part of a trait implementation
        if func.file.contains("providers/") && trait_methods.contains(&func.name.as_str()) {
            return true;
        }

        false
    }

    /// Check if a function is likely React code that should be skipped
    fn is_likely_react_code(func: &FunctionNode) -> bool {
        // Check if it's a React component file
        let is_tsx = func.file.ends_with(".tsx") || func.file.ends_with(".jsx");
        let is_jsx = func.file.ends_with(".jsx");

        // Check if the name starts with uppercase (React component convention)
        let is_component = func
            .name
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false);

        // Check if it's a React hook (useState, useEffect, etc.)
        let is_hook = func.name.starts_with("use") && !func.name.starts_with("useSolanaGiveaway");

        // Check if it's a state setter
        let is_setter = func.name.starts_with("set")
            && func
                .name
                .chars()
                .nth(3)
                .map(|c| c.is_uppercase())
                .unwrap_or(false);

        // Check if it's a React component file
        let is_react_file = func.file.contains("components/")
            || func.file.contains("pages/")
            || func.file.contains("providers/");

        // Skip state hooks (useState, setState, etc.)
        let is_state_hook = func.name.contains("useState")
            || func.name.contains("useEffect")
            || func.name.contains("useRef")
            || func.name.contains("useCallback")
            || func.name.contains("useMemo")
            || func.name.contains("useContext")
            || func.name.contains("useReducer");

        (is_tsx || is_jsx)
            && (is_component || is_hook || is_setter || is_react_file || is_state_hook)
    }

    /// Check if a function is a React Router hook result
    fn is_react_router_hook(func: &FunctionNode) -> bool {
        // React Router hooks
        let router_hooks = [
            "useLocation",
            "useNavigate",
            "useParams",
            "useSearchParams",
            "useRouteMatch",
            "useRoutes",
            "useOutletContext",
            "useOutlet",
            "useResolvedPath",
            "useHref",
            "useInRouterContext",
            "useNavigationType",
            "useSubmit",
            "useFetcher",
            "useFetchers",
            "useRevalidator",
            "useNavigation",
        ];

        // Check if it's a variable that holds a router hook result
        let is_router_var = router_hooks.contains(&func.name.as_str());

        // Check if it's a destructured variable from a router hook
        let is_destructured = func.name == "location"
            || func.name == "navigate"
            || func.name == "params"
            || func.name == "searchParams"
            || func.name == "match"
            || func.name == "routes";

        // Check if the file imports react-router-dom
        let is_router_file = func.file.contains("App.tsx")
            || func.file.contains("Router")
            || func.file.contains("routes");

        is_router_var || (is_destructured && is_router_file)
    }

    /// Check if a function is a Router component
    fn is_router_component(func: &FunctionNode) -> bool {
        let router_components = ["CreatePage", "SearchPage", "App"];
        router_components.contains(&func.name.as_str())
    }

    /// Check if a function is an API client method that's actually used
    fn is_alive_api_method(func: &FunctionNode) -> bool {
        let alive_methods = [
            "constructor",
            "request",
            "buildCreateAndCommitGiveaway",
            "submitGiveaway",
            "buildReveal",
            "submitReveal",
        ];
        alive_methods.contains(&func.name.as_str())
    }

    /// Check if a function is exported but unused
    fn is_exported_but_unused(&self, func: &FunctionNode, import_graph: &ImportGraph) -> bool {
        // Check if function is exported
        let is_exported = func.file.contains("export") || func.file.contains("pub fn");

        // Check if it's imported anywhere
        let is_imported = import_graph.get_importers(&func.full_path).len() > 0;

        is_exported && !is_imported
    }

    /// Generate cache key for analysis
    fn get_cache_key(call_graph: &CallGraph) -> String {
        format!("cg_{}", call_graph.node_count())
    }

    pub fn analyze(
        &mut self,
        call_graph: &CallGraph,
        type_graph: &TypeGraph,
        import_graph: &ImportGraph,
        _dependency_graph: &DependencyGraph,
        _files: &[ParsedFile],
        git_analysis: Option<&GitAnalysis>,
    ) -> DeadCodeAnalysis {
        // Check cache first
        let cache_key = Self::get_cache_key(call_graph);
        if let Some(cached) = self.cache.get(&cache_key) {
            return cached.clone();
        }

        // 1. Reachability Analysis
        let reachability = ReachabilityAnalyzer::analyze_reachability(call_graph);

        // 2. Function dead code analysis
        let mut dead_functions = Vec::new();
        let mut total_loc = 0;

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];

            // Skip excluded functions (trait methods, benchmarks, tests, etc.)
            if Self::is_excluded_function(func) {
                continue;
            }

            // Skip React components and hooks
            if Self::is_likely_react_code(func) {
                continue;
            }

            // Skip React Router hooks
            if Self::is_react_router_hook(func) {
                continue;
            }

            // Skip Router components
            if Self::is_router_component(func) {
                continue;
            }

            // Skip alive API methods
            if Self::is_alive_api_method(func) {
                continue;
            }

            // Skip exported but unused functions (they might be used externally)
            if self.is_exported_but_unused(func, import_graph) {
                continue;
            }

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

        let analysis = DeadCodeAnalysis {
            functions: dead_functions,
            types: type_report,
            modules: module_report,
            reachability,
            summary,
        };

        // Cache the result
        self.cache.insert(cache_key, analysis.clone());

        analysis
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

        let (estimated_removal_impact, removal_cost) = if dependencies.is_empty() {
            (
                "Low impact - self-contained function".to_string(),
                RemovalCost::Low,
            )
        } else if dependencies.len() <= 3 {
            (
                format!(
                    "Medium impact - affects {} dependencies",
                    dependencies.len()
                ),
                RemovalCost::Medium,
            )
        } else {
            (
                format!("High impact - affects {} dependencies", dependencies.len()),
                RemovalCost::High,
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

    /// Generate a detailed report from the analysis
    pub fn generate_report(&self, analysis: &DeadCodeAnalysis) -> String {
        let mut report = String::new();
        report.push_str("=== Dead Code Analysis Report ===\n\n");
        report.push_str(&format!(
            "Total functions: {}\n",
            analysis.summary.total_functions
        ));
        report.push_str(&format!(
            "Dead functions: {}\n",
            analysis.summary.dead_functions
        ));
        report.push_str(&format!("Dead types: {}\n", analysis.summary.dead_types));
        report.push_str(&format!(
            "Dead modules: {}\n",
            analysis.summary.dead_modules
        ));
        report.push_str(&format!("Dead files: {}\n", analysis.summary.dead_files));
        report.push_str(&format!(
            "Estimated LOC removable: {}\n",
            analysis.summary.estimated_loc_removable
        ));
        report.push_str(&format!(
            "Average confidence: {:.2}%\n\n",
            analysis.summary.avg_confidence * 100.0
        ));

        // Group by confidence level
        for level in [
            ConfidenceLevel::Guaranteed,
            ConfidenceLevel::VeryLikely,
            ConfidenceLevel::Probably,
        ] {
            let functions: Vec<_> = analysis
                .functions
                .iter()
                .filter(|f| f.score.level == level)
                .collect();
            if !functions.is_empty() {
                report.push_str(&format!("\n{:?} ({})\n", level, functions.len()));
                for func in functions {
                    report.push_str(&format!(
                        "  #{} - {} ({}@{})\n",
                        func.removal_order, func.name, func.file, func.line
                    ));
                    report.push_str(&format!(
                        "    Confidence: {:.2}%, Impact: {}\n",
                        func.score.score * 100.0,
                        func.impact.estimated_removal_impact
                    ));
                    report.push_str(&format!(
                        "    Removal cost: {:?}\n",
                        func.impact.removal_cost
                    ));
                }
            }
        }

        report
    }

    /// Generate safe removal steps for a dead function
    pub fn safe_removal_plan(&self, dead_func: &DeadFunction) -> Vec<String> {
        let mut steps = Vec::new();
        steps.push(format!("1. Remove function: {}", dead_func.full_path));

        if dead_func.impact.dependencies.is_empty() {
            steps.push("2. ✓ Safe to remove (no dependencies)".to_string());
        } else {
            steps.push(format!(
                "2. ⚠ Check {} dependencies:",
                dead_func.impact.dependencies.len()
            ));
            for dep in &dead_func.impact.dependencies {
                steps.push(format!("   - {}", dep));
            }
            steps.push("3. Consider removing dependencies first or refactoring".to_string());
            steps.push(format!(
                "4. ⚠ Removal cost: {:?}",
                dead_func.impact.removal_cost
            ));
        }

        steps
    }

    /// Clear the analysis cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for DeadCodeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
