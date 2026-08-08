// src/ml/model_serialization.rs

//! Versioned model serialization with schema tracking

use crate::ml::calibration::CalibrationParams;
use crate::ml::classifier::LinearClassifier;
use crate::ml::feature_schema::{FeatureSchema, FEATURE_SCHEMA};
use crate::ml::features::FeatureScaler;
use serde::{Deserialize, Serialize};

// Versioned Model

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedModel {
    pub version: u32,
    pub model_id: String,
    pub created_at: String,
    pub feature_schema: FeatureSchema,
    pub classifier: LinearClassifier,
    pub scaler: Option<FeatureScaler>,          // ⭐ NEW
    pub calibration: Option<CalibrationParams>, // ⭐ NEW
    pub threshold: f64,                         // ⭐ NEW
    pub training_metadata: TrainingMetadata,
    pub performance: Option<ModelPerformance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingMetadata {
    pub training_repositories: Vec<String>,
    pub examples_count: usize,
    pub alive_count: usize,
    pub dead_count: usize,
    pub languages: Vec<String>,
    pub training_date: String,
    pub training_duration_secs: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformance {
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
    pub fpr: f64,
    pub fnr: f64,
    pub threshold: f64,
}

impl VersionedModel {
    pub fn new(
        classifier: LinearClassifier,
        metadata: TrainingMetadata,
        performance: Option<ModelPerformance>,
    ) -> Self {
        Self {
            version: 2,
            model_id: format!("model_{}", chrono::Utc::now().timestamp()),
            created_at: chrono::Utc::now().to_rfc3339(),
            feature_schema: FEATURE_SCHEMA.clone(),
            classifier,
            scaler: None,
            calibration: None,
            threshold: 0.92,
            training_metadata: metadata,
            performance,
        }
    }

    /// Create a model with all components
    pub fn new_with_components(
        classifier: LinearClassifier,
        scaler: Option<FeatureScaler>,
        calibration: Option<CalibrationParams>,
        threshold: f64,
        metadata: TrainingMetadata,
        performance: Option<ModelPerformance>,
    ) -> Self {
        Self {
            version: 2,
            model_id: format!("model_{}", chrono::Utc::now().timestamp()),
            created_at: chrono::Utc::now().to_rfc3339(),
            feature_schema: FEATURE_SCHEMA.clone(),
            classifier,
            scaler,
            calibration,
            threshold,
            training_metadata: metadata,
            performance,
        }
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        std::fs::write(path, json).map_err(|e| format!("Failed to write file: {}", e))?;
        Ok(())
    }

    pub fn load(path: &str) -> Result<Self, String> {
        let data =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
        let model: VersionedModel =
            serde_json::from_str(&data).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        // Validate schema compatibility
        model.validate_schema()?;

        Ok(model)
    }

    pub fn validate_schema(&self) -> Result<(), String> {
        let current = FEATURE_SCHEMA.clone();

        if self.feature_schema.version != current.version {
            return Err(format!(
                "Schema version mismatch: model has v{}, current is v{}",
                self.feature_schema.version, current.version
            ));
        }

        if self.feature_schema.feature_count() != current.feature_count() {
            return Err(format!(
                "Feature count mismatch: model has {}, current has {}",
                self.feature_schema.feature_count(),
                current.feature_count()
            ));
        }

        // Validate feature names match
        let model_names: Vec<_> = self.feature_schema.feature_names();
        let current_names: Vec<_> = current.feature_names();

        for (i, (m, c)) in model_names.iter().zip(current_names.iter()).enumerate() {
            if m != c {
                return Err(format!(
                    "Feature name mismatch at index {}: model '{}', current '{}'",
                    i, m, c
                ));
            }
        }

        Ok(())
    }

    pub fn get_classifier(&self) -> &LinearClassifier {
        &self.classifier
    }

    pub fn get_metadata(&self) -> &TrainingMetadata {
        &self.training_metadata
    }

    pub fn get_performance(&self) -> Option<&ModelPerformance> {
        self.performance.as_ref()
    }

    pub fn get_scaler(&self) -> Option<&FeatureScaler> {
        self.scaler.as_ref()
    }

    pub fn get_calibration(&self) -> Option<&CalibrationParams> {
        self.calibration.as_ref()
    }

    pub fn get_threshold(&self) -> f64 {
        self.threshold
    }

    pub fn set_scaler(&mut self, scaler: FeatureScaler) {
        self.scaler = Some(scaler);
    }

    pub fn set_calibration(&mut self, calibration: CalibrationParams) {
        self.calibration = Some(calibration);
    }

    pub fn set_threshold(&mut self, threshold: f64) {
        self.threshold = threshold;
    }
}

// Legacy Model Compatibility

impl VersionedModel {
    /// Migrate from legacy model format (just LinearClassifier)
    pub fn from_legacy(classifier: LinearClassifier, repo_name: Option<&str>) -> Self {
        let metadata = TrainingMetadata {
            training_repositories: repo_name.map(|r| vec![r.to_string()]).unwrap_or_default(),
            examples_count: 0,
            alive_count: 0,
            dead_count: 0,
            languages: Vec::new(),
            training_date: chrono::Utc::now().to_rfc3339(),
            training_duration_secs: 0.0,
        };

        Self::new(classifier, metadata, None)
    }

    /// Load legacy model (just LinearClassifier) and convert
    pub fn load_legacy(path: &str) -> Result<Self, String> {
        let classifier: LinearClassifier = crate::ml::serialization::load_model(path)?;
        Ok(Self::from_legacy(classifier, None))
    }
}
