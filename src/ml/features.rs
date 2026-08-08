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

    pub fn get_feature_names() -> Vec<String> {
        vec![
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
        ]
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
}
