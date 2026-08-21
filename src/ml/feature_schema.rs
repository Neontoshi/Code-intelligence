// src/ml/feature_schema.rs

//! Single source of truth for ML feature definitions

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSchema {
    pub version: u32,
    pub features: Vec<FeatureDefinition>,
    #[serde(skip)]
    name_to_index: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDefinition {
    pub name: String,
    pub index: usize,
    pub description: String,
    pub category: FeatureCategory,
    pub normalization: Normalization,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FeatureCategory {
    Graph,
    Signature,
    Name,
    File,
    Type,
    Complexity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Normalization {
    None,
    MinMax { min: f64, max: f64 },
    Standard,
    Scale { factor: f64 },
}

impl FeatureSchema {
    pub fn new() -> Self {
        let mut features = Vec::new();
        let mut index = 0;

        // Graph features (4)
        features.push(FeatureDefinition {
            name: "fan_in".to_string(),
            index,
            description: "Number of callers".to_string(),
            category: FeatureCategory::Graph,
            normalization: Normalization::MinMax {
                min: 0.0,
                max: 50.0,
            },
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "fan_out".to_string(),
            index,
            description: "Number of callees".to_string(),
            category: FeatureCategory::Graph,
            normalization: Normalization::MinMax {
                min: 0.0,
                max: 50.0,
            },
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "call_depth".to_string(),
            index,
            description: "Depth in call tree".to_string(),
            category: FeatureCategory::Graph,
            normalization: Normalization::MinMax {
                min: 0.0,
                max: 10.0,
            },
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "is_cycle".to_string(),
            index,
            description: "Part of a cycle".to_string(),
            category: FeatureCategory::Graph,
            normalization: Normalization::None,
        });
        index += 1;

        // Signature features (4)
        features.push(FeatureDefinition {
            name: "param_count".to_string(),
            index,
            description: "Number of parameters".to_string(),
            category: FeatureCategory::Signature,
            normalization: Normalization::MinMax {
                min: 0.0,
                max: 10.0,
            },
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "return_count".to_string(),
            index,
            description: "Number of return values".to_string(),
            category: FeatureCategory::Signature,
            normalization: Normalization::MinMax { min: 0.0, max: 5.0 },
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "is_public".to_string(),
            index,
            description: "Function is public".to_string(),
            category: FeatureCategory::Signature,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "is_async".to_string(),
            index,
            description: "Function is async".to_string(),
            category: FeatureCategory::Signature,
            normalization: Normalization::None,
        });
        index += 1;

        // Complexity (1)
        features.push(FeatureDefinition {
            name: "complexity".to_string(),
            index,
            description: "Cyclomatic complexity".to_string(),
            category: FeatureCategory::Complexity,
            normalization: Normalization::MinMax {
                min: 0.0,
                max: 50.0,
            },
        });
        index += 1;

        // Name contains patterns (21)
        let contains_patterns = vec![
            "use", "test", "init", "get", "set", "new", "create", "build", "parse", "validate",
            "handle", "process", "convert", "commit", "reveal", "submit", "upload", "download",
            "fetch", "verify", "audit",
        ];

        for pattern in contains_patterns {
            features.push(FeatureDefinition {
                name: format!("contains_{}", pattern),
                index,
                description: format!("Name contains '{}'", pattern),
                category: FeatureCategory::Name,
                normalization: Normalization::None,
            });
            index += 1;
        }

        // Name starts/ends (5)
        features.push(FeatureDefinition {
            name: "starts_with_use".to_string(),
            index,
            description: "Name starts with 'use'".to_string(),
            category: FeatureCategory::Name,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "starts_with_test".to_string(),
            index,
            description: "Name starts with 'test_'".to_string(),
            category: FeatureCategory::Name,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "starts_with_bench".to_string(),
            index,
            description: "Name starts with 'bench_'".to_string(),
            category: FeatureCategory::Name,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "ends_with_test".to_string(),
            index,
            description: "Name ends with '_test'".to_string(),
            category: FeatureCategory::Name,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "name_length".to_string(),
            index,
            description: "Length of function name".to_string(),
            category: FeatureCategory::Name,
            normalization: Normalization::MinMax {
                min: 0.0,
                max: 50.0,
            },
        });
        index += 1;

        // File context (5)
        let file_patterns = vec![
            "is_in_test_file",
            "is_in_benches",
            "is_in_meta",
            "is_in_examples",
            "is_generated",
        ];

        for pattern in file_patterns {
            features.push(FeatureDefinition {
                name: pattern.to_string(),
                index,
                description: format!("File context: {}", pattern.replace('_', " ")),
                category: FeatureCategory::File,
                normalization: Normalization::None,
            });
            index += 1;
        }

        // Type context (6)
        features.push(FeatureDefinition {
            name: "is_method".to_string(),
            index,
            description: "Function is a method".to_string(),
            category: FeatureCategory::Type,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "is_trait_impl".to_string(),
            index,
            description: "Function implements a trait".to_string(),
            category: FeatureCategory::Type,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "is_associated".to_string(),
            index,
            description: "Function is associated with a type".to_string(),
            category: FeatureCategory::Type,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "type_name_length".to_string(),
            index,
            description: "Length of type name".to_string(),
            category: FeatureCategory::Type,
            normalization: Normalization::MinMax {
                min: 0.0,
                max: 20.0,
            },
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "trait_name_length".to_string(),
            index,
            description: "Length of trait name".to_string(),
            category: FeatureCategory::Type,
            normalization: Normalization::MinMax {
                min: 0.0,
                max: 20.0,
            },
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "type_and_trait_match".to_string(),
            index,
            description: "Type and trait names match".to_string(),
            category: FeatureCategory::Type,
            normalization: Normalization::None,
        });

        let name_to_index = features.iter().map(|f| (f.name.clone(), f.index)).collect();

        Self {
            version: 1,
            features,
            name_to_index,
        }
    }

    pub fn feature_names(&self) -> Vec<String> {
        self.features.iter().map(|f| f.name.clone()).collect()
    }

    pub fn feature_count(&self) -> usize {
        self.features.len()
    }

    pub fn get_feature(&self, name: &str) -> Option<&FeatureDefinition> {
        self.name_to_index
            .get(name)
            .and_then(|&idx| self.features.get(idx))
    }

    pub fn get_by_category(&self, category: &FeatureCategory) -> Vec<&FeatureDefinition> {
        self.features
            .iter()
            .filter(|f| &f.category == category)
            .collect()
    }

    pub fn validate_vector(&self, vector: &[f64]) -> Result<(), String> {
        let expected = self.feature_count();
        if vector.len() != expected {
            return Err(format!(
                "Feature vector length {} does not match schema count {}",
                vector.len(),
                expected
            ));
        }

        // Check for NaN or infinite values
        for (i, &value) in vector.iter().enumerate() {
            if value.is_nan() {
                return Err(format!("Feature {} has NaN value", i));
            }
            if value.is_infinite() {
                return Err(format!("Feature {} has infinite value", i));
            }
            // Check value is in reasonable range based on normalization
            let feature = &self.features[i];
            match feature.normalization {
                Normalization::None => {
                    if value != 0.0 && value != 1.0 {
                        return Err(format!(
                            "Feature '{}' should be 0 or 1, got {}",
                            feature.name, value
                        ));
                    }
                }
                Normalization::MinMax { min, max } => {
                    if value < min || value > max {
                        return Err(format!(
                            "Feature '{}' value {} outside range [{}, {}]",
                            feature.name, value, min, max
                        ));
                    }
                }
                Normalization::Standard => {
                    // Standard normalization can produce values outside [-3, 3] but rarely
                    if value.abs() > 5.0 {
                        return Err(format!(
                            "Feature '{}' value {} unusually large for standard normalization",
                            feature.name, value
                        ));
                    }
                }
                Normalization::Scale { factor } => {
                    if value < -factor || value > factor {
                        return Err(format!(
                            "Feature '{}' value {} outside range [-{}, {}]",
                            feature.name, value, factor, factor
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    pub fn assert_compatible(&self, other: &FeatureSchema) -> Result<(), String> {
        if self.version != other.version {
            return Err(format!(
                "Schema version mismatch: {} vs {}",
                self.version, other.version
            ));
        }

        if self.feature_count() != other.feature_count() {
            return Err(format!(
                "Feature count mismatch: {} vs {}",
                self.feature_count(),
                other.feature_count()
            ));
        }

        // Check each feature name
        for (i, (a, b)) in self.features.iter().zip(other.features.iter()).enumerate() {
            if a.name != b.name {
                return Err(format!(
                    "Feature name mismatch at index {}: '{}' vs '{}'",
                    i, a.name, b.name
                ));
            }
            if a.category != b.category {
                return Err(format!(
                    "Feature category mismatch at index {}: '{:?}' vs '{:?}'",
                    i, a.category, b.category
                ));
            }
        }

        Ok(())
    }

    // ⭐ NEW: Check if a feature exists
    pub fn has_feature(&self, name: &str) -> bool {
        self.name_to_index.contains_key(name)
    }

    // ⭐ NEW: Print schema summary
    pub fn print_summary(&self) {
        println!("📊 Feature Schema v{}", self.version);
        println!("   Total features: {}", self.feature_count());
        println!("   Categories:");
        let categories = [
            FeatureCategory::Graph,
            FeatureCategory::Signature,
            FeatureCategory::Complexity,
            FeatureCategory::Name,
            FeatureCategory::File,
            FeatureCategory::Type,
        ];
        for category in &categories {
            let count = self.get_by_category(category).len();
            println!("      {:?}: {}", category, count);
        }
        println!();
    }
}

impl Default for FeatureSchema {
    fn default() -> Self {
        Self::new()
    }
}

pub static FEATURE_SCHEMA: Lazy<FeatureSchema> = Lazy::new(FeatureSchema::new);

pub fn feature_names() -> Vec<String> {
    FEATURE_SCHEMA.feature_names()
}

pub fn feature_count() -> usize {
    FEATURE_SCHEMA.feature_count()
}

pub fn features_by_category(category: &FeatureCategory) -> Vec<&'static FeatureDefinition> {
    FEATURE_SCHEMA.get_by_category(category)
}

// ============================================================================
// Feature Vector Builder
// ============================================================================

#[derive(Debug, Clone)]
pub struct FeatureVectorBuilder {
    features: Vec<f64>,
}

impl FeatureVectorBuilder {
    pub fn new() -> Self {
        Self {
            features: Vec::with_capacity(feature_count()),
        }
    }

    pub fn push(&mut self, value: f64) -> &mut Self {
        self.features.push(value);
        self
    }

    pub fn push_bool(&mut self, value: bool) -> &mut Self {
        self.features.push(if value { 1.0 } else { 0.0 });
        self
    }

    pub fn push_normalized(&mut self, value: f64, max: f64) -> &mut Self {
        self.features.push(if max > 0.0 {
            (value / max).min(1.0)
        } else {
            0.0
        });
        self
    }

    pub fn push_opt(&mut self, value: Option<f64>, default: f64) -> &mut Self {
        self.features.push(value.unwrap_or(default));
        self
    }

    pub fn build(self) -> Vec<f64> {
        debug_assert_eq!(
            self.features.len(),
            feature_count(),
            "Feature vector length mismatch"
        );
        self.features
    }

    pub fn build_unchecked(self) -> Vec<f64> {
        self.features
    }
}

impl Default for FeatureVectorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_schema_count() {
        let schema = FeatureSchema::new();
        assert_eq!(schema.feature_count(), 46);
    }

    #[test]
    fn test_get_feature() {
        let schema = FeatureSchema::new();
        assert!(schema.get_feature("fan_in").is_some());
        assert!(schema.get_feature("nonexistent").is_none());
    }

    #[test]
    fn test_get_by_category() {
        let schema = FeatureSchema::new();
        let graph_features = schema.get_by_category(&FeatureCategory::Graph);
        assert_eq!(graph_features.len(), 4);
    }

    #[test]
    fn test_feature_vector_builder() {
        let mut builder = FeatureVectorBuilder::new();
        // Push all 46 features (or test with build_unchecked)
        builder
            .push(1.0)
            .push_bool(true)
            .push_normalized(25.0, 50.0)
            .push_opt(Some(0.5), 0.0);
        let features = builder.build_unchecked(); // Use unchecked for testing
        assert_eq!(features.len(), 4);
        assert_eq!(features[0], 1.0);
        assert_eq!(features[1], 1.0);
        assert_eq!(features[2], 0.5);
        assert_eq!(features[3], 0.5);
    }
}
