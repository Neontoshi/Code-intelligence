// src/ml/mod.rs

pub mod classifier;
pub mod features;

// Re-export main types
pub use classifier::DeadCodeClassifier;
pub use features::FeatureExtractor;
