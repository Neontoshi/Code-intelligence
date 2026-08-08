// src/ml/features.rs

use crate::analysis::training_data::{TrainingExample, TrainingLabel};
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};

pub struct FeatureExtractor {
    scaler: Option<FeatureScaler>,
}

impl FeatureExtractor {
    pub fn new() -> Self {
        Self { scaler: None }
    }

    /// Delegates to the canonical schema (`crate::ml::feature_schema`) so
    /// this can never drift out of sync with the actual 46-wide feature
    /// vector again.
    pub fn get_feature_names() -> Vec<String> {
        crate::ml::feature_schema::feature_names()
    }

    pub fn extract_features(&self, example: &TrainingExample) -> Array1<f64> {
        let vec = example.features.to_feature_vector();
        Array1::from(vec)
    }

    pub fn extract_batch(&self, examples: &[TrainingExample]) -> (Array2<f64>, Array1<f64>) {
        let mut features = Vec::new();
        let mut targets = Vec::new();

        for example in examples {
            if example.label != TrainingLabel::Unknown {
                features.push(self.extract_features(example).to_vec());
                targets.push(match example.label {
                    TrainingLabel::Alive => 1.0,
                    TrainingLabel::Dead => 0.0,
                    TrainingLabel::Unknown => 0.5,
                });
            }
        }

        let n_samples = features.len();
        let n_features = if n_samples > 0 { features[0].len() } else { 33 };

        let flat_features: Vec<f64> = features.into_iter().flatten().collect();
        let features_array = Array2::from_shape_vec((n_samples, n_features), flat_features)
            .unwrap_or_else(|_| Array2::zeros((0, n_features)));

        let targets_array = Array1::from(targets);

        (features_array, targets_array)
    }

    pub fn fit_scaler(&mut self, examples: &[TrainingExample]) {
        let (features, _) = self.extract_batch(examples);
        if features.nrows() > 0 {
            let mut scaler = FeatureScaler::new();
            scaler.fit(&features);
            self.scaler = Some(scaler);
        }
    }
}

impl Default for FeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureScaler {
    pub mean: Vec<f64>,
    pub std: Vec<f64>,
}

impl FeatureScaler {
    pub fn new() -> Self {
        Self {
            mean: Vec::new(),
            std: Vec::new(),
        }
    }

    pub fn fit(&mut self, features: &Array2<f64>) {
        let n_features = features.ncols();
        let mut mean = Array1::zeros(n_features);
        let mut std = Array1::zeros(n_features);

        for j in 0..n_features {
            let col = features.column(j);
            let col_mean = col.mean().unwrap_or(0.0);
            let col_std = col.std(0.0);

            mean[j] = col_mean;
            std[j] = if col_std > 1e-10 { col_std } else { 1.0 };
        }

        self.mean = mean.into_iter().collect();
        self.std = std.into_iter().collect();
    }

    /// Convenience fit for callers that have a flat list of raw feature
    /// vectors rather than an ndarray Array2 already built (e.g. LinearClassifier).
    pub fn fit_from_vectors(&mut self, features: &[Vec<f64>]) {
        if features.is_empty() {
            return;
        }
        let n_samples = features.len();
        let n_features = features[0].len();
        let flat: Vec<f64> = features.iter().flatten().copied().collect();
        if let Ok(arr) = Array2::from_shape_vec((n_samples, n_features), flat) {
            self.fit(&arr);
        }
    }

    /// Standardize a raw feature vector using the fitted mean/std.
    /// Returns the input unchanged if the scaler hasn't been fit yet, or
    /// if the vector length doesn't match what was fit (defensive —
    /// should not happen in normal use).
    pub fn transform(&self, features: &[f64]) -> Vec<f64> {
        if self.mean.is_empty() || self.mean.len() != features.len() {
            return features.to_vec();
        }
        features
            .iter()
            .zip(self.mean.iter().zip(self.std.iter()))
            .map(|(&x, (&m, &s))| (x - m) / s)
            .collect()
    }
}
