// src/ml/mod.rs

//! Machine Learning module for code intelligence

pub mod classifier;
pub mod duplicate_classifier;
pub mod features;

// Re-export main types
pub use classifier::DeadCodeClassifier;
pub use duplicate_classifier::DuplicateClassifier;
pub use duplicate_classifier::DuplicateExample;
pub use duplicate_classifier::DuplicateLabel;
pub use features::FeatureExtractor;
