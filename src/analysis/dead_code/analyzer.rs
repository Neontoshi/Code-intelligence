// src/analysis/dead_code/analyzer.rs

use crate::analysis::git_analysis::GitAnalysis;
use crate::graph::call_graph::{CallGraph, FunctionNode};
use crate::graph::dependency_graph::DependencyGraph;
use crate::graph::import_graph::ImportGraph;
use crate::graph::type_graph::TypeGraph;
use crate::parser::tree_sitter::ParsedFile;

use super::modules::{DeadModuleReport, ModuleDeadCodeDetector};
use super::reachability::{ReachabilityAnalyzer, ReachabilityReport};
use super::scorer::{ConfidenceLevel, ConfidenceScorer, DeadScore};
use super::types::{DeadTypeReport, TypeDeadCodeDetector};
use super::whitelist::WHITELIST;
use crate::graph::traits::GraphMetrics;

#[cfg(feature = "ml")]
use crate::ml::classifier::DeadCodeClassifier;

use std::collections::{HashMap, HashSet};

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
    ml_model: Option<DeadCodeClassifier>,
    use_ml: bool,
}

impl DeadCodeAnalyzer {
    pub fn new() -> Self {
        Self {
            scorer: ConfidenceScorer::new(),
            cache: HashMap::new(),
            ml_model: None,
            use_ml: false,
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

    // ================================================================
    // UPDATED: Less aggressive exclusion
    // ================================================================

    fn is_excluded_function(&self, func: &FunctionNode) -> bool {
        // ============================================================
        // 1️⃣ FIRST: Entry points — definitely alive
        // ============================================================
        let entry_points = ["main", "async_main", "run", "start", "init", "setup"];
        if entry_points.contains(&func.name.as_str()) {
            return true;
        }

        // ============================================================
        // 2️⃣ SECOND: Test functions — called by test runner
        // ============================================================
        if func.name.starts_with("test_")
            || func.name.starts_with("Test")
            || func.name.starts_with("bench_")
            || func.name.starts_with("Benchmark")
            || func.file.contains("/tests/")
            || func.file.ends_with("_test.rs")
            || func.file.ends_with("_test.go")
        {
            return true;
        }

        // ============================================================
        // 3️⃣ THIRD: ML Model (if available)
        // ============================================================
        if self.use_ml {
            if let Some(model) = &self.ml_model {
                use crate::analysis::training_data::{TrainingExample, TrainingLabel};
                use crate::graph::call_graph::CallGraph;

                let example = TrainingExample {
                    function_name: func.name.clone(),
                    full_path: func.full_path.clone(),
                    file: func.file.clone(),
                    language: TrainingExample::detect_language(&func.file),
                    features: crate::analysis::training_data::FunctionFeatures::from_function(
                        func,
                        &CallGraph::new(),
                    ),
                    label: TrainingLabel::Unknown,
                    confidence: 0.0,
                    source: "ml".to_string(),
                };

                let prob = model.predict_probability(&example);

                // High confidence ALIVE → skip
                if prob > 0.85 {
                    return true;
                }

                // High confidence DEAD → don't skip
                if prob < 0.15 {
                    return false;
                }

                // If uncertain (0.15-0.85), fall through to other checks
            }
        }

        // ============================================================
        // 4️⃣ FOURTH: Whitelist (fallback)
        // ============================================================
        if WHITELIST.is_whitelisted(&func.name) {
            return true;
        }
        if WHITELIST.is_whitelisted_path(&func.file) {
            return true;
        }

        // ============================================================
        // 5️⃣ FIFTH: Trait implementations — only skip if actually used
        // ============================================================
        if let Some(trait_name) = &func.trait_impl {
            // We need to check if this trait is actually used anywhere
            // For now, be conservative: don't skip trait impls
            // They might be used polymorphically
            // return true; // OLD: always skip
            // NEW: only skip if it's a standard trait
            let standard_traits = [
                "Clone", "Debug", "Default", "Display", "From", "Into", "TryFrom",
            ];
            if standard_traits.contains(&trait_name.as_str()) {
                return true;
            }
            // For custom traits, we still report them as potentially dead
            // They'll be scored and filtered by confidence
        }

        // ============================================================
        // 6️⃣ SIXTH: Generated files
        // ============================================================
        if Self::is_generated_file(func) {
            return true;
        }

        // ============================================================
        // 7️⃣ SEVENTH: React components (not relevant for Rust)
        // ============================================================
        if Self::is_likely_react_code(func) {
            return true;
        }

        // ============================================================
        // 8️⃣ EIGHTH: Bundled JS (not relevant for Rust)
        // ============================================================
        if Self::is_bundled_js(func) {
            return true;
        }

        // ============================================================
        // 9️⃣ NINTH: Library exports — don't automatically skip
        // ============================================================
        // OLD: if func.is_public && func.fan_in == 0 { return true; }
        // NEW: Only skip if it's in a lib.rs or mod.rs AND has documentation
        if func.is_public && func.fan_in == 0 {
            // Check if it's a library root
            if func.file.contains("lib.rs") || func.file.contains("mod.rs") {
                // If it has documentation, it's probably intentional
                if func.doc_comment.is_some() {
                    return true;
                }
                // If it's a standard library export pattern
                let export_patterns = ["new", "default", "from", "into", "try_from"];
                if export_patterns.contains(&func.name.as_str()) {
                    return true;
                }
            }
            // Otherwise, we don't skip it — let the scorer decide
        }

        // ============================================================
        // 🔟 TENTH: Functions with callers — don't automatically skip
        // ============================================================
        // OLD: if func.fan_in > 0 { return true; }
        // NEW: Only skip if the callers are reachable from entry points
        // For now, we'll let the scorer handle it

        // If we made it here, the function is a candidate for dead code
        false
    }

    // ================================================================
    // Helper methods (unchanged from original)
    // ================================================================

    fn is_bundled_js(func: &FunctionNode) -> bool {
        if !func.file.ends_with(".js") && !func.file.ends_with(".js.map") {
            return false;
        }
        func.file.contains("/dist/")
            || func.file.contains("/build/")
            || func.file.contains("/assets/")
            || func.file.ends_with(".min.js")
            || func.file.contains("browser-")
            || func.file.contains("main-")
            || func.file.contains("index-")
            || func.file.contains("chunk-")
            || func.file.contains("node_modules/")
    }

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

    fn is_exported_but_unused(&self, func: &FunctionNode, import_graph: &ImportGraph) -> bool {
        let is_exported = func.file.contains("export") || func.file.contains("pub fn");
        let is_imported = import_graph.get_importers(&func.full_path).len() > 0;
        is_exported && !is_imported
    }

    fn get_cache_key(call_graph: &CallGraph) -> String {
        format!("cg_{}", call_graph.node_count())
    }

    // ================================================================
    // MAIN ANALYSIS METHOD — Updated with debug output
    // ================================================================

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

        // Debug counters
        let mut skipped_entry = 0;
        let mut skipped_test = 0;
        let mut skipped_ml = 0;
        let mut skipped_whitelist = 0;
        let mut skipped_trait = 0;
        let mut skipped_generated = 0;
        let mut skipped_react = 0;
        let mut skipped_exported = 0;
        let mut scored = 0;
        let mut low_confidence = 0;

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];

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

            // ⭐ NEW: Don't skip exported functions automatically
            // Instead, score them and let the confidence threshold decide
            if func.is_public && func.fan_in == 0 {
                // Check if it's a library root with documentation
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

            // ⭐ NEW: Don't skip functions with callers automatically
            // They might be dead if all their callers are dead

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
                });
                scored += 1;
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
        println!("   Scored (high confidence): {}", scored);
        println!("   Scored (low confidence): {}", low_confidence);

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
