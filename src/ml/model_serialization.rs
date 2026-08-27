// src/ml/model_serialization.rs

use crate::ml::calibration::CalibrationParams;
use crate::ml::classifier::LinearClassifier;
use crate::ml::feature_schema::{FeatureSchema, FEATURE_SCHEMA};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedModel {
    pub version: u32,
    pub model_id: String,
    pub created_at: String,
    pub feature_schema: FeatureSchema,
    pub classifier: LinearClassifier,
    pub calibration: Option<CalibrationParams>,
    pub threshold: f64,
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
    pub dataset_version: String,
    pub git_commit: Option<String>,
    pub feature_names_hash: String,
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

/// Model validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelValidationResult {
    pub passed: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub feature_count: usize,
    pub model_version: u32,
    pub created_at: String,
}

impl ModelValidationResult {
    pub fn is_valid(&self) -> bool {
        self.passed
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn print(&self) {
        println!("\n📋 Model Validation Report");
        println!("==========================");
        println!("Model Version: v{}", self.model_version);
        println!("Created: {}", self.created_at);
        println!("Features: {}", self.feature_count);
        println!();

        if self.passed {
            println!("✅ Model validation PASSED");
        } else {
            println!("❌ Model validation FAILED");
        }

        if !self.issues.is_empty() {
            println!("\n❌ Issues:");
            for issue in &self.issues {
                println!("  - {}", issue);
            }
        }

        if !self.warnings.is_empty() {
            println!("\n⚠️ Warnings:");
            for warning in &self.warnings {
                println!("  - {}", warning);
            }
        }
    }
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
            calibration: None,
            threshold: 0.92,
            training_metadata: metadata,
            performance,
        }
    }

    pub fn new_with_components(
        classifier: LinearClassifier,
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
            calibration,
            threshold,
            training_metadata: metadata,
            performance,
        }
    }

    pub fn save(&self, path: &str) -> crate::error::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize: {}", e))?;
        std::fs::write(path, json).map_err(|e| anyhow::anyhow!("Failed to write file: {}", e))?;
        Ok(())
    }

    pub fn load(path: &str) -> crate::error::Result<Self> {
        let data = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;
        let model: VersionedModel = serde_json::from_str(&data)
            .map_err(|e| anyhow::anyhow!("Failed to parse JSON: {}", e))?;
        model.validate_schema()?;
        Ok(model)
    }

    /// Comprehensive model validation
    pub fn validate(&self) -> crate::error::Result<ModelValidationResult> {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();
        let mut passed = true;

        // 1. Check schema compatibility
        if let Err(e) = self.validate_schema() {
            issues.push(e.to_string());
            passed = false;
        }

        // 2. Check model ID format
        if self.model_id.is_empty() || self.model_id.len() < 8 {
            warnings.push("Model ID is empty or too short".to_string());
        }

        // 3. Check creation date
        if let Err(e) = chrono::DateTime::parse_from_rfc3339(&self.created_at) {
            warnings.push(format!("Invalid creation date format: {}", e));
        }

        // 4. Check version
        if self.version == 0 {
            warnings.push("Model version is 0 - this may be an unversioned model".to_string());
        }

        // 5. Validate feature schema features
        for feature in &self.feature_schema.features {
            if feature.name.is_empty() {
                issues.push("Feature has empty name".to_string());
                passed = false;
            }
            if feature.index >= self.feature_schema.feature_count() {
                issues.push(format!("Feature index out of bounds: {}", feature.index));
                passed = false;
            }
        }

        // 6. Check classifier weights match feature count
        let weight_count = self.classifier.weights.len();
        let feature_count = self.feature_schema.feature_count();
        if weight_count != feature_count {
            issues.push(format!(
                "Weight count ({}) does not match feature count ({})",
                weight_count, feature_count
            ));
            passed = false;
        }

        // 7. Check for NaN/infinite weights
        for (i, &w) in self.classifier.weights.iter().enumerate() {
            if w.is_nan() || w.is_infinite() {
                issues.push(format!("Weight {} is NaN or infinite", i));
                passed = false;
            }
        }
        if self.classifier.bias.is_nan() || self.classifier.bias.is_infinite() {
            issues.push("Bias is NaN or infinite".to_string());
            passed = false;
        }

        // 8. Check threshold
        if self.threshold <= 0.0 || self.threshold >= 1.0 {
            warnings.push(format!(
                "Threshold {} is outside valid range (0.0-1.0)",
                self.threshold
            ));
        }

        // 9. Check calibration if present
        if let Some(cal) = &self.calibration {
            if cal.temperature <= 0.0 {
                issues.push(format!(
                    "Calibration temperature {} is not positive",
                    cal.temperature
                ));
                passed = false;
            }
            if cal.num_samples == 0 && cal.bins.is_empty() {
                warnings.push("Calibration has no samples and no bins".to_string());
            }
        }

        // 10. Check performance metrics if present
        if let Some(perf) = &self.performance {
            if perf.accuracy < 0.0 || perf.accuracy > 1.0 {
                warnings.push(format!("Accuracy {} is outside valid range", perf.accuracy));
            }
            if perf.precision < 0.0 || perf.precision > 1.0 {
                warnings.push(format!(
                    "Precision {} is outside valid range",
                    perf.precision
                ));
            }
            if perf.recall < 0.0 || perf.recall > 1.0 {
                warnings.push(format!("Recall {} is outside valid range", perf.recall));
            }
            if perf.f1 < 0.0 || perf.f1 > 1.0 {
                warnings.push(format!("F1 {} is outside valid range", perf.f1));
            }
        }

        // 11. Check training metadata
        if self.training_metadata.examples_count == 0 {
            warnings.push("Training metadata shows 0 examples".to_string());
        }
        if self.training_metadata.dataset_version == "unknown" {
            warnings.push("Unknown training dataset version".to_string());
        }

        Ok(ModelValidationResult {
            passed,
            issues,
            warnings,
            feature_count: self.feature_schema.feature_count(),
            model_version: self.version,
            created_at: self.created_at.clone(),
        })
    }

    /// Validate schema compatibility
    pub fn validate_schema(&self) -> crate::error::Result<()> {
        let current = FEATURE_SCHEMA.clone();

        if self.feature_schema.version != current.version {
            return Err(anyhow::anyhow!(
                "Schema version mismatch: model has v{}, current is v{}",
                self.feature_schema.version,
                current.version
            ));
        }

        if self.feature_schema.feature_count() != current.feature_count() {
            return Err(anyhow::anyhow!(
                "Feature count mismatch: model has {}, current has {}",
                self.feature_schema.feature_count(),
                current.feature_count()
            ));
        }

        let model_names: Vec<_> = self.feature_schema.feature_names();
        let current_names: Vec<_> = current.feature_names();

        for (i, (m, c)) in model_names.iter().zip(current_names.iter()).enumerate() {
            if m != c {
                return Err(anyhow::anyhow!(
                    "Feature name mismatch at index {}: model '{}', current '{}'",
                    i,
                    m,
                    c
                ));
            }
        }

        Ok(())
    }

    /// Check if model is compatible with current runtime
    pub fn is_compatible(&self) -> bool {
        self.validate_schema().is_ok()
    }

    /// Get model summary
    pub fn summary(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("Model: {}\n", self.model_id));
        s.push_str(&format!("Version: v{}\n", self.version));
        s.push_str(&format!("Created: {}\n", self.created_at));
        s.push_str(&format!(
            "Features: {}\n",
            self.feature_schema.feature_count()
        ));
        s.push_str(&format!("Threshold: {:.2}\n", self.threshold));

        if let Some(perf) = &self.performance {
            s.push_str(&format!("Accuracy: {:.1}%\n", perf.accuracy * 100.0));
            s.push_str(&format!("F1: {:.1}%\n", perf.f1 * 100.0));
        }

        if let Some(cal) = &self.calibration {
            s.push_str(&format!("Calibration: {:?}\n", cal.method));
            s.push_str(&format!("Calibration samples: {}\n", cal.num_samples));
        }

        s
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

    pub fn get_calibration(&self) -> Option<&CalibrationParams> {
        self.calibration.as_ref()
    }

    pub fn get_threshold(&self) -> f64 {
        self.threshold
    }

    pub fn set_calibration(&mut self, calibration: CalibrationParams) {
        self.calibration = Some(calibration);
    }

    pub fn set_threshold(&mut self, threshold: f64) {
        self.threshold = threshold;
    }
}

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
            dataset_version: "unknown".to_string(),
            git_commit: None,
            feature_names_hash: "unknown".to_string(),
        };

        Self::new(classifier, metadata, None)
    }

    pub fn compute_feature_names_hash() -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        for name in FEATURE_SCHEMA.feature_names() {
            hasher.update(name.as_bytes());
            hasher.update(b"\n");
        }
        hex::encode(hasher.finalize())[..16].to_string()
    }

    /// Load legacy model (just LinearClassifier) and convert
    pub fn load_legacy(path: &str) -> crate::error::Result<Self> {
        let classifier: LinearClassifier =
            crate::ml::serialization::ModelSerializer::load_auto(path)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(Self::from_legacy(classifier, None))
    }
}
