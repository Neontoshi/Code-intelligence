pub mod analysis;
pub mod engine;
pub mod graph;
pub mod llm;
pub mod ml;
pub mod optimize;
pub mod output;
pub mod parser;
pub mod utils;

// Re-export main types
pub use analysis::complexity::ComplexityAnalyzer;
pub use analysis::context::ProjectAnalysis;
pub use analysis::dead_code::DeadCodeDetector;
pub use analysis::dynamic_refs::{DynamicRefDetector, DynamicRefType, DynamicReference};
pub use analysis::git_analysis::GitAnalyzer;
pub use analysis::importance::ImportanceScorer;
pub use analysis::roots::{
    ReachabilityAnalyzer, ReachabilityMap, RootDetectionConfig, RootDetector, RootSet,
};
pub use analysis::verdict::{
    Signal, SignalDirection, Verdict, VerdictConfig, VerdictEngine, VerdictStats,
};
pub use engine::cache::FileCache;
pub use engine::indexer::IndexBuilder;
pub use engine::{
    AnalyzedProject, OptimizedProject, ParsedProject, Pipeline, PipelineConfig,
    ProjectIntelligence, RawProject,
};
pub use graph::call_graph::CallGraph;
pub use graph::call_graph::FunctionNode;
pub use graph::dependency_graph::DependencyGraph;
pub use graph::import_graph::ImportGraph;
pub use graph::type_graph::TypeGraph;
pub use ml::calibration::{CalibratedModel, CalibrationMethod, CalibrationStats};
pub use ml::classifier::DeadCodeClassifier;
pub use ml::model_serialization::{ModelPerformance, TrainingMetadata, VersionedModel};
pub use optimize::chunk::ChunkStrategy;
pub use optimize::dedup::Deduplicator;
pub use optimize::SemanticCompressor;
pub use optimize::TokenEstimator;
pub use output::graphviz::GraphVizOutput;
pub use output::json::JsonOutput;
pub use output::markdown::MarkdownOutput;
pub use output::rag::RAGGenerator;
pub use parser::comments::CommentAnalyzer;
pub use parser::semantic::SemanticAnalyzer;
pub use utils::hashing::HashUtils;
pub use utils::parallel::ParallelUtils;
