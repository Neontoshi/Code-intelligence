// src/analysis/dead_code/analyzer.rs

use crate::analysis::git_analysis::GitAnalysis;
use crate::graph::call_graph::{CallGraph, FunctionNode};
use crate::graph::dependency_graph::DependencyGraph;
use crate::graph::import_graph::ImportGraph;
use crate::graph::type_graph::TypeGraph;
use crate::parser::tree_sitter::ParsedFile;

use super::modules::{DeadModuleReport, ModuleDeadCodeDetector};
use super::scorer::{ConfidenceLevel, ConfidenceScorer, DeadScore};
use super::types::{DeadTypeReport, TypeDeadCodeDetector};
use super::whitelist::WHITELIST;
use crate::analysis::roots::ReachabilityMap;
use crate::graph::traits::GraphMetrics;

#[cfg(feature = "ml")]
use crate::ml::classifier::DeadCodeClassifier;

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
    scorer: ConfidenceScorer,
    cache: HashMap<String, DeadCodeAnalysis>,
    ml_model: Option<DeadCodeClassifier>,
    use_ml: bool,
    use_verdict_engine: bool,
}

impl DeadCodeAnalyzer {
    pub fn new() -> Self {
        Self {
            scorer: ConfidenceScorer::new(),
            cache: HashMap::new(),
            ml_model: None,
            use_ml: false,
            use_verdict_engine: true,
        }
    }

    /// Create analyzer that only computes impact, not verdicts
    pub fn new_for_impact_only() -> Self {
        Self {
            scorer: ConfidenceScorer::new(),
            cache: HashMap::new(),
            ml_model: None,
            use_ml: false,
            use_verdict_engine: true,
        }
    }

    #[deprecated(note = "Use new_for_impact_only() instead")]
    pub fn new_legacy() -> Self {
        Self {
            scorer: ConfidenceScorer::new(),
            cache: HashMap::new(),
            ml_model: None,
            use_ml: false,
            use_verdict_engine: false,
        }
    }

    pub fn with_ml(mut self, model_path: &str) -> Result<Self, String> {
        #[cfg(feature = "ml")]
        {
            let classifier = DeadCodeClassifier::load(model_path)
                .map_err(|e| format!("Failed to load ML model: {}", e))?;
            self.ml_model = Some(classifier);
            self.use_ml = true;
            Ok(self)
        }
        #[cfg(not(feature = "ml"))]
        {
            Err("ML feature not enabled. Recompile with --features ml".to_string())
        }
    }

    pub fn with_classifier(mut self, classifier: DeadCodeClassifier) -> Self {
        self.ml_model = Some(classifier);
        self.use_ml = true;
        self
    }

    pub fn use_ml(&self) -> bool {
        self.use_ml
    }

    pub fn get_ml_model(&self) -> Option<&DeadCodeClassifier> {
        self.ml_model.as_ref()
    }

    pub fn import_verdicts(
        &mut self,
        verdicts: &[&crate::analysis::verdict::Verdict],
        call_graph: &CallGraph,
    ) -> Vec<DeadFunction> {
        let mut dead_functions = Vec::new();

        for verdict in verdicts.iter().filter(|v| v.is_dead()) {
            // Find the actual function node
            let idx = call_graph
                .node_indices()
                .find(|idx| call_graph[*idx].full_path == verdict.full_path);

            if let Some(idx) = idx {
                let func = &call_graph[idx];
                let impact = self.calculate_impact(func, call_graph);

                // Convert verdict to DeadFunction
                dead_functions.push(DeadFunction {
                    full_path: verdict.full_path.clone(),
                    name: verdict.function_name.clone(),
                    file: func.file.clone(),
                    line: func.line,
                    score: crate::analysis::dead_code::DeadScore {
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
                            .map(|s| crate::analysis::dead_code::ScoreFactor {
                                name: s.name.clone(),
                                weight: s.weight,
                                contribution: if s.direction
                                    == crate::analysis::verdict::SignalDirection::SupportsDead
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
                    is_binary_only: false,
                    is_internal_call: false,
                });
            }
        }

        // Sort by confidence and assign removal order
        dead_functions.sort_by(|a, b| b.score.score.total_cmp(&a.score.score));
        for (i, func) in dead_functions.iter_mut().enumerate() {
            func.removal_order = i + 1;
        }

        dead_functions
    }

    /// Check if a function is only used in binary executables
    fn is_binary_only_function(&self, func: &FunctionNode) -> bool {
        // Functions in bin/ directories are binary-only
        if func.file.contains("/bin/") || func.file.starts_with("src/bin/") {
            return true;
        }

        // Functions in main.rs are binary-only
        if func.file.ends_with("main.rs") {
            return true;
        }

        // Functions in benches/ are binary-only
        if func.file.contains("/benches/") {
            return true;
        }

        // Check if it's in a binary-only directory
        let binary_dirs = ["/bin/", "/cli/", "/cmd/", "/executables/"];
        for dir in binary_dirs {
            if func.file.contains(dir) {
                return true;
            }
        }

        // Check if the file name suggests it's a binary
        if func.file.ends_with("_bin.rs") || func.file.ends_with("_cli.rs") {
            return true;
        }

        false
    }

    // Helper methods

    fn is_generated_file(func: &FunctionNode) -> bool {
        func.file.contains(".gen.go")
            || func.file.contains("_gen.go")
            || func.file.contains(".pb.go")
            || func.file.contains("/.meta/")
            || func.file.ends_with(".d.ts")
    }

    fn is_likely_react_code(func: &FunctionNode) -> bool {
        let is_tsx = func.file.ends_with(".tsx") || func.file.ends_with(".jsx");
        let is_jsx = func.file.ends_with(".jsx");
        let is_component = func
            .name
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false);
        let is_hook = func.name.starts_with("use") && !func.name.starts_with("useSolanaGiveaway");
        let is_setter = func.name.starts_with("set")
            && func
                .name
                .chars()
                .nth(3)
                .map(|c| c.is_uppercase())
                .unwrap_or(false);
        let is_react_file = func.file.contains("components/")
            || func.file.contains("pages/")
            || func.file.contains("providers/");
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

    fn get_cache_key(call_graph: &CallGraph) -> String {
        use crate::utils::hashing::HashUtils;

        // Create a hash of the call graph content
        let mut content = String::new();

        // Sort node indices to ensure deterministic order
        let mut indices: Vec<_> = call_graph.node_indices().collect();
        indices.sort_by_key(|idx| idx.index());

        for idx in indices {
            let func = &call_graph[idx];
            content.push_str(&func.full_path);
            content.push('|');

            // Get callees and sort for determinism
            let mut callees: Vec<_> = call_graph
                .get_callees(idx)
                .iter()
                .map(|f| f.full_path.clone())
                .collect();
            callees.sort();

            for callee in callees {
                content.push_str(&callee);
                content.push(',');
            }
            content.push(';');
        }

        // Hash the content
        HashUtils::hash_string(&content)
    }

    // MAIN ANALYSIS METHOD

    #[deprecated(
        since = "0.2.0",
        note = "Use VerdictEngine for dead/alive decisions. This method is for backward compatibility only."
    )]
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
        use crate::analysis::roots::{ReachabilityAnalyzer, RootDetectionConfig, RootDetector};

        let config = RootDetectionConfig::default();
        let root_set = RootDetector::detect_roots(
            call_graph,
            &[], // files not needed for basic root detection
            &config,
        );
        let reachability = ReachabilityAnalyzer::compute_reachability(call_graph, &root_set);

        // 2. Function dead code analysis
        let mut dead_functions = Vec::new();
        let mut total_loc = 0;

        // Debug counters
        let mut skipped_entry = 0;
        let mut skipped_test = 0;
        let mut skipped_ml = 0;
        let mut skipped_whitelist = 0;
        let mut skipped_trait = 0;
        let mut skipped_generated = 0;
        let mut skipped_react = 0;
        let mut skipped_exported = 0;
        let mut skipped_binary = 0;
        let mut scored = 0;
        let mut low_confidence = 0;

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];

            // ⭐ NEW: Skip scoring if using verdict engine
            // The caller will provide pre-computed verdicts
            if self.use_verdict_engine {
                continue;
            }

            // Track why functions are skipped
            let entry_points = ["main", "async_main", "run", "start", "init", "setup"];
            if entry_points.contains(&func.name.as_str()) {
                skipped_entry += 1;
                continue;
            }

            if func.name.starts_with("test_")
                || func.name.starts_with("Test")
                || func.name.starts_with("bench_")
                || func.name.starts_with("Benchmark")
                || func.file.contains("/tests/")
                || func.file.ends_with("_test.rs")
                || func.file.ends_with("_test.go")
            {
                skipped_test += 1;
                continue;
            }

            // ML filter
            if self.use_ml {
                if let Some(model) = &self.ml_model {
                    use crate::analysis::training_data::{TrainingExample, TrainingLabel};
                    let example = TrainingExample {
                        function_name: func.name.clone(),
                        full_path: func.full_path.clone(),
                        file: func.file.clone(),
                        language: TrainingExample::detect_language(&func.file),
                        features: crate::analysis::training_data::FunctionFeatures::from_function(
                            func, call_graph,
                        ),
                        label: TrainingLabel::Unknown,
                        confidence: 0.0,
                        source: "ml".to_string(),
                        repository_id: None,
                        commit_hash: None,
                        dataset_split: None,
                        label_reason: Some("ml".to_string()),
                        label_version: Some(1),
                    };
                    let prob = model.predict_probability(&example);
                    if prob > 0.85 {
                        skipped_ml += 1;
                        continue;
                    }
                }
            }

            // Whitelist
            if WHITELIST.is_whitelisted(&func.name) || WHITELIST.is_whitelisted_path(&func.file) {
                skipped_whitelist += 1;
                continue;
            }

            // Trait implementations - only skip standard ones
            if let Some(trait_name) = &func.trait_impl {
                let standard_traits = [
                    "Clone", "Debug", "Default", "Display", "From", "Into", "TryFrom",
                ];
                if standard_traits.contains(&trait_name.as_str()) {
                    skipped_trait += 1;
                    continue;
                }
            }

            // Generated files
            if Self::is_generated_file(func) {
                skipped_generated += 1;
                continue;
            }

            // React components (not relevant for Rust)
            if Self::is_likely_react_code(func) {
                skipped_react += 1;
                continue;
            }

            // ⭐ NEW: Skip binary-only functions
            if self.is_binary_only_function(func) {
                skipped_binary += 1;
                continue;
            }

            // Don't skip exported functions automatically
            // Score them and let the confidence threshold decide
            if func.is_public && func.fan_in == 0 {
                if func.file.contains("lib.rs") || func.file.contains("mod.rs") {
                    if func.doc_comment.is_some() {
                        skipped_exported += 1;
                        continue;
                    }
                    let export_patterns = ["new", "default", "from", "into", "try_from"];
                    if export_patterns.contains(&func.name.as_str()) {
                        skipped_exported += 1;
                        continue;
                    }
                }
            }

            // Score the function
            let git_info =
                git_analysis.and_then(|g| g.files.get(&std::path::PathBuf::from(&func.file)));
            let score = self.scorer.score_function(func, git_info);

            // Check if this is a binary-only function (double-check)
            let is_binary_only = self.is_binary_only_function(func);

            // ⭐ NEW: Check if this function might be an internal call
            // This is a heuristic: if the function has no callers but is in a file
            // with other functions, it might be called internally
            let is_internal_call =
                func.fan_in == 0 && !is_binary_only && !func.is_public && func.file.contains(".rs");

            // Only consider if confidence is high enough and it's not binary-only
            if matches!(
                score.level,
                ConfidenceLevel::Probably
                    | ConfidenceLevel::VeryLikely
                    | ConfidenceLevel::Guaranteed
            ) && !is_binary_only
            {
                let impact = self.calculate_impact(func, call_graph);
                total_loc += impact.lines_of_code;

                dead_functions.push(DeadFunction {
                    full_path: func.full_path.clone(),
                    name: func.name.clone(),
                    file: func.file.clone(),
                    line: func.line,
                    score,
                    impact,
                    removal_order: 0,
                    is_binary_only: false,
                    is_internal_call,
                });
                scored += 1;
            } else if is_binary_only {
                skipped_binary += 1;
            } else {
                low_confidence += 1;
            }
        }

        // Print debug info
        println!("\n🔍 Dead Code Analyzer Debug:");
        println!("   Total functions: {}", call_graph.node_count());
        println!("   Skipped (entry points): {}", skipped_entry);
        println!("   Skipped (test functions): {}", skipped_test);
        println!("   Skipped (ML model): {}", skipped_ml);
        println!("   Skipped (whitelist): {}", skipped_whitelist);
        println!("   Skipped (trait impls): {}", skipped_trait);
        println!("   Skipped (generated): {}", skipped_generated);
        println!("   Skipped (React): {}", skipped_react);
        println!("   Skipped (exported API): {}", skipped_exported);
        println!("   Skipped (binary-only): {}", skipped_binary);
        println!("   Scored (high confidence): {}", scored);
        println!("   Scored (low confidence): {}", low_confidence);

        // 3. Type dead code analysis
        let type_report = TypeDeadCodeDetector::detect_dead_types(type_graph, call_graph);

        // 4. Module dead code analysis
        let module_report = ModuleDeadCodeDetector::detect_dead_modules(import_graph);

        // 5. Sort by priority and assign removal order
        dead_functions.sort_by(|a, b| b.score.score.total_cmp(&a.score.score));
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
        let idx = call_graph.name_index.get(&func.full_path).copied();
        let mut dependencies = Vec::new();
        let mut complexity = func.complexity;

        if let Some(idx) = idx {
            // Get callees (what this function calls)
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

        // Impact is now based on removal risk (size + complexity), not caller
        // count. Caller count is a poor signal here: functions that reach this
        // method are already confidently dead, which by definition means they
        // have ~0 callers — so the old caller-count logic collapsed almost
        // everything into "Low impact" regardless of how much code or
        // complexity was actually being removed.
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

        // Show binary-only and internal call counts
        let binary_only = analysis
            .functions
            .iter()
            .filter(|f| f.is_binary_only)
            .count();
        let internal_calls = analysis
            .functions
            .iter()
            .filter(|f| f.is_internal_call)
            .count();
        let truly_dead = analysis.functions.len() - binary_only - internal_calls;

        if binary_only > 0 {
            report.push_str(&format!(
                "   ⚠️ {} functions are binary-only (used in CLI tools)\n",
                binary_only
            ));
        }
        if internal_calls > 0 {
            report.push_str(&format!(
                "   ⚠️ {} functions are internal calls (may be false positives)\n",
                internal_calls
            ));
        }
        if truly_dead > 0 {
            report.push_str(&format!(
                "   ✅ {} functions are truly dead\n\n",
                truly_dead
            ));
        }

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
                let label = if level == ConfidenceLevel::Guaranteed {
                    "✅ Guaranteed"
                } else if level == ConfidenceLevel::VeryLikely {
                    "🔶 Very Likely"
                } else {
                    "🔷 Probably"
                };
                report.push_str(&format!("\n{} ({})\n", label, functions.len()));
                for func in functions {
                    let status = if func.is_binary_only {
                        " [BINARY-ONLY]"
                    } else if func.is_internal_call {
                        " [INTERNAL CALL]"
                    } else {
                        ""
                    };
                    report.push_str(&format!(
                        "  #{} - {}{} ({}@{})\n",
                        func.removal_order, func.name, status, func.file, func.line
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

        if dead_func.is_binary_only {
            steps.push(
                "2. ⚠️ This function is binary-only — check if the binary is still needed"
                    .to_string(),
            );
        }

        if dead_func.is_internal_call {
            steps.push(
                "2. ⚠️ This may be an internal call — check if it's called elsewhere".to_string(),
            );
        }

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
