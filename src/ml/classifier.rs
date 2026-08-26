use crate::analysis::training_data::{TrainingExample, TrainingLabel};
use crate::ml::feature_schema::{feature_count, feature_names, FeatureCategory, FEATURE_SCHEMA};
use crate::ml::features::FeatureScaler;
use crate::ml::CalibrationParams;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct TrainingResult {
    pub trainable_accuracy: f64,
    pub verified_accuracy: f64,
    pub weighted_accuracy: f64,
    pub heuristic_accuracy: f64,
}

impl TrainingResult {
    pub fn empty() -> Self {
        Self {
            trainable_accuracy: 0.0,
            verified_accuracy: 0.0,
            weighted_accuracy: 0.0,
            heuristic_accuracy: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearClassifier {
    pub weights: Vec<f64>,
    pub bias: f64,
    pub learning_rate: f64,
    pub epochs: usize,
    pub feature_count: usize,
    #[serde(default)]
    pub scaler: Option<FeatureScaler>,
    #[serde(default = "default_threshold")]
    pub threshold: f64,
}

fn default_threshold() -> f64 {
    0.92
}

impl LinearClassifier {
    pub fn new(feature_count: usize) -> Self {
        Self {
            weights: vec![0.0; feature_count],
            bias: 0.0,
            learning_rate: 0.005,
            epochs: 50,
            feature_count,
            scaler: None,
            threshold: default_threshold(),
        }
    }

    pub fn new_with_schema() -> Self {
        Self::new(feature_count())
    }

    pub fn with_learning_rate(mut self, lr: f64) -> Self {
        self.learning_rate = lr;
        self
    }

    pub fn with_epochs(mut self, epochs: usize) -> Self {
        self.epochs = epochs;
        self
    }

    pub fn validate_features(&self, features: &[f64]) -> Result<(), String> {
        if features.len() != self.feature_count {
            return Err(format!(
                "Feature count mismatch: expected {}, got {}",
                self.feature_count,
                features.len()
            ));
        }

        FEATURE_SCHEMA.validate_vector(features)?;
        Ok(())
    }

    pub fn predict_validated(&self, features: &[f64]) -> Result<f64, String> {
        self.validate_features(features)?;
        Ok(self.predict(features))
    }

    pub fn predict_label_validated(&self, features: &[f64]) -> Result<TrainingLabel, String> {
        self.validate_features(features)?;
        Ok(self.predict_label(features))
    }

    pub fn train(&mut self, examples: &[TrainingExample]) -> TrainingResult {
        use crate::analysis::training_data_filter::TrainingDataFilter;

        let trainable = TrainingDataFilter::filter_trainable(examples);

        if trainable.is_empty() {
            println!("❌ No trainable examples found (all excluded by TrainingDataFilter)");
            return TrainingResult::empty();
        }

        let stats = TrainingDataFilter::separate_by_source(examples);
        println!("\n📊 Training Data Report:");
        println!("{}", stats.format_report());
        println!("   Actually training on: {} examples\n", trainable.len());

        if self.weights.len() != self.feature_count {
            self.weights = vec![0.0; self.feature_count];
        }

        let raw_vectors: Vec<Vec<f64>> = trainable
            .iter()
            .map(|e| e.features.to_feature_vector())
            .collect();
        let mut scaler = FeatureScaler::new();
        scaler.fit_from_vectors(&raw_vectors);
        self.scaler = Some(scaler);

        for epoch in 0..self.epochs {
            let mut total_loss = 0.0;

            let mut shuffled = trainable.clone();
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            shuffled.shuffle(&mut rng);

            for example in &shuffled {
                let raw = example.features.to_feature_vector();
                let features = self.scaler.as_ref().unwrap().transform(&raw);

                let target = match example.label {
                    TrainingLabel::Alive => 1.0,
                    TrainingLabel::Dead => 0.0,
                    TrainingLabel::Unknown => 0.5,
                };

                let weight = example.label_source.training_weight();

                let dot: f64 = features.iter().zip(&self.weights).map(|(f, w)| f * w).sum();
                let z = (dot + self.bias).clamp(-20.0, 20.0);

                let prediction = 1.0 / (1.0 + (-z).exp());

                let p_safe = prediction.clamp(1e-7, 1.0 - 1e-7);
                let loss = (-target * p_safe.ln() - (1.0 - target) * (1.0 - p_safe).ln()) * weight;
                total_loss += loss;

                let error = (prediction - target) * weight;
                for (i, &feature) in features.iter().enumerate() {
                    if i < self.weights.len() {
                        let grad = (error * feature).clamp(-5.0, 5.0);
                        self.weights[i] -= self.learning_rate * grad;
                    }
                }
                self.bias -= self.learning_rate * error.clamp(-5.0, 5.0);
            }

            let avg_loss = total_loss / shuffled.len() as f64;
            if epoch % 10 == 0 && epoch > 0 {
                println!("    Epoch {}: loss = {:.4}", epoch, avg_loss);
            }
        }

        self.calculate_accuracy(examples)
    }

    pub fn calculate_accuracy(&self, examples: &[TrainingExample]) -> TrainingResult {
        use crate::analysis::training_data_filter::TrainingDataFilter;

        let trainable = TrainingDataFilter::filter_trainable(examples);
        let verified: Vec<_> = examples.iter().filter(|e| e.is_verified()).collect();
        let heuristic: Vec<_> = examples.iter().filter(|e| e.is_heuristic()).collect();

        let mut result = TrainingResult::empty();

        if trainable.is_empty() {
            return result;
        }

        let mut correct_trainable = 0;
        let mut weighted_correct = 0.0;
        let mut weighted_total = 0.0;

        for example in &trainable {
            let features = example.features.to_feature_vector();
            let pred = self.predict_label(&features);
            if pred == example.label {
                correct_trainable += 1;
            }

            let weight = example.label_source.training_weight();
            if pred == example.label {
                weighted_correct += weight;
            }
            weighted_total += weight;
        }

        result.trainable_accuracy = correct_trainable as f64 / trainable.len() as f64;
        result.weighted_accuracy = weighted_correct / weighted_total;

        if !verified.is_empty() {
            let mut correct_verified = 0;
            for example in &verified {
                let features = example.features.to_feature_vector();
                let pred = self.predict_label(&features);
                if pred == example.label {
                    correct_verified += 1;
                }
            }
            result.verified_accuracy = correct_verified as f64 / verified.len() as f64;
        }

        if !heuristic.is_empty() {
            let mut correct_heuristic = 0;
            for example in &heuristic {
                let features = example.features.to_feature_vector();
                let pred = self.predict_label(&features);
                if pred == example.label {
                    correct_heuristic += 1;
                }
            }
            result.heuristic_accuracy = correct_heuristic as f64 / heuristic.len() as f64;
        }

        println!(
            "\n    📊 Training Accuracy (trainable only): {:.1}%",
            result.trainable_accuracy * 100.0
        );
        if result.verified_accuracy > 0.0 {
            println!(
                "    ✅ Verified Accuracy: {:.1}%",
                result.verified_accuracy * 100.0
            );
        }
        if result.weighted_accuracy > 0.0 {
            println!(
                "    ⚖️  Weighted Accuracy: {:.1}%",
                result.weighted_accuracy * 100.0
            );
        }

        result
    }

    pub fn predict(&self, features: &[f64]) -> f64 {
        let scaled;
        let features = if let Some(scaler) = &self.scaler {
            scaled = scaler.transform(features);
            &scaled
        } else {
            features
        };
        let dot: f64 = features.iter().zip(&self.weights).map(|(f, w)| f * w).sum();
        let z = (dot + self.bias).clamp(-20.0, 20.0);
        1.0 / (1.0 + (-z).exp())
    }

    pub fn predict_label(&self, features: &[f64]) -> TrainingLabel {
        if self.predict(features) >= 0.5 {
            TrainingLabel::Alive
        } else {
            TrainingLabel::Dead
        }
    }

    pub fn feature_importance(&self) -> Vec<(String, f64)> {
        let names = feature_names();
        let mut importance: Vec<(String, f64)> = names
            .iter()
            .zip(self.weights.iter())
            .map(|(name, &weight)| (name.clone(), weight))
            .collect();

        importance.sort_by(|a, b| b.1.abs().total_cmp(&a.1.abs()));
        importance
    }

    pub fn print_feature_importance(&self) {
        println!("\n📊 Feature Importance (top 15):");
        println!(
            "    {:>30} | {:>10} | {:>15}",
            "Feature", "Weight", "Direction"
        );
        println!("    {:-<30}-+-{:-<10}-+-{:-<15}", "", "", "");

        for (name, weight) in self.feature_importance().iter().take(15) {
            let direction = if *weight > 0.05 {
                "→ ALIVE"
            } else if *weight < -0.05 {
                "→ DEAD"
            } else {
                "→ UNCERTAIN"
            };
            println!("    {:>30} | {:>10.3} | {:>15}", name, weight, direction);
        }

        println!("\n    By Category (average absolute weight):");
        for category in [
            FeatureCategory::Graph,
            FeatureCategory::Signature,
            FeatureCategory::Complexity,
            FeatureCategory::Name,
            FeatureCategory::File,
            FeatureCategory::Type,
        ] {
            let features = FEATURE_SCHEMA.get_by_category(&category);
            if !features.is_empty() {
                let avg_weight: f64 = features
                    .iter()
                    .filter_map(|f| self.weights.get(f.index).map(|w| w.abs()))
                    .sum::<f64>()
                    / features.len() as f64;
                println!("       {:?}: {:.3}", category, avg_weight);
            }
        }
    }

    pub fn get_weights(&self) -> &[f64] {
        &self.weights
    }

    pub fn get_bias(&self) -> f64 {
        self.bias
    }

    pub fn feature_count(&self) -> usize {
        self.feature_count
    }
}

impl Default for LinearClassifier {
    fn default() -> Self {
        Self::new_with_schema()
    }
}

// Dead Code Classifier (Wrapper)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeClassifier {
    pub model: Option<LinearClassifier>,
    pub accuracy: f64,
    pub feature_count: usize,
    pub calibration: Option<CalibrationParams>,
}

pub const EMBEDDED_MODEL_BYTES: &[u8] = include_bytes!("../../models/model.bin");

impl DeadCodeClassifier {
    pub fn new() -> Self {
        Self {
            model: None,
            accuracy: 0.0,
            feature_count: feature_count(),
            calibration: None,
        }
    }

    pub fn load_embedded() -> Result<Self, String> {
        bincode::deserialize(EMBEDDED_MODEL_BYTES)
            .map_err(|e| format!("Failed to deserialize embedded model.bin: {}", e))
    }

    pub fn train(&mut self, examples: &[TrainingExample]) -> Result<(), String> {
        let feature_count = if let Some(first) = examples.first() {
            first.features.to_feature_vector().len()
        } else {
            return Err("No training examples found".to_string());
        };

        println!(
            "📊 Training Linear Classifier on {} examples",
            examples.len()
        );
        println!("    Features: {}", feature_count);

        let mut classifier = LinearClassifier::new(feature_count)
            .with_learning_rate(0.005)
            .with_epochs(50);

        let training_result = classifier.train(examples);
        self.accuracy = training_result.trainable_accuracy;
        self.model = Some(classifier);

        Ok(())
    }

    pub fn set_threshold(&mut self, threshold: f64) {
        if let Some(model) = &mut self.model {
            model.threshold = threshold;
        }
    }

    pub fn get_threshold(&self) -> f64 {
        if let Some(model) = &self.model {
            model.threshold
        } else {
            0.92
        }
    }

    pub fn validate_features(&self, features: &[f64]) -> Result<(), String> {
        if let Some(model) = &self.model {
            model.validate_features(features)
        } else {
            Err("No model loaded".to_string())
        }
    }

    pub fn predict_validated(&self, example: &TrainingExample) -> Result<TrainingLabel, String> {
        let features = example.features.to_feature_vector();
        self.validate_features(&features)?;
        Ok(self.predict(example))
    }

    pub fn is_schema_compatible(&self) -> bool {
        if let Some(model) = &self.model {
            model.feature_count == FEATURE_SCHEMA.feature_count()
        } else {
            false
        }
    }

    pub fn schema_info(&self) -> String {
        format!(
            "Schema v{} ({} features) - Model: {} features",
            FEATURE_SCHEMA.version,
            FEATURE_SCHEMA.feature_count(),
            self.feature_count
        )
    }

    pub fn predict(&self, example: &TrainingExample) -> TrainingLabel {
        if let Some(model) = &self.model {
            let features = example.features.to_feature_vector();
            model.predict_label(&features)
        } else {
            TrainingLabel::Unknown
        }
    }

    pub fn predict_probability(&self, example: &TrainingExample) -> f64 {
        if let Some(model) = &self.model {
            let features = example.features.to_feature_vector();
            model.predict(&features)
        } else {
            0.5
        }
    }

    pub fn predict_features(&self, features: &[f64]) -> f64 {
        if let Some(model) = &self.model {
            model.predict(features)
        } else {
            0.5
        }
    }

    pub fn get_accuracy(&self) -> f64 {
        self.accuracy
    }

    pub fn is_trained(&self) -> bool {
        self.model.is_some()
    }

    pub fn get_model(&self) -> Option<&LinearClassifier> {
        self.model.as_ref()
    }

    pub fn get_model_mut(&mut self) -> Option<&mut LinearClassifier> {
        self.model.as_mut()
    }

    pub fn predict_alive_probability(&self, example: &TrainingExample) -> f64 {
        self.predict_probability(example)
    }

    pub fn predict_dead_probability(&self, example: &TrainingExample) -> f64 {
        1.0 - self.predict_probability(example)
    }

    pub fn print_feature_importance(&self) {
        if let Some(model) = &self.model {
            model.print_feature_importance();
        } else {
            println!("No model trained yet.");
        }
    }

    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> crate::error::Result<()> {
        crate::ml::serialization::ModelSerializer::save_binary(self, path)
    }

    pub fn load<P: AsRef<std::path::Path>>(path: P) -> crate::error::Result<Self> {
        crate::ml::serialization::ModelSerializer::load_auto(path)
    }
}

impl Default for DeadCodeClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::training_data::{FunctionFeatures, TrainingExample, TrainingLabel};
    use crate::analysis::verdict_source::label_source::LabelSource;
    use crate::ml::feature_schema::feature_count;

    fn create_test_example(
        name: &str,
        label: TrainingLabel,
        source: LabelSource,
        fan_in: usize,
        fan_out: usize,
    ) -> TrainingExample {
        let mut features = FunctionFeatures::default();
        features.fan_in = fan_in;
        features.fan_out = fan_out;
        features.name_length = name.len();
        features.name_contains_get = name.contains("get");
        features.name_contains_set = name.contains("set");
        features.is_public = fan_in > 0;

        TrainingExample {
            function_name: name.to_string(),
            full_path: format!("/test/{}", name),
            file: format!("{}.rs", name),
            language: "rust".to_string(),
            features,
            label,
            confidence: 0.9,
            source: "test".to_string(),
            repository_id: None,
            commit_hash: None,
            dataset_split: None,
            label_reason: Some("test".to_string()),
            label_version: Some(1),
            label_source: source,
            generated_by_model: None,
            verified_by: None,
            created_at: None,
        }
    }

    #[test]
    fn test_static_heuristic_only_training_refuses() {
        let examples: Vec<TrainingExample> = (0..10)
            .map(|i| {
                create_test_example(
                    &format!("dead_fn_{}", i),
                    TrainingLabel::Dead,
                    LabelSource::StaticHeuristic,
                    0,
                    0,
                )
            })
            .collect();

        let mut classifier = LinearClassifier::new(feature_count());
        let result = classifier.train(&examples);

        assert_eq!(result.trainable_accuracy, 0.0);
        assert_eq!(result.verified_accuracy, 0.0);
        assert_eq!(result.weighted_accuracy, 0.0);
    }

    #[test]
    fn test_only_human_verified_influences_model() {
        let mut examples: Vec<TrainingExample> = Vec::new();

        for i in 0..1000 {
            examples.push(create_test_example(
                &format!("heuristic_dead_{}", i),
                TrainingLabel::Dead,
                LabelSource::StaticHeuristic,
                0,
                0,
            ));
        }

        for i in 0..100 {
            examples.push(create_test_example(
                &format!("verified_alive_{}", i),
                TrainingLabel::Alive,
                LabelSource::HumanVerified,
                10,
                5,
            ));
        }

        let mut classifier = LinearClassifier::new(feature_count());
        let result = classifier.train(&examples);

        assert!(
            result.trainable_accuracy > 0.5,
            "Should learn from verified examples"
        );

        let alive_features = create_test_example(
            "alive_test",
            TrainingLabel::Alive,
            LabelSource::HumanVerified,
            10,
            5,
        );
        let alive_pred = classifier.predict(&alive_features.features.to_feature_vector());
        assert!(alive_pred > 0.5, "Should predict alive for high fan-in/out");
    }

    #[test]
    fn test_silver_excluded_by_default() {
        let examples: Vec<TrainingExample> = (0..50)
            .map(|i| {
                create_test_example(
                    &format!("silver_dead_{}", i),
                    TrainingLabel::Dead,
                    LabelSource::Silver,
                    0,
                    0,
                )
            })
            .collect();

        let mut classifier = LinearClassifier::new(feature_count());
        let result = classifier.train(&examples);

        assert_eq!(
            result.trainable_accuracy, 0.0,
            "Silver should not be trainable by default"
        );
    }

    #[test]
    fn test_training_weights_applied() {
        let mut examples: Vec<TrainingExample> = Vec::new();

        for i in 0..50 {
            examples.push(create_test_example(
                &format!("production_alive_{}", i),
                TrainingLabel::Alive,
                LabelSource::ProductionVerified,
                20,
                15,
            ));
        }

        for i in 0..50 {
            examples.push(create_test_example(
                &format!("dataset_dead_{}", i),
                TrainingLabel::Dead,
                LabelSource::DatasetVerified,
                0,
                0,
            ));
        }

        let mut classifier = LinearClassifier::new(feature_count());
        let result = classifier.train(&examples);

        assert!(
            result.trainable_accuracy > 0.7,
            "Should learn from weighted examples"
        );
        assert!(
            result.verified_accuracy > 0.7,
            "Verified accuracy should be high"
        );
        assert!(
            result.weighted_accuracy > 0.0,
            "Weighted accuracy should be calculated"
        );
    }
}
