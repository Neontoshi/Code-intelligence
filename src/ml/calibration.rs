// src/ml/calibration.rs

//! Confidence calibration for ML predictions
//!
//! This module provides tools to calibrate raw model probabilities
//! so they better reflect actual accuracy.

use crate::analysis::training_data::{TrainingExample, TrainingLabel};
use crate::ml::classifier::{DeadCodeClassifier, LinearClassifier};
use serde::{Deserialize, Serialize};

// ============================================================================
// Calibrated Model
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibratedModel {
    pub classifier: LinearClassifier,
    pub calibration: CalibrationParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationParams {
    pub temperature: f64,
    pub bins: Vec<CalibrationBin>,
    pub method: CalibrationMethod,
    pub num_samples: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationBin {
    pub lower: f64,
    pub upper: f64,
    pub empirical_accuracy: f64,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum CalibrationMethod {
    TemperatureScaling,
    HistogramBinning,
    IsotonicRegression,
    #[default]
    None,
}

impl CalibratedModel {
    /// Calibrate a model using validation data
    pub fn calibrate(
        classifier: &LinearClassifier,
        val_examples: &[TrainingExample],
        method: CalibrationMethod,
    ) -> Self {
        match method {
            CalibrationMethod::TemperatureScaling => {
                Self::calibrate_temperature(classifier, val_examples)
            }
            CalibrationMethod::HistogramBinning => {
                Self::calibrate_histogram(classifier, val_examples)
            }
            CalibrationMethod::IsotonicRegression => {
                Self::calibrate_isotonic(classifier, val_examples)
            }
            CalibrationMethod::None => Self {
                classifier: classifier.clone(),
                calibration: CalibrationParams {
                    temperature: 1.0,
                    bins: Vec::new(),
                    method: CalibrationMethod::None,
                    num_samples: 0,
                },
            },
        }
    }

    /// Temperature scaling: learn a single temperature parameter
    fn calibrate_temperature(
        classifier: &LinearClassifier,
        val_examples: &[TrainingExample],
    ) -> Self {
        let labeled: Vec<_> = val_examples
            .iter()
            .filter(|e| e.label != TrainingLabel::Unknown)
            .collect();

        if labeled.is_empty() {
            return Self {
                classifier: classifier.clone(),
                calibration: CalibrationParams {
                    temperature: 1.0,
                    bins: Vec::new(),
                    method: CalibrationMethod::TemperatureScaling,
                    num_samples: labeled.len(),
                },
            };
        }

        // Find optimal temperature using grid search
        let mut best_temp = 1.0;
        let mut best_loss = f64::MAX;

        for temp in (5..=20).map(|t| t as f64 / 10.0) {
            let mut loss = 0.0;
            for example in &labeled {
                let features = example.features.to_feature_vector();
                let raw_prob = classifier.predict(&features);
                let calibrated = 1.0 / (1.0 + (-(-((1.0 - raw_prob).ln() / temp))).exp());
                let target = match example.label {
                    TrainingLabel::Alive => 1.0,
                    TrainingLabel::Dead => 0.0,
                    _ => 0.5,
                };
                loss += -target * calibrated.ln() - (1.0 - target) * (1.0 - calibrated).ln();
            }

            if loss < best_loss {
                best_loss = loss;
                best_temp = temp;
            }
        }

        Self {
            classifier: classifier.clone(),
            calibration: CalibrationParams {
                temperature: best_temp,
                bins: Vec::new(),
                method: CalibrationMethod::TemperatureScaling,
                num_samples: labeled.len(),
            },
        }
    }

    /// Histogram binning: group predictions into bins
    fn calibrate_histogram(
        classifier: &LinearClassifier,
        val_examples: &[TrainingExample],
    ) -> Self {
        let labeled: Vec<_> = val_examples
            .iter()
            .filter(|e| e.label != TrainingLabel::Unknown)
            .collect();

        let mut bins = Vec::new();
        let num_bins = 10;

        if labeled.is_empty() {
            return Self {
                classifier: classifier.clone(),
                calibration: CalibrationParams {
                    temperature: 1.0,
                    bins: Vec::new(),
                    method: CalibrationMethod::HistogramBinning,
                    num_samples: 0,
                },
            };
        }

        // Compute raw predictions
        let mut preds: Vec<(f64, f64)> = labeled
            .iter()
            .map(|e| {
                let features = e.features.to_feature_vector();
                let prob = classifier.predict(&features);
                let target = match e.label {
                    TrainingLabel::Alive => 1.0,
                    TrainingLabel::Dead => 0.0,
                    _ => 0.5,
                };
                (prob, target)
            })
            .collect();

        preds.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let bin_size = preds.len() / num_bins;
        for i in 0..num_bins {
            let start = i * bin_size;
            let end = if i == num_bins - 1 {
                preds.len()
            } else {
                start + bin_size
            };

            if start < preds.len() {
                let slice = &preds[start..end];
                let lower = slice.first().map(|p| p.0).unwrap_or(0.0);
                let upper = slice.last().map(|p| p.0).unwrap_or(1.0);
                let empirical_accuracy =
                    slice.iter().map(|p| p.1).sum::<f64>() / slice.len() as f64;

                bins.push(CalibrationBin {
                    lower,
                    upper,
                    empirical_accuracy,
                    count: slice.len(),
                });
            }
        }

        Self {
            classifier: classifier.clone(),
            calibration: CalibrationParams {
                temperature: 1.0,
                bins,
                method: CalibrationMethod::HistogramBinning,
                num_samples: labeled.len(),
            },
        }
    }

    /// Isotonic regression (simplified - using piecewise linear)
    fn calibrate_isotonic(classifier: &LinearClassifier, val_examples: &[TrainingExample]) -> Self {
        // For simplicity, use histogram binning as a proxy for isotonic regression
        Self::calibrate_histogram(classifier, val_examples)
    }

    /// Predict with calibration
    pub fn predict_calibrated(&self, features: &[f64]) -> f64 {
        let raw = self.classifier.predict(features);

        match self.calibration.method {
            CalibrationMethod::TemperatureScaling => {
                // Apply temperature scaling
                1.0 / (1.0 + (-(-((1.0 - raw).ln()) / self.calibration.temperature)).exp())
            }
            CalibrationMethod::HistogramBinning => {
                // Find which bin the prediction falls into
                for bin in &self.calibration.bins {
                    if raw >= bin.lower && raw < bin.upper {
                        return bin.empirical_accuracy;
                    }
                }
                // If outside all bins, use raw
                raw
            }
            _ => raw,
        }
    }

    /// Get calibration statistics
    pub fn calibration_stats(&self, examples: &[TrainingExample]) -> CalibrationStats {
        let labeled: Vec<_> = examples
            .iter()
            .filter(|e| e.label != TrainingLabel::Unknown)
            .collect();

        if labeled.is_empty() {
            return CalibrationStats::default();
        }

        let mut total_bins = 0;
        let mut total_errors = 0.0;
        let mut miscalibration = 0.0;

        for bin in &self.calibration.bins {
            total_bins += 1;
            let bin_center = (bin.lower + bin.upper) / 2.0;
            let error = (bin.empirical_accuracy - bin_center).abs();
            total_errors += error * bin.count as f64;
            miscalibration += error;
        }

        CalibrationStats {
            expected_calibration_error: total_errors / labeled.len() as f64,
            miscalibration_area: miscalibration / total_bins as f64,
            num_bins: total_bins,
            method: self.calibration.method.clone(),
        }
    }
}

// ============================================================================
// Calibration Stats
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct CalibrationStats {
    pub expected_calibration_error: f64, // ECE - lower is better
    pub miscalibration_area: f64,
    pub num_bins: usize,
    pub method: CalibrationMethod,
}

impl CalibrationStats {
    pub fn is_well_calibrated(&self) -> bool {
        self.expected_calibration_error < 0.05
    }

    pub fn print(&self) {
        println!("   Method: {:?}", self.method);
        println!(
            "   Expected Calibration Error: {:.2}%",
            self.expected_calibration_error * 100.0
        );
        println!(
            "   Miscalibration Area: {:.2}%",
            self.miscalibration_area * 100.0
        );
        println!("   Bins: {}", self.num_bins);
        println!("   Well calibrated: {}", self.is_well_calibrated());
    }
}

// ============================================================================
// Calibration Wrapper for DeadCodeClassifier
// ============================================================================

impl DeadCodeClassifier {
    /// Calibrate the model using validation data
    pub fn calibrate(&mut self, val_examples: &[TrainingExample]) -> Result<(), String> {
        if let Some(model) = &self.model {
            let calibrated = CalibratedModel::calibrate(
                model,
                val_examples,
                CalibrationMethod::TemperatureScaling,
            );
            self.model = Some(calibrated.classifier);
            self.calibration = Some(calibrated.calibration);
            Ok(())
        } else {
            Err("No model to calibrate".to_string())
        }
    }

    pub fn predict_calibrated(&self, features: &[f64]) -> f64 {
        let raw = self.predict_features(features);

        // If we have calibration parameters, apply them
        if let Some(calibration) = &self.calibration {
            match calibration.method {
                CalibrationMethod::TemperatureScaling => {
                    // Apply temperature scaling
                    1.0 / (1.0 + (-(-((1.0 - raw).ln()) / calibration.temperature)).exp())
                }
                CalibrationMethod::HistogramBinning => {
                    // Find which bin the prediction falls into
                    for bin in &calibration.bins {
                        if raw >= bin.lower && raw < bin.upper {
                            return bin.empirical_accuracy;
                        }
                    }
                    raw
                }
                CalibrationMethod::IsotonicRegression => {
                    // For now, fall back to histogram binning
                    for bin in &calibration.bins {
                        if raw >= bin.lower && raw < bin.upper {
                            return bin.empirical_accuracy;
                        }
                    }
                    raw
                }
                CalibrationMethod::None => raw,
            }
        } else {
            // No calibration available, return raw prediction
            raw
        }
    }

    /// Predict calibrated probability that the function is ALIVE
    pub fn predict_alive_calibrated(&self, example: &TrainingExample) -> f64 {
        let features = example.features.to_feature_vector();
        self.predict_calibrated(&features)
    }

    /// Predict calibrated probability that the function is DEAD
    pub fn predict_dead_calibrated(&self, example: &TrainingExample) -> f64 {
        1.0 - self.predict_alive_calibrated(example)
    }
}
