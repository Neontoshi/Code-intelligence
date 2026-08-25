// src/analysis/service.rs

use crate::analysis::dead_code::{DeadCodeAnalysis, DeadCodeAnalyzer};
use crate::analysis::dynamic_refs::{DynamicRefDetector, DynamicReference};
use crate::analysis::roots::{
    ReachabilityAnalyzer, ReachabilityMap, RootDetectionConfig, RootDetector, RootSet,
};
use crate::analysis::verdict_source::state::{Verdict, VerdictConfig, VerdictEngine};
use crate::error::{err, Result};
use crate::graph::call_graph::CallGraph;
use crate::ml::classifier::DeadCodeClassifier;
use crate::parser::tree_sitter::ParsedFile;
use crate::Pipeline;
use std::path::PathBuf;
use std::sync::Arc;

/// Configuration for the analysis service
#[derive(Debug, Clone)]
pub struct AnalysisServiceConfig {
    /// Path to ML model file (optional)
    pub model_path: Option<PathBuf>,
    /// Confidence threshold (overrides model default)
    pub threshold: Option<f64>,
    /// Enable verbose output
    pub verbose: bool,
    /// Enable debug mode
    pub debug: bool,
    /// Enable disk cache
    pub cache: bool,
    /// Cache directory
    pub cache_dir: Option<PathBuf>,
    /// Enable LLM analysis
    pub llm: bool,
    /// Enable Git analysis
    pub git: bool,
}

impl Default for AnalysisServiceConfig {
    fn default() -> Self {
        Self {
            model_path: None,
            threshold: None,
            verbose: false,
            debug: false,
            cache: false,
            cache_dir: None,
            llm: false,
            git: false,
        }
    }
}

/// Result of the analysis service
#[derive(Debug, Clone)]
pub struct AnalysisServiceResult {
    /// The full project analysis
    pub project_analysis: crate::analysis::context::ProjectAnalysis,
    /// The call graph
    pub call_graph: Arc<CallGraph>,
    /// The parsed files
    pub files: Arc<Vec<ParsedFile>>,
    /// The root set
    pub root_set: RootSet,
    /// The reachability map
    pub reachability: ReachabilityMap,
    /// Dynamic references found
    pub dynamic_refs: Vec<DynamicReference>,
    /// Verdicts for all functions
    pub verdicts: Vec<Verdict>,
    /// Dead verdicts (filtered)
    pub dead_verdicts: Vec<Verdict>,
    /// Alive verdicts (filtered)
    pub alive_verdicts: Vec<Verdict>,
    /// Unknown verdicts (filtered)
    pub unknown_verdicts: Vec<Verdict>,
    /// Dead code analysis
    pub dead_code_analysis: DeadCodeAnalysis,
    /// The verdict engine used
    pub verdict_engine: VerdictEngine,
    /// The ML model used (if any)
    pub ml_model: Option<DeadCodeClassifier>,
    /// The effective threshold used
    pub effective_threshold: f64,
}

/// Shared analysis service
pub struct AnalysisService {
    config: AnalysisServiceConfig,
    pipeline: Pipeline,
    ml_model: Option<DeadCodeClassifier>,
    verdict_engine: Option<VerdictEngine>,
}

impl AnalysisService {
    /// Create a new analysis service
    pub fn new(config: AnalysisServiceConfig) -> Self {
        let mut pipeline = Pipeline::new();

        if config.cache {
            let cache_path = config
                .cache_dir
                .clone()
                .unwrap_or_else(|| PathBuf::from(".code-intelligence-cache"));
            pipeline = pipeline.with_cache_dir(cache_path);
        }

        if config.llm {
            // LLM setup would go here
        }

        if config.git {
            pipeline = pipeline.enable_git();
        }

        Self {
            config,
            pipeline,
            ml_model: None,
            verdict_engine: None,
        }
    }

    /// Load the ML model if configured, or fall back to the embedded binary model
    pub fn load_model(&mut self) -> Result<()> {
        if let Some(model_path) = &self.config.model_path {
            if !model_path.exists() {
                return Err(err::model(format!(
                    "Model file not found: {:?}",
                    model_path
                )));
            }

            // Try loading versioned model first
            match crate::ml::model_serialization::VersionedModel::load(
                &model_path.to_string_lossy(),
            ) {
                Ok(versioned) => {
                    let classifier = DeadCodeClassifier {
                        model: Some(versioned.classifier.clone()),
                        accuracy: versioned
                            .performance
                            .as_ref()
                            .map(|p| p.accuracy)
                            .unwrap_or(0.0),
                        feature_count: versioned.feature_schema.feature_count(),
                        calibration: versioned.calibration.clone(),
                    };
                    self.ml_model = Some(classifier);
                    if self.config.verbose {
                        println!("✅ Loaded versioned model from: {:?}", model_path);
                        println!("   Version: {}", versioned.version);
                        if let Some(perf) = versioned.get_performance() {
                            println!(
                                "   F1: {:.1}%, Precision: {:.1}%",
                                perf.f1 * 100.0,
                                perf.precision * 100.0
                            );
                        }
                    }
                    Ok(())
                }
                Err(_) => {
                    // Try legacy model
                    match DeadCodeClassifier::load(model_path) {
                        Ok(classifier) => {
                            self.ml_model = Some(classifier);
                            if self.config.verbose {
                                println!("✅ Loaded legacy model from: {:?}", model_path);
                            }
                            Ok(())
                        }
                        Err(e) => Err(err::model(format!("Failed to load model: {}", e))),
                    }
                }
            }
        } else {
            // Fall back to the model compiled directly into the binary
            match DeadCodeClassifier::load_embedded() {
                Ok(classifier) => {
                    self.ml_model = Some(classifier);
                    if self.config.verbose {
                        println!("🧠 Using built-in embedded dead code model");
                    }
                    Ok(())
                }
                Err(e) => {
                    if self.config.verbose {
                        eprintln!("⚠️ Warning: Failed to load embedded model: {}", e);
                    }
                    Ok(())
                }
            }
        }
    }

    /// Get the effective threshold
    pub fn get_effective_threshold(&self) -> f64 {
        if let Some(threshold) = self.config.threshold {
            return threshold;
        }

        if let Some(model) = &self.ml_model {
            // Try to get threshold from model calibration
            if let Some(cal) = &model.calibration {
                if cal.temperature != 1.0 {
                    // Use temperature-adjusted threshold
                    let base = 0.92;
                    return (base / cal.temperature).clamp(0.5, 0.95);
                }
            }
        }

        0.92 // Default
    }

    /// Build the verdict engine
    pub fn build_verdict_engine(&mut self, dynamic_refs: Vec<DynamicReference>) -> VerdictEngine {
        let effective_threshold = self.get_effective_threshold();

        let mut config = VerdictConfig::default();
        config.enable_ml = self.ml_model.is_some();

        let mut engine = VerdictEngine::new(config)
            .with_dead_threshold(effective_threshold)
            .with_dynamic_refs(dynamic_refs);
        if let Some(model_path) = &self.config.model_path {
            engine = engine.with_model_path(&model_path.to_string_lossy());
        }
        engine = engine.with_stage("service_analysis");

        if let Some(model) = &self.ml_model {
            engine = engine.with_ml(model.clone());
        }
        self.verdict_engine = Some(engine.clone());
        engine
    }

    /// Run the full analysis
    pub async fn analyze(&mut self, project_path: &PathBuf) -> Result<AnalysisServiceResult> {
        // 1. Load model (custom path or embedded default)
        self.load_model()?;

        // 2. Run pipeline
        let analysis = self
            .pipeline
            .process_project(project_path)
            .await
            .map_err(|e| err::analysis(format!("Pipeline failed: {}", e)))?;

        let call_graph = analysis.call_graph.clone();
        let files = analysis.files.clone();

        // 3. Detect roots
        let root_config = RootDetectionConfig::default();
        let root_set = RootDetector::detect_roots(&call_graph, &files, &root_config);

        // 4. Compute reachability
        let reachability = ReachabilityAnalyzer::compute_reachability(&call_graph, &root_set);

        // 5. Detect dynamic references
        let dynamic_detector = DynamicRefDetector::new();
        let dynamic_refs = dynamic_detector.detect_all(&call_graph, &files);

        // 6. Build verdict engine
        let engine = self.build_verdict_engine(dynamic_refs.clone());

        // 7. Evaluate all functions
        let verdicts = engine.evaluate_all(&call_graph, &reachability);
        let dead_verdicts: Vec<Verdict> =
            engine.filter_dead(&verdicts).into_iter().cloned().collect();
        let alive_verdicts: Vec<Verdict> = engine
            .filter_alive(&verdicts)
            .into_iter()
            .cloned()
            .collect();
        let unknown_verdicts: Vec<Verdict> = engine
            .filter_unknown(&verdicts)
            .into_iter()
            .cloned()
            .collect();

        // 8. Build dead code analysis
        let dead_verdict_refs: Vec<&Verdict> = dead_verdicts.iter().collect();
        let mut analyzer = DeadCodeAnalyzer::new();
        let dead_functions = analyzer.import_verdicts(&dead_verdict_refs, &call_graph);

        let dead_code_analysis = analyzer.analyze_structural_components(
            dead_functions,
            reachability.clone(),
            &call_graph,
            &analysis.type_graph,
            &analysis.import_graph,
        );

        Ok(AnalysisServiceResult {
            project_analysis: analysis,
            call_graph,
            files,
            root_set,
            reachability,
            dynamic_refs,
            verdicts,
            dead_verdicts,
            alive_verdicts,
            unknown_verdicts,
            dead_code_analysis,
            verdict_engine: engine,
            ml_model: self.ml_model.clone(),
            effective_threshold: self.get_effective_threshold(),
        })
    }

    /// Get the pipeline (for advanced use)
    pub fn pipeline(&self) -> &Pipeline {
        &self.pipeline
    }

    /// Get the pipeline mutably (for advanced use)
    pub fn pipeline_mut(&mut self) -> &mut Pipeline {
        &mut self.pipeline
    }
}

/// Helper to create a service with default config
pub fn create_analysis_service(_project_path: &PathBuf) -> AnalysisService {
    let config = AnalysisServiceConfig {
        model_path: None,
        threshold: None,
        verbose: false,
        debug: false,
        cache: false,
        cache_dir: None,
        llm: false,
        git: false,
    };
    AnalysisService::new(config)
}

/// Helper to create a service with model
pub fn create_analysis_service_with_model(
    _project_path: &PathBuf,
    model_path: PathBuf,
) -> AnalysisService {
    let config = AnalysisServiceConfig {
        model_path: Some(model_path),
        threshold: None,
        verbose: false,
        debug: false,
        cache: false,
        cache_dir: None,
        llm: false,
        git: false,
    };
    AnalysisService::new(config)
}
