// src/ml/duplicate_classifier.rs

//! ML-based duplicate detection classifier

use crate::analysis::training_data::FunctionFeatures;
use crate::utils::serialization::{load_from_file, save_to_file};
use serde::{Deserialize, Serialize};

/// Simple linear classifier for duplicate detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateClassifier {
    weights: Vec<f64>,
    bias: f64,
    threshold: f64,
    feature_count: usize,
}

impl DuplicateClassifier {
    /// Create a new classifier with the given number of features
    pub fn new(feature_count: usize) -> Self {
        Self {
            weights: vec![0.0; feature_count],
            bias: 0.0,
            threshold: 0.7,
            feature_count,
        }
    }

    /// Train the classifier on labeled examples
    /// Returns the accuracy
    pub fn train(&mut self, examples: &[DuplicateExample]) -> f64 {
        if examples.is_empty() {
            return 0.0;
        }

        let feature_count = self.feature_count;
        let mut weights = vec![0.0; feature_count];
        let mut bias = 0.0;
        let learning_rate = 0.01;
        let epochs = 50;

        for epoch in 0..epochs {
            let mut total_loss = 0.0;

            for example in examples {
                let features = self.extract_features(example);
                let target = match example.label {
                    DuplicateLabel::Duplicate => 1.0,
                    DuplicateLabel::Similar => 0.5,
                    DuplicateLabel::NotDuplicate => 0.0,
                };

                // Forward pass (sigmoid)
                let dot: f64 = features.iter().zip(&weights).map(|(f, w)| f * w).sum();
                let prediction = 1.0 / (1.0 + (-(dot + bias)).exp());

                // Loss (binary cross-entropy)
                let loss = -target * prediction.ln() - (1.0 - target) * (1.0 - prediction).ln();
                total_loss += loss;

                // Backward pass (gradient descent)
                let error = prediction - target;

                for (i, &feature) in features.iter().enumerate() {
                    if i < weights.len() {
                        weights[i] -= learning_rate * error * feature;
                    }
                }
                bias -= learning_rate * error;
            }

            if epoch % 20 == 0 {
                let avg_loss = total_loss / examples.len() as f64;
                println!("   Epoch {}: loss = {:.4}", epoch, avg_loss);
            }
        }

        self.weights = weights;
        self.bias = bias;

        self.calculate_accuracy(examples)
    }

    /// Predict if two functions are duplicates (returns probability 0-1)
    pub fn predict(&self, a: &FunctionFeatures, b: &FunctionFeatures) -> f64 {
        let features = self.extract_features_pair(a, b);
        let dot: f64 = features.iter().zip(&self.weights).map(|(f, w)| f * w).sum();
        let mut probability = 1.0 / (1.0 + (-(dot + self.bias)).exp());

        // ⭐ NEW: Adjust probability based on type context
        // If they're on different types, reduce confidence
        if a.type_name.is_some() && b.type_name.is_some() && a.type_name != b.type_name {
            // Different types = likely not duplicates
            probability = (probability * 0.4) + (0.2 * 0.6);
        }

        // If they're on the same type, boost confidence
        if a.type_name.is_some() && b.type_name.is_some() && a.type_name == b.type_name {
            // Same type = more likely duplicates
            probability = (probability * 0.7) + (0.9 * 0.3);
        }

        // If one is a trait impl and the other isn't, reduce confidence
        if a.is_trait_impl != b.is_trait_impl {
            probability = (probability * 0.5) + (0.3 * 0.5);
        }

        probability
    }

    /// Check if two functions are duplicates (boolean)
    pub fn is_duplicate(&self, a: &FunctionFeatures, b: &FunctionFeatures) -> bool {
        self.predict(a, b) > self.threshold
    }

    /// Extract features from a pair of functions for prediction
    fn extract_features_pair(&self, a: &FunctionFeatures, b: &FunctionFeatures) -> Vec<f64> {
        let mut features = a.to_feature_vector();
        features.extend(b.to_feature_vector());

        // Add difference features
        let a_vec = a.to_feature_vector();
        let b_vec = b.to_feature_vector();
        let diff: Vec<f64> = a_vec
            .iter()
            .zip(b_vec.iter())
            .map(|(x, y)| (x - y).abs())
            .collect();

        features.extend(diff);

        // ⭐ NEW: Add type comparison features
        let type_same =
            if a.type_name.is_some() && b.type_name.is_some() && a.type_name == b.type_name {
                1.0
            } else {
                0.0
            };
        features.push(type_same);

        let trait_same =
            if a.trait_name.is_some() && b.trait_name.is_some() && a.trait_name == b.trait_name {
                1.0
            } else {
                0.0
            };
        features.push(trait_same);

        features
    }

    /// Extract features from a training example
    fn extract_features(&self, example: &DuplicateExample) -> Vec<f64> {
        let mut features = example.func_a.to_feature_vector();
        features.extend(example.func_b.to_feature_vector());

        let diff: Vec<f64> = example
            .func_a
            .to_feature_vector()
            .iter()
            .zip(example.func_b.to_feature_vector().iter())
            .map(|(x, y)| (x - y).abs())
            .collect();

        features.extend(diff);

        // ⭐ NEW: Add type comparison features
        let type_same = if example.func_a.type_name.is_some()
            && example.func_b.type_name.is_some()
            && example.func_a.type_name == example.func_b.type_name
        {
            1.0
        } else {
            0.0
        };
        features.push(type_same);

        let trait_same = if example.func_a.trait_name.is_some()
            && example.func_b.trait_name.is_some()
            && example.func_a.trait_name == example.func_b.trait_name
        {
            1.0
        } else {
            0.0
        };
        features.push(trait_same);

        features
    }

    /// Calculate accuracy on training data
    fn calculate_accuracy(&self, examples: &[DuplicateExample]) -> f64 {
        let mut correct = 0;
        let mut total = 0;

        for example in examples {
            let features = self.extract_features(example);
            let dot: f64 = features.iter().zip(&self.weights).map(|(f, w)| f * w).sum();
            let prediction = 1.0 / (1.0 + (-(dot + self.bias)).exp());

            let expected = match example.label {
                DuplicateLabel::Duplicate => 1.0,
                DuplicateLabel::Similar => 0.5,
                DuplicateLabel::NotDuplicate => 0.0,
            };

            if (prediction - expected).abs() < 0.3 {
                correct += 1;
            }
            total += 1;
        }

        if total == 0 {
            0.0
        } else {
            correct as f64 / total as f64
        }
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        save_to_file(self, path)
    }

    pub fn load(path: &str) -> Result<Self, String> {
        load_from_file(path)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateExample {
    pub func_a: FunctionFeatures,
    pub func_b: FunctionFeatures,
    pub label: DuplicateLabel,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DuplicateLabel {
    Duplicate,    // These are duplicates
    Similar,      // Similar but not duplicates
    NotDuplicate, // These are not duplicates
}

impl Default for DuplicateClassifier {
    fn default() -> Self {
        Self::new(101) // 33 * 3 + 2 type features = 101
    }
}
