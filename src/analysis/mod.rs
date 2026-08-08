// src/analysis/mod.rs

pub mod complexity;
pub mod context;
pub mod dead_code;
pub mod features;
pub mod git_analysis;
pub mod importance;
pub mod roots;
pub mod training_data;
pub mod verdict;

pub use context::AnalysisIndexes;
pub use context::ProjectAnalysis;
pub use context::ProjectAnalysisBuilder;
pub use context::ProjectMetrics;
pub use dead_code::DeadCodeDetector;
pub use features::FeatureExtractor;
pub use features::FunctionFeatures;
pub use roots::{
    ReachabilityAnalyzer, ReachabilityMap, RootDetectionConfig, RootDetector, RootSet,
};
pub use training_data::TrainingDataCollector;
pub use training_data::TrainingExample;
pub use training_data::TrainingLabel;
pub use verdict::{Signal, SignalDirection, Verdict, VerdictConfig, VerdictEngine, VerdictStats};
