// src/ml/classifier.rs
use crate::analysis::training_data::{TrainingExample, TrainingLabel};
use crate::ml::feature_schema::{feature_count, feature_names, FeatureCategory, FEATURE_SCHEMA};
use crate::ml::features::FeatureScaler;
use crate::ml::CalibrationParams;
use serde::{Deserialize, Serialize};

// ============================================================================
// Linear Classifier
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearClassifier {
    weights: Vec<f64>,
    bias: f64,
    learning_rate: f64,
    epochs: usize,
    feature_count: usize,
    // `#[serde(default)]` so model files saved before this field existed
    // still deserialize (they'll just load with scaler: None).
    #[serde(default)]
    scaler: Option<FeatureScaler>,
}

impl LinearClassifier {
    pub fn new(feature_count: usize) -> Self {
        Self {
            weights: vec![0.0; feature_count],
            bias: 0.0,
            learning_rate: 0.01,
            epochs: 100,
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

    pub fn train(&mut self, examples: &[TrainingExample]) -> f64 {
        let labeled: Vec<_> = examples
            .iter()
            .filter(|e| e.label != TrainingLabel::Unknown)
            .collect();

        if labeled.is_empty() {
            return 0.0;
        }

        // Ensure we have the right number of features
        if self.weights.len() != self.feature_count {
            self.weights = vec![0.0; self.feature_count];
        }

        // Fit the scaler on the raw training features before touching any
        // weights, then use it for every example below (and keep it, so
        // predict() applies the same transform later).
        let raw_vectors: Vec<Vec<f64>> = labeled
            .iter()
            .map(|e| e.features.to_feature_vector())
            .collect();
        let mut scaler = FeatureScaler::new();
        scaler.fit_from_vectors(&raw_vectors);
        self.scaler = Some(scaler);

        // Train using gradient descent
        for epoch in 0..self.epochs {
            let mut total_loss = 0.0;

            // Shuffle examples
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

                // Forward pass (sigmoid)
                let dot: f64 = features.iter().zip(&self.weights).map(|(f, w)| f * w).sum();
                let prediction = 1.0 / (1.0 + (-(dot + self.bias)).exp());

                // Loss (binary cross-entropy)
                let loss = -target * prediction.ln() - (1.0 - target) * (1.0 - prediction).ln();
                total_loss += loss;

                // Backward pass (gradient descent)
                let error = prediction - target;

                for (i, &feature) in features.iter().enumerate() {
                    if i < self.weights.len() {
                        self.weights[i] -= self.learning_rate * error * feature;
                    }
                }
                self.bias -= self.learning_rate * error;
            }

            let avg_loss = total_loss / shuffled.len() as f64;

            if epoch % 20 == 0 && epoch > 0 {
                println!("   Epoch {}: loss = {:.4}", epoch, avg_loss);
            }
        }

        self.calculate_accuracy(examples)
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
        1.0 / (1.0 + (-(dot + self.bias)).exp())
    }

    pub fn predict_label(&self, features: &[f64]) -> TrainingLabel {
        if self.predict(features) > 0.5 {
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
            "   {:>30} | {:>10} | {:>15}",
            "Feature", "Weight", "Direction"
        );
        println!("   {:-<30}-+-{:-<10}-+-{:-<15}", "", "", "");

        for (name, weight) in self.feature_importance().iter().take(15) {
            let direction = if *weight > 0.05 {
                "→ ALIVE"
            } else if *weight < -0.05 {
                "→ DEAD"
            } else {
                "→ UNCERTAIN"
            };
            println!("   {:>30} | {:>10.3} | {:>15}", name, weight, direction);
        }

        // Also show feature category breakdown
        println!("\n   By Category (average absolute weight):");
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
                println!("      {:?}: {:.3}", category, avg_weight);
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

// ============================================================================
// Dead Code Classifier (Wrapper)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeClassifier {
    pub model: Option<LinearClassifier>,
    pub accuracy: f64,
    pub feature_count: usize,
    pub calibration: Option<CalibrationParams>, // ⭐ NEW
}

impl DeadCodeClassifier {
    pub fn new() -> Self {
        Self {
            model: None,
            accuracy: 0.0,
            feature_count: feature_count(),
            calibration: None, // ⭐ NEW
        }
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
        println!("   Features: {}", feature_count);

        let mut classifier = LinearClassifier::new(feature_count)
            .with_learning_rate(0.01)
            .with_epochs(50);

        self.accuracy = classifier.train(examples);
        self.model = Some(classifier);

        println!("   Accuracy: {:.1}%", self.accuracy * 100.0);

        Ok(())
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

    pub fn save(&self, path: &str) -> Result<(), String> {
        if let Some(model) = &self.model {
            crate::ml::serialization::save_model(model, path)
        } else {
            Err("No model trained".to_string())
        }
    }

    pub fn load(path: &str) -> Result<Self, String> {
        let model: LinearClassifier = crate::ml::serialization::load_model(path)?;
        let feature_count = model.feature_count();
        Ok(Self {
            model: Some(model),
            accuracy: 0.0,
            feature_count,
            calibration: None,
        })
    }
}

impl Default for DeadCodeClassifier {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::call_graph::FunctionNode;

    fn create_test_function(name: &str, fan_in: usize, is_public: bool) -> FunctionNode {
        FunctionNode {
            name: name.to_string(),
            full_path: format!("test::{}", name),
            file: "test.rs".to_string(),
            line: 1,
            body_start_line: 1,
            body_end_line: 1,
            is_public,
            is_async: false,
            params: vec![],
            returns: vec![],
            complexity: 1.0,
            importance_score: 0.0,
            doc_comment: None,
            writes_to: vec![],
            reads_from: vec![],
            errors: vec![],
            fan_in,
            fan_out: 0,
            is_cycle: false,
            depth: 0,
            layer: "core".to_string(),
            trait_impl: None,
            is_test: true,
            is_trait_method: false,
            is_trait_default: false,
        }
    }

    #[test]
    fn test_linear_classifier_new() {
        let classifier = LinearClassifier::new_with_schema();
        assert_eq!(classifier.feature_count(), feature_count());
        assert_eq!(classifier.weights.len(), feature_count());
    }

    #[test]
    fn test_linear_classifier_train_and_predict() {
        let mut classifier = LinearClassifier::new(10)
            .with_learning_rate(0.1)
            .with_epochs(10);

        // Create synthetic training data
        let mut examples = Vec::new();

        // Alive examples (high fan_in)
        for i in 0..10 {
            let func = create_test_function(&format!("alive_{}", i), 5 + i, true);
            let features = crate::analysis::training_data::FunctionFeatures::from_function(
                &func,
                &crate::graph::call_graph::CallGraph::new(),
            );
            examples.push(TrainingExample {
                function_name: func.name.clone(),
                full_path: func.full_path.clone(),
                file: func.file.clone(),
                language: "rust".to_string(),
                features,
                label: TrainingLabel::Alive,
                confidence: 0.9,
                source: "test".to_string(),
                repository_id: None,
                commit_hash: None,
                dataset_split: None,
                label_reason: None,
                label_version: Some(1),
            });
        }

        // Dead examples (low fan_in)
        for i in 0..10 {
            let func = create_test_function(&format!("dead_{}", i), 0, false);
            let features = crate::analysis::training_data::FunctionFeatures::from_function(
                &func,
                &crate::graph::call_graph::CallGraph::new(),
            );
            examples.push(TrainingExample {
                function_name: func.name.clone(),
                full_path: func.full_path.clone(),
                file: func.file.clone(),
                language: "rust".to_string(),
                features,
                label: TrainingLabel::Dead,
                confidence: 0.9,
                source: "test".to_string(),
                repository_id: None,
                commit_hash: None,
                dataset_split: None,
                label_reason: None,
                label_version: Some(1),
            });
        }

        let accuracy = classifier.train(&examples);
        assert!(accuracy > 0.5);
    }
}
