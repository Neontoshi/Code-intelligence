// src/ml/mod.rs

pub mod calibration;
pub mod classifier;
pub mod duplicate_classifier;
pub mod feature_schema;
pub mod features;
pub mod model_serialization;
pub mod serialization;

pub use classifier::DeadCodeClassifier;
pub use duplicate_classifier::{DuplicateClassifier, DuplicateExample, DuplicateLabel};
pub use feature_schema::{
    feature_count, feature_names, FeatureCategory, FeatureDefinition, FeatureSchema,
    FeatureVectorBuilder, Normalization, FEATURE_SCHEMA,
};
pub use features::FeatureExtractor;
pub use model_serialization::{ModelPerformance, TrainingMetadata, VersionedModel};
pub use serialization::{load_model, save_model};
