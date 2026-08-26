use crate::analysis::training_data::{TrainingExample, TrainingLabel};
use crate::ml::feature_schema::{feature_count, feature_names, FeatureCategory, FEATURE_SCHEMA};
use crate::ml::features::FeatureScaler;
use crate::ml::CalibrationParams;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearClassifier {
    pub weights: Vec<f64>,
    pub bias: f64,
    pub learning_rate: f64,
    pub epochs: usize,
    pub feature_count: usize,
    #[serde(default)]
    pub scaler: Option<FeatureScaler>,
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

    pub fn train(&mut self, examples: &[TrainingExample]) -> f64 {
        let labeled: Vec<_> = examples
            .iter()
            .filter(|e| e.label != TrainingLabel::Unknown)
            .collect();

        if labeled.is_empty() {
            return 0.0;
        }

        if self.weights.len() != self.feature_count {
            self.weights = vec![0.0; self.feature_count];
        }

        let raw_vectors: Vec<Vec<f64>> = labeled
            .iter()
            .map(|e| e.features.to_feature_vector())
            .collect();
        let mut scaler = FeatureScaler::new();
        scaler.fit_from_vectors(&raw_vectors);
        self.scaler = Some(scaler);

        for epoch in 0..self.epochs {
            let mut total_loss = 0.0;

            let mut shuffled = labeled.clone();
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

                let dot: f64 = features
                    .iter() // Look at each feature one by one
                    .zip(&self.weights) // Pair each feature with its weight
                    .map(|(f, w)| f * w) // Multiply: feature × weight
                    .sum(); // Add all products together
                let z = (dot + self.bias).clamp(-20.0, 20.0);

                let prediction = 1.0 / (1.0 + (-z).exp());

                // EXAMPLE WITH A "DEAD" FUNCTION
                // Input: features = [0.0, 0.0, 0.0]  (no callers, no callees, private)
                // Weights: same as before: [0.425, -0.310, 0.280]
                // Bias: 0.5
                //
                // dot = (0.0 × 0.425) + (0.0 × -0.310) + (0.0 × 0.280) = 0.0
                // z = 0.0 + 0.5 = 0.5
                //
                // prediction = 1.0 / (1.0 + e^(-0.5))
                //            = 1.0 / (1.0 + 0.606)
                //            = 1.0 / 1.606
                //            = 0.622
                //
                // 0.622 >= 0.5 → ALIVE! (62.2% chance alive)
                //
                // COMPLETE EXAMPLE WITH A "DEAD" FUNCTION
                //
                // Input: features = [0.0, 0.0, 0.0]  (no callers, no callees, private)
                // Weights: [0.425, -0.310, 0.280]
                // Bias: -0.5 (different bias!)
                //
                // dot = (0.0 × 0.425) + (0.0 × -0.310) + (0.0 × 0.280) = 0.0
                // z = 0.0 + (-0.5) = -0.5
                //
                // prediction = 1.0 / (1.0 + e^(-(-0.5)))
                //            = 1.0 / (1.0 + e^(0.5))
                //            = 1.0 / (1.0 + 1.648)
                //            = 1.0 / 2.648
                //            = 0.377
                //
                // 0.377 < 0.5 → DEAD! (37.7% chance alive, 62.3% chance dead)
                // Numerically bounded BCE loss
                let p_safe = prediction.clamp(1e-7, 1.0 - 1e-7);
                let loss = -target * p_safe.ln() - (1.0 - target) * (1.0 - p_safe).ln();
                total_loss += loss;

                // Gradient descent with gradient clipping
                let error = prediction - target;
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

        let training_accuracy = self.calculate_accuracy(examples);
        println!(
            "\n    📊 Training Accuracy: {:.1}%",
            training_accuracy * 100.0
        );
        training_accuracy
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

    pub fn calculate_accuracy(&self, examples: &[TrainingExample]) -> f64 {
        let labeled: Vec<_> = examples
            .iter()
            .filter(|e| e.label != TrainingLabel::Unknown)
            .collect();

        if labeled.is_empty() {
            return 0.0;
        }

        let mut correct = 0;
        for example in &labeled {
            let features = example.features.to_feature_vector();
            let pred = self.predict_label(&features);
            if pred == example.label {
                correct += 1;
            }
        }

        correct as f64 / labeled.len() as f64
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

        let training_accuracy = classifier.train(examples);
        self.accuracy = training_accuracy;
        self.model = Some(classifier);

        Ok(())
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
