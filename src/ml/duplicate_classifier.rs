// src/ml/duplicate_classifier.rs

//! ML-based duplicate detection classifier

use crate::analysis::training_data::FunctionFeatures;
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
        debug_assert_eq!(
            features.len(),
            self.weights.len(),
            "DuplicateClassifier: feature vector length {} != weight count {}",
            features.len(),
            self.weights.len()
        );
        let dot: f64 = features.iter().zip(&self.weights).map(|(f, w)| f * w).sum();

        // type_same / trait_same are already part of the trained feature
        // vector (see extract_features_pair). Don't re-apply hand-tuned
        // adjustments on top of a trained model — that overrides whatever
        // the weights actually learned with an untrained, hardcoded prior,
        // and it's exactly what was suppressing genuine cross-type trait-impl
        // duplicates (see: LLMProvider implementations across ollama/openai/mock).
        1.0 / (1.0 + (-(dot + self.bias)).exp())
    }

    /// Check if two functions are duplicates (boolean)
    pub fn is_duplicate(&self, a: &FunctionFeatures, b: &FunctionFeatures) -> bool {
        self.predict(a, b) > self.threshold
    }

    /// Extract features from a pair of functions for prediction.
    /// Duplicate-ness is a symmetric relation, so we canonicalize the
    /// pair order (by full_path) before building the vector — otherwise
    /// is_duplicate(a, b) and is_duplicate(b, a) can disagree.
    fn extract_features_pair(&self, a: &FunctionFeatures, b: &FunctionFeatures) -> Vec<f64> {
        // FunctionFeatures (training_data) has no full_path/name field — use
        // body_hash as the stable ordering key so extract_features_pair(a, b)
        // and extract_features_pair(b, a) always canonicalize the same way.
        let (a, b) = if a.body_hash <= b.body_hash {
            (a, b)
        } else {
            (b, a)
        };
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

    /// Extract features from a training example (order-canonicalized —
    fn extract_features(&self, example: &DuplicateExample) -> Vec<f64> {
        let (a, b) = if example.func_a.body_hash <= example.func_b.body_hash {
            (&example.func_a, &example.func_b)
        } else {
            (&example.func_b, &example.func_a)
        };

        let mut features = a.to_feature_vector();
        features.extend(b.to_feature_vector());

        let diff: Vec<f64> = a
            .to_feature_vector()
            .iter()
            .zip(b.to_feature_vector().iter())
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

    /// Evaluate accuracy on any labeled examples — pass a held-out split
    /// (not the training data) to get a meaningful generalization estimate.
    pub fn evaluate(&self, examples: &[DuplicateExample]) -> f64 {
        self.calculate_accuracy(examples)
    }

    /// Calculate accuracy on the given examples (used both for the
    /// train-time self-report and for `evaluate`)
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
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> crate::error::Result<()> {
        crate::ml::serialization::ModelSerializer::save_binary(self, path)
    }

    pub fn load<P: AsRef<std::path::Path>>(path: P) -> crate::error::Result<Self> {
        crate::ml::serialization::ModelSerializer::load_auto(path)
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
        // 3 vectors (a, b, |a-b| diff) using the live feature schema,
        // plus 2 type-comparison features appended in extract_features_pair.
        Self::new(crate::ml::feature_schema::feature_count() * 3 + 2)
    }
}
