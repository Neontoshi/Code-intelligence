pub mod complexity;
pub mod context;
pub mod dead_code;
pub mod dynamic_refs;
pub mod explainability;
pub mod features;
pub mod git_analysis;
pub mod importance;
pub mod layers;
pub mod outcomes;
pub mod roots;
pub mod service;
pub mod training_data;
pub mod training_data_filter;
pub mod verdict_source;

pub use context::AnalysisIndexes;
pub use context::AnalysisMetadata;
pub use context::ProjectAnalysis;
pub use context::ProjectAnalysisBuilder;
pub use context::ProjectMetrics;
pub use dead_code::DeadCodeDetector;
pub use dynamic_refs::{DynamicRefDetector, DynamicRefType, DynamicReference};
pub use features::FeatureExtractor;
pub use features::FunctionFeatures;
pub use outcomes::{OutcomeStats, OutcomeTracker, TrackedVerdict, VerdictOutcome};
pub use roots::{
    ReachabilityAnalyzer, ReachabilityMap, RootDetectionConfig, RootDetector, RootSet,
};
pub use training_data::TrainingDataCollector;
pub use training_data::TrainingExample;
pub use training_data::TrainingLabel;
pub use verdict_source::state::{VerdictConfig, VerdictEngine, VerdictStats};
pub use verdict_source::{EvidenceSource, Signal, SignalDirection, Verdict, VerdictState};
