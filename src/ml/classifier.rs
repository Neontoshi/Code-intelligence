// src/ml/classifier.rs

use crate::analysis::training_data::{TrainingExample, TrainingLabel};
use serde::{Deserialize, Serialize};

// Simple linear classifier with gradient descent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearClassifier {
    weights: Vec<f64>,
    bias: f64,
    learning_rate: f64,
    epochs: usize,
}

impl LinearClassifier {
    pub fn new(feature_count: usize) -> Self {
        Self {
            weights: vec![0.0; feature_count],
            bias: 0.0,
            learning_rate: 0.01,
            epochs: 100,
        }
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

        let feature_count = if let Some(first) = labeled.first() {
            first.features.to_feature_vector().len()
        } else {
            33
        };

        // Initialize weights if not set
        if self.weights.len() != feature_count {
            self.weights = vec![0.0; feature_count];
        }

        // Train using gradient descent
        for epoch in 0..self.epochs {
            let mut total_loss = 0.0;

            // Shuffle examples
            let mut shuffled = labeled.clone();
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            shuffled.shuffle(&mut rng);

            for example in &shuffled {
                let features = example.features.to_feature_vector();
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

            if epoch % 20 == 0 {
                println!("   Epoch {}: loss = {:.4}", epoch, avg_loss);
            }
        }

        self.calculate_accuracy(examples)
    }

    pub fn predict(&self, features: &[f64]) -> f64 {
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

    pub fn feature_importance(&self, feature_names: &[String]) -> Vec<(String, f64)> {
        let mut importance: Vec<(String, f64)> = feature_names
            .iter()
            .zip(self.weights.iter())
            .map(|(name, &weight)| (name.clone(), weight))
            .collect();

        importance.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap());
        importance
    }
}

pub struct DeadCodeClassifier {
    model: Option<LinearClassifier>,
    accuracy: f64,
}

impl DeadCodeClassifier {
    pub fn new() -> Self {
        Self {
            model: None,
            accuracy: 0.0,
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

    pub fn get_accuracy(&self) -> f64 {
        self.accuracy
    }

    pub fn is_trained(&self) -> bool {
        self.model.is_some()
    }

    pub fn print_feature_importance(&self) {
        if let Some(model) = &self.model {
            let feature_names = vec![
                "param_count".to_string(),
                "return_count".to_string(),
                "is_public".to_string(),
                "is_async".to_string(),
                "name_length".to_string(),
                "starts_with_use".to_string(),
                "starts_with_test".to_string(),
                "starts_with_bench".to_string(),
                "ends_with_test".to_string(),
                "contains_trait_impl".to_string(),
                "fan_in".to_string(),
                "fan_out".to_string(),
                "complexity".to_string(),
                "call_depth".to_string(),
                "is_cycle".to_string(),
                "is_in_test_file".to_string(),
                "is_in_benches".to_string(),
                "is_in_meta".to_string(),
                "is_in_examples".to_string(),
                "is_generated".to_string(),
                "name_contains_use".to_string(),
                "name_contains_test".to_string(),
                "name_contains_init".to_string(),
                "name_contains_get".to_string(),
                "name_contains_set".to_string(),
                "name_contains_new".to_string(),
                "name_contains_create".to_string(),
                "name_contains_build".to_string(),
                "name_contains_parse".to_string(),
                "name_contains_validate".to_string(),
                "name_contains_handle".to_string(),
                "name_contains_process".to_string(),
                "name_contains_convert".to_string(),
            ];
            let importance = model.feature_importance(&feature_names);

            println!("\n📊 Feature Importance (top 10):");
            for (name, weight) in importance.iter().take(10) {
                let direction = if *weight > 0.0 {
                    "→ ALIVE"
                } else {
                    "→ DEAD"
                };
                println!("   {}: {:.3} {}", name, weight, direction);
            }
        }
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        if let Some(model) = &self.model {
            let data = serde_json::to_string_pretty(model)
                .map_err(|e| format!("Failed to serialize: {}", e))?;
            std::fs::write(path, data).map_err(|e| format!("Failed to write model: {}", e))?;
            Ok(())
        } else {
            Err("No model trained".to_string())
        }
    }

    pub fn load(path: &str) -> Result<Self, String> {
        let data =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read model: {}", e))?;
        let model: LinearClassifier =
            serde_json::from_str(&data).map_err(|e| format!("Failed to parse model: {}", e))?;

        Ok(Self {
            model: Some(model),
            accuracy: 0.0,
        })
    }
}

impl Default for DeadCodeClassifier {
    fn default() -> Self {
        Self::new()
    }
}
