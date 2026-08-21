## Document 4: `docs/api.md`

```markdown
# API Documentation

## Overview

This document describes the public API of `code-intelligence`. The API is designed to be used programmatically for integrating dead code detection into other tools and workflows.

---

## Core Types

### ProjectAnalysis

The main analysis result containing all extracted information.

```rust
pub struct ProjectAnalysis {
    pub root: PathBuf,
    pub files: Arc<Vec<ParsedFile>>,
    pub project_graph: Arc<ProjectGraph>,
    pub call_graph: Arc<CallGraph>,
    pub type_graph: Arc<TypeGraph>,
    pub import_graph: Arc<ImportGraph>,
    pub dependency_graph: Arc<DependencyGraph>,
    pub indexes: Arc<AnalysisIndexes>,
    pub rich_indexes: Arc<RichIndexes>,
    pub metrics: Arc<ProjectMetrics>,
    pub features: Arc<FeatureExtractor>,
    pub cache: Arc<FileCache>,
    pub llm_analysis: Option<LLMAnalysis>,
    pub created_at: DateTime<Utc>,
    pub version: u32,
}
```

### Methods

```rust
impl ProjectAnalysis {
    /// Get a function by its full path
    pub fn get_function(&self, full_path: &str) -> Option<&FunctionNode>;

    /// Get all functions with a given name
    pub fn get_functions_by_name(&self, name: &str) -> Vec<&FunctionNode>;

    /// Get all functions in a file
    pub fn get_functions_by_file(&self, file: &str) -> Vec<&FunctionNode>;

    /// Get all function names
    pub fn function_names(&self) -> Vec<String>;

    /// Get all file paths
    pub fn file_paths(&self) -> Vec<String>;

    /// Check if a function exists
    pub fn has_function(&self, full_path: &str) -> bool;

    /// Get total function count
    pub fn function_count(&self) -> usize;

    /// Get total file count
    pub fn file_count(&self) -> usize;

    /// Get call edge count
    pub fn call_edge_count(&self) -> usize;

    /// Generate markdown report
    pub fn to_markdown(&self) -> String;

    /// Generate JSON report
    pub fn to_json(&self) -> String;

    /// Generate training data JSON
    pub fn to_training_json(&self) -> String;

    /// Generate GraphViz DOT format
    pub fn to_graphviz(&self) -> String;

    /// Generate full report with LLM analysis
    pub fn to_full_report(&self) -> String;
}
```

---

### CallGraph

Represents the call graph of the project.

```rust
pub struct CallGraph {
    pub graph: DiGraph<FunctionNode, CallEdge>,
    pub name_index: HashMap<String, NodeIndex>,
    pub name_to_functions: HashMap<String, Vec<NodeIndex>>,
    pub file_to_functions: HashMap<String, Vec<NodeIndex>>,
    pub public_functions: Vec<NodeIndex>,
    pub async_functions: Vec<NodeIndex>,
    pub duplicate_functions: Vec<String>,
    pub resolution_cache: HashMap<String, ResolutionConfidence>,
    pub unresolved_calls: HashMap<String, Vec<String>>,
}
```

### Methods

```rust
impl CallGraph {
    /// Create a new empty call graph
    pub fn new() -> Self;

    /// Add a function to the graph
    pub fn add_function(&mut self, func: FunctionNode) -> NodeIndex;

    /// Add a call edge between functions
    pub fn add_call(&mut self, caller: NodeIndex, callee: NodeIndex, edge: CallEdge);

    /// Get all functions with a given name
    pub fn get_functions_by_name(&self, name: &str) -> Vec<&FunctionNode>;

    /// Get all functions in a file
    pub fn get_functions_by_file(&self, file: &str) -> Vec<&FunctionNode>;

    /// Get all public functions
    pub fn get_public_functions(&self) -> Vec<&FunctionNode>;

    /// Get all async functions
    pub fn get_async_functions(&self) -> Vec<&FunctionNode>;

    /// Get callees of a function
    pub fn get_callees(&self, func: NodeIndex) -> Vec<&FunctionNode>;

    /// Get callers of a function
    pub fn get_callers(&self, func: NodeIndex) -> Vec<&FunctionNode>;

    /// Get resolution confidence for a call
    pub fn get_resolution_confidence(&self, caller: &str, callee: &str) -> ResolutionConfidence;

    /// Get unresolved calls for a function
    pub fn get_unresolved_calls(&self, full_path: &str) -> Vec<&str>;

    /// Get resolution statistics
    pub fn resolution_stats(&self) -> ResolutionStats;

    /// Generate DOT format
    pub fn to_dot(&self) -> String;

    /// Calculate fan-in/fan-out metrics
    pub fn calculate_fan_metrics(&mut self);

    /// Detect and mark cycle members
    pub fn mark_cycle_members(&mut self);

    /// Detect layers from file paths
    pub fn detect_layers(&mut self);

    /// Calculate call depth from entry points
    pub fn calculate_call_depth(&mut self);

    /// Get top important nodes
    pub fn top_important_nodes(&self, max_nodes: usize, min_importance: f64) -> Vec<NodeIndex>;
}
```

---

### FunctionNode

Represents a single function in the call graph.

```rust
pub struct FunctionNode {
    pub name: String,
    pub full_path: String,
    pub file: String,
    pub line: usize,
    pub body_start_line: usize,
    pub body_end_line: usize,
    pub is_public: bool,
    pub is_async: bool,
    pub params: Vec<String>,
    pub returns: Vec<String>,
    pub complexity: f64,
    pub importance_score: f64,
    pub doc_comment: Option<String>,
    pub writes_to: Vec<String>,
    pub reads_from: Vec<String>,
    pub errors: Vec<String>,
    pub fan_in: usize,
    pub fan_out: usize,
    pub is_cycle: bool,
    pub depth: usize,
    pub layer: String,
    pub trait_impl: Option<String>,
    pub is_test: bool,
    pub is_trait_method: bool,
    pub is_trait_default: bool,
}
```

---

### Verdict

Represents a dead code verdict for a function.

```rust
pub struct Verdict {
    pub function_name: String,
    pub full_path: String,
    pub label: TrainingLabel,
    pub state: VerdictState,
    pub confidence: f64,
    pub signals: Vec<Signal>,
    pub ml_probability: Option<f64>,
    pub static_score: Option<f64>,
    pub explanation: String,
    pub evidence_sources: Vec<EvidenceSource>,
    pub verified: bool,
    pub verified_by: Option<String>,
}
```

### Methods

```rust
impl Verdict {
    /// Check if the function is dead
    pub fn is_dead(&self) -> bool;

    /// Check if the function is alive
    pub fn is_alive(&self) -> bool;

    /// Check if the function needs review
    pub fn needs_review(&self) -> bool;

    /// Check if high confidence
    pub fn is_high_confidence(&self) -> bool;

    /// Mark as verified by a user
    pub fn mark_verified(&mut self, verified_by: &str);

    /// Format the verdict state
    pub fn format_state(&self) -> String;

    /// Format a full explanation
    pub fn format_explanation(&self) -> String;
}
```

---

### VerdictEngine

Engine that evaluates functions and produces verdicts.

```rust
pub struct VerdictEngine {
    config: VerdictConfig,
    ml_model: Option<DeadCodeClassifier>,
    dynamic_refs: Option<Vec<DynamicReference>>,
}
```

### Methods

```rust
impl VerdictEngine {
    /// Create a new verdict engine
    pub fn new(config: VerdictConfig) -> Self;

    /// Set the ML model
    pub fn with_ml(mut self, model: DeadCodeClassifier) -> Self;

    /// Set dynamic references
    pub fn with_dynamic_refs(mut self, refs: Vec<DynamicReference>) -> Self;

    /// Set dead threshold
    pub fn with_dead_threshold(mut self, threshold: f64) -> Self;

    /// Set alive threshold
    pub fn with_alive_threshold(mut self, threshold: f64) -> Self;

    /// Evaluate a single function
    pub fn evaluate_function(
        &self,
        func: &FunctionNode,
        call_graph: &CallGraph,
        reachability: &ReachabilityMap,
    ) -> Verdict;

    /// Evaluate all functions
    pub fn evaluate_all(
        &self,
        call_graph: &CallGraph,
        reachability: &ReachabilityMap,
    ) -> Vec<Verdict>;

    /// Filter dead verdicts
    pub fn filter_dead<'a>(&self, verdicts: &'a [Verdict]) -> Vec<&'a Verdict>;

    /// Filter alive verdicts
    pub fn filter_alive<'a>(&self, verdicts: &'a [Verdict]) -> Vec<&'a Verdict>;

    /// Filter unknown verdicts
    pub fn filter_unknown<'a>(&self, verdicts: &'a [Verdict]) -> Vec<&'a Verdict>;

    /// Get statistics
    pub fn stats(&self, verdicts: &[Verdict]) -> VerdictStats;
}
```

---

### Pipeline

Main pipeline for processing projects.

```rust
pub struct Pipeline {
    parser: TreeSitterParser,
    scorer: ImportanceScorer,
    cache: FileCache,
    config: PipelineConfig,
    llm_provider: Option<Arc<dyn LLMProvider>>,
    code_understanding: Option<CodeUnderstandingEngine>,
    analysis_cache: Option<AnalysisCacheManager>,
    progress: Option<ProgressFn>,
    logger: Option<Mutex<StructuredLogger>>,
    file_tracker: Option<FileTracker>,
    enable_incremental: bool,
}
```

### Methods

```rust
impl Pipeline {
    /// Create a new pipeline
    pub fn new() -> Self;

    /// Set configuration
    pub fn with_config(mut self, config: PipelineConfig) -> Self;

    /// Enable LLM analysis
    pub fn with_llm(mut self, provider: Arc<dyn LLMProvider>) -> Self;

    /// Enable Git analysis
    pub fn enable_git(mut self) -> Self;

    /// Enable disk cache
    pub fn with_cache_dir(mut self, cache_dir: PathBuf) -> Self;

    /// Enable incremental analysis
    pub fn enable_incremental(mut self) -> Self;

    /// Set progress reporter
    pub fn with_progress_reporter(mut self, f: ProgressFn) -> Self;

    /// Set logging
    pub fn with_logging(mut self, logger: StructuredLogger) -> Self;

    /// Use Ollama phi-2
    pub async fn with_ollama_phi2(mut self) -> Result<Self, String>;

    /// Process a project
    pub async fn process_project(&mut self, root: &Path) -> Result<ProjectAnalysis, Box<dyn Error>>;

    /// Process a project with Git analysis
    pub async fn process_project_with_git(&mut self, root: &Path) -> Result<ProjectAnalysis, Box<dyn Error>>;

    /// Check memory usage
    pub fn check_memory(&self) -> Result<(), String>;

    /// Get current memory usage
    pub fn get_current_memory_usage_mb(&self) -> f64;

    /// Take build summary
    pub fn take_build_summary(&mut self) -> Option<BuildSummary>;
}
```

---

### DeadCodeDetector

High-level dead code detection API.

```rust
pub struct DeadCodeDetector;
```

### Methods

```rust
impl DeadCodeDetector {
    /// Get dead code statistics
    pub fn get_dead_stats(call_graph: &CallGraph, files: &[ParsedFile]) -> DeadStats;

    /// Find dead modules
    pub fn find_dead_modules(files: &[ParsedFile]) -> Vec<String>;

    /// Full analysis
    pub fn analyze(
        call_graph: &CallGraph,
        type_graph: &TypeGraph,
        import_graph: &ImportGraph,
        dependency_graph: &DependencyGraph,
        files: &[ParsedFile],
        git_analysis: Option<&GitAnalysis>,
    ) -> DeadCodeAnalysis;

    /// Generate report
    pub fn generate_report(analysis: &DeadCodeAnalysis) -> String;

    /// Calculate dead code ratio
    pub fn dead_code_ratio(call_graph: &CallGraph, files: &[ParsedFile]) -> f64;
}
```

---

### DeadCodeClassifier

ML model for dead code prediction.

```rust
pub struct DeadCodeClassifier {
    pub model: Option<LinearClassifier>,
    pub accuracy: f64,
    pub feature_count: usize,
    pub calibration: Option<CalibrationParams>,
}
```

### Methods

```rust
impl DeadCodeClassifier {
    /// Create a new classifier
    pub fn new() -> Self;

    /// Train the model
    pub fn train(&mut self, examples: &[TrainingExample]) -> Result<(), String>;

    /// Predict label
    pub fn predict(&self, example: &TrainingExample) -> TrainingLabel;

    /// Predict probability (alive)
    pub fn predict_probability(&self, example: &TrainingExample) -> f64;

    /// Predict probability (dead)
    pub fn predict_dead_probability(&self, example: &TrainingExample) -> f64;

    /// Predict from feature vector
    pub fn predict_features(&self, features: &[f64]) -> f64;

    /// Validate features
    pub fn validate_features(&self, features: &[f64]) -> Result<(), String>;

    /// Check schema compatibility
    pub fn is_schema_compatible(&self) -> bool;

    /// Get schema info
    pub fn schema_info(&self) -> String;

    /// Get accuracy
    pub fn get_accuracy(&self) -> f64;

    /// Check if trained
    pub fn is_trained(&self) -> bool;

    /// Get model reference
    pub fn get_model(&self) -> Option<&LinearClassifier>;

    /// Calibrate the model
    pub fn calibrate(&mut self, val_examples: &[TrainingExample]) -> Result<(), String>;

    /// Predict with calibration
    pub fn predict_calibrated(&self, features: &[f64]) -> f64;

    /// Predict alive calibrated
    pub fn predict_alive_calibrated(&self, example: &TrainingExample) -> f64;

    /// Predict dead calibrated
    pub fn predict_dead_calibrated(&self, example: &TrainingExample) -> f64;

    /// Print feature importance
    pub fn print_feature_importance(&self);

    /// Save model
    pub fn save(&self, path: &str) -> Result<(), String>;

    /// Load model
    pub fn load(path: &str) -> Result<Self, String>;
}
```

---

### DuplicateClassifier

ML model for duplicate code detection.

```rust
pub struct DuplicateClassifier {
    weights: Vec<f64>,
    bias: f64,
    threshold: f64,
    feature_count: usize,
}
```

### Methods

```rust
impl DuplicateClassifier {
    /// Create a new classifier
    pub fn new(feature_count: usize) -> Self;

    /// Train the model
    pub fn train(&mut self, examples: &[DuplicateExample]) -> f64;

    /// Predict probability
    pub fn predict(&self, a: &FunctionFeatures, b: &FunctionFeatures) -> f64;

    /// Check if duplicates
    pub fn is_duplicate(&self, a: &FunctionFeatures, b: &FunctionFeatures) -> bool;

    /// Evaluate on test data
    pub fn evaluate(&self, examples: &[DuplicateExample]) -> f64;

    /// Save model
    pub fn save(&self, path: &str) -> Result<(), String>;

    /// Load model
    pub fn load(path: &str) -> Result<Self, String>;
}
```

---

### OutcomeTracker

Track outcomes of dead code verdicts.

```rust
pub struct OutcomeTracker {
    verdicts: Vec<TrackedVerdict>,
    storage_path: PathBuf,
}
```

### Methods

```rust
impl OutcomeTracker {
    /// Create a new tracker
    pub fn new(project_root: &Path) -> Self;

    /// Track a new verdict
    pub fn track_verdict(
        &mut self,
        function_name: &str,
        full_path: &str,
        file: &str,
        line: usize,
        confidence: f64,
        project: &str,
    ) -> String;

    /// Update outcome
    pub fn update_outcome(
        &mut self,
        id: &str,
        outcome: VerdictOutcome,
        notes: Option<String>,
        removed_commit: Option<String>,
    ) -> Result<(), String>;

    /// Mark as removed
    pub fn mark_removed(&mut self, id: &str, commit_hash: Option<&str>) -> Result<(), String>;

    /// Mark as false positive
    pub fn mark_false_positive(&mut self, id: &str, reason: &str) -> Result<(), String>;

    /// Get all verdicts
    pub fn get_verdicts(&self) -> &[TrackedVerdict];

    /// Get pending verdicts
    pub fn get_pending(&self) -> Vec<&TrackedVerdict>;

    /// Get removed verdicts
    pub fn get_removed(&self) -> Vec<&TrackedVerdict>;

    /// Get kept verdicts
    pub fn get_kept(&self) -> Vec<&TrackedVerdict>;

    /// Get statistics
    pub fn get_stats(&self) -> OutcomeStats;

    /// Import verdicts from analysis
    pub fn import_verdicts(&mut self, dead_verdicts: &[&Verdict], project: &str) -> usize;

    /// Generate report
    pub fn generate_report(&self) -> String;
}
```

---

### ExplainabilityEngine

Generate explanations for verdicts.

```rust
pub struct ExplainabilityEngine;
```

### Methods

```rust
impl ExplainabilityEngine {
    /// Generate an explanation for a verdict
    pub fn generate_explanation(
        verdict: &Verdict,
        func: &FunctionNode,
        git_info: Option<&GitInfo>,
    ) -> VerdictExplanation;
}
```

---

## Types Reference

### Enums

#### TrainingLabel

```rust
pub enum TrainingLabel {
    Alive,
    Dead,
    Unknown,
}
```

#### VerdictState

```rust
pub enum VerdictState {
    DefinitelyAlive,
    ProbablyAlive,
    Unknown,
    ProbablyDead,
    DefinitelyDead,
}
```

#### LabelSource

```rust
pub enum LabelSource {
    StaticHeuristic,
    Silver,
    Weak,
    HumanVerified,
    GitVerified,
    ProductionVerified,
    DatasetVerified,
}
```

#### RiskLevel

```rust
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}
```

#### DuplicateType

```rust
pub enum DuplicateType {
    Exact,
    Structural,
    Algorithmic,
    Partial,
    FalsePositive,
}
```

---

## Usage Examples

### Basic Analysis

```rust
use code_intelligence::Pipeline;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut pipeline = Pipeline::new();
    let analysis = pipeline.process_project(Path::new("./my-project")).await?;
    
    println!("Found {} functions", analysis.function_count());
    println!("Dead code ratio: {:.1}%", 
        DeadCodeDetector::dead_code_ratio(&analysis.call_graph, &analysis.files) * 100.0
    );
    
    Ok(())
}
```

### With ML Model

```rust
use code_intelligence::{Pipeline, ml::DeadCodeClassifier};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = DeadCodeClassifier::load("models/dead_code_model_v2.bin")?;
    
    let mut pipeline = Pipeline::new();
    let analysis = pipeline.process_project(Path::new("./my-project")).await?;
    
    // ... use model with analysis ...
    
    Ok(())
}
```

### With LLM Analysis

```rust
use code_intelligence::{
    Pipeline,
    llm::{create_ollama_phi2, LLMProvider},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = create_ollama_phi2().await?;
    let mut pipeline = Pipeline::new().with_llm(provider);
    
    let analysis = pipeline.process_project(Path::new("./my-project")).await?;
    
    if let Some(llm_analysis) = analysis.llm_analysis {
        println!("LLM Documentation: {}", llm_analysis.documentation.unwrap_or_default());
    }
    
    Ok(())
}
```

---

## Error Handling

All API functions return `Result<T, CodeIntelError>`:

```rust
pub enum CodeIntelError {
    ParseError { path: PathBuf, source: Box<dyn Error> },
    GraphError { message: String },
    ModelError { message: String },
    DatasetError { message: String },
    CacheError { message: String },
    ConfigError { message: String },
    GitError { message: String },
    IoError { source: io::Error },
    AnalysisError { message: String },
    AnalysisTimeout { duration: u64 },
    AnalysisCancelled,
    MemoryLimitExceeded { limit: usize },
    SerializationError { source: serde_json::Error },
    LlmError { message: String },
    FeatureError { message: String },
    TrainingError { message: String },
    InternalError { message: String },
    Unreachable { message: String },
    NotImplemented { feature: String },
}
```

---

## Versioning

The API follows semantic versioning. Breaking changes will result in a major version bump.

**Current Version**: 0.2.0
```

---
