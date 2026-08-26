// src/ml/feature_schema.rs

//! Single source of truth for ML feature definitions
//!
//! This schema defines ALL features used by the ML model.
//! Version 2 includes language-agnostic features for all 10 supported languages.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// CORE TYPES

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
    Language,
    Framework,
    Decorator,
    Dynamic,
    ErrorHandling,
    Concurrency,
    Documentation,
    Testing,
    Visibility,
    Ownership,
    Generics,
    Pattern,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Normalization {
    None,
    MinMax { min: f64, max: f64 },
    Standard,
    Scale { factor: f64 },
}

// FEATURE SCHEMA IMPLEMENTATION
impl FeatureSchema {
    pub fn new() -> Self {
        let mut features = Vec::new();
        let mut index = 0;

        // 1. GRAPH FEATURES (4)
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

        // 2. SIGNATURE FEATURES (8)
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
            description: "Function is public/exported".to_string(),
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

        features.push(FeatureDefinition {
            name: "is_generator".to_string(),
            index,
            description: "Function is a generator (yield)".to_string(),
            category: FeatureCategory::Signature,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "is_static".to_string(),
            index,
            description: "Function is static/class method".to_string(),
            category: FeatureCategory::Signature,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "is_abstract".to_string(),
            index,
            description: "Function is abstract/virtual".to_string(),
            category: FeatureCategory::Signature,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "is_override".to_string(),
            index,
            description: "Function overrides parent method".to_string(),
            category: FeatureCategory::Signature,
            normalization: Normalization::None,
        });
        index += 1;

        // 3. COMPLEXITY FEATURES (4)
        features.push(FeatureDefinition {
            name: "cyclomatic_complexity".to_string(),
            index,
            description: "Cyclomatic complexity".to_string(),
            category: FeatureCategory::Complexity,
            normalization: Normalization::MinMax {
                min: 0.0,
                max: 50.0,
            },
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "cognitive_complexity".to_string(),
            index,
            description: "Cognitive complexity (nesting depth)".to_string(),
            category: FeatureCategory::Complexity,
            normalization: Normalization::MinMax {
                min: 0.0,
                max: 20.0,
            },
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "line_count".to_string(),
            index,
            description: "Number of lines in function body".to_string(),
            category: FeatureCategory::Complexity,
            normalization: Normalization::MinMax {
                min: 0.0,
                max: 100.0,
            },
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "token_count".to_string(),
            index,
            description: "Number of tokens in function".to_string(),
            category: FeatureCategory::Complexity,
            normalization: Normalization::MinMax {
                min: 0.0,
                max: 500.0,
            },
        });
        index += 1;

        // 4. NAME PATTERN FEATURES (110)
        let contains_patterns = vec![
            // Original patterns
            "use",
            "test",
            "init",
            "get",
            "set",
            "new",
            "create",
            "build",
            "parse",
            "validate",
            "handle",
            "process",
            "convert",
            "commit",
            "reveal",
            "submit",
            "upload",
            "download",
            "fetch",
            "verify",
            "audit",
            // Language-agnostic patterns
            "main",
            "start",
            "run",
            "load",
            "save",
            "read",
            "write",
            "open",
            "close",
            "connect",
            "send",
            "receive",
            "delete",
            "update",
            "patch",
            "put",
            "post",
            "get",
            "list",
            "find",
            "search",
            "filter",
            "map",
            "reduce",
            "clone",
            "copy",
            "move",
            "swap",
            "sort",
            "is",
            "has",
            "can",
            "should",
            "will",
            "do",
            "make",
            "take",
            "give",
            "call",
            "apply",
            "register",
            "unregister",
            "subscribe",
            "unsubscribe",
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

        // Name starts with patterns
        let start_patterns = vec![
            "use", "test", "bench", "get", "set", "is", "has", "can", "should", "will", "on",
            "handle", "process", "parse", "create", "build", "make", "do", "apply",
        ];

        for pattern in start_patterns {
            features.push(FeatureDefinition {
                name: format!("starts_with_{}", pattern),
                index,
                description: format!("Name starts with '{}'", pattern),
                category: FeatureCategory::Name,
                normalization: Normalization::None,
            });
            index += 1;
        }

        // Name ends with patterns
        let end_patterns = vec![
            "test",
            "handler",
            "processor",
            "service",
            "repository",
            "controller",
            "manager",
            "factory",
            "builder",
            "validator",
            "converter",
            "mapper",
            "filter",
            "loader",
            "saver",
            "creator",
            "updater",
            "deleter",
            "finder",
            "parser",
            "renderer",
            "serializer",
        ];

        for pattern in end_patterns {
            features.push(FeatureDefinition {
                name: format!("ends_with_{}", pattern),
                index,
                description: format!("Name ends with '{}'", pattern),
                category: FeatureCategory::Name,
                normalization: Normalization::None,
            });
            index += 1;
        }

        // Name length
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

        // 5. LANGUAGE FEATURES (10)
        let languages = vec![
            "rust",
            "python",
            "javascript",
            "typescript",
            "go",
            "java",
            "dart",
            "php",
            "cpp",
            "csharp",
        ];

        for lang in languages {
            features.push(FeatureDefinition {
                name: format!("is_{}", lang),
                index,
                description: format!("Function is written in {}", lang),
                category: FeatureCategory::Language,
                normalization: Normalization::None,
            });
            index += 1;
        }

        // 6. FRAMEWORK FEATURES (23)
        let framework_patterns = vec![
            // Web frameworks
            "flask_route",
            "fastapi_route",
            "express_route",
            "nextjs_route",
            "spring_controller",
            "aspnet_controller",
            "laravel_controller",
            "django_view",
            "rails_action",
            // UI frameworks
            "react_component",
            "react_hook",
            "vue_component",
            "svelte_component",
            "flutter_widget",
            "flutter_state",
            // Go
            "go_init",
            "go_interface",
            "go_goroutine",
            // Rust
            "rust_trait_impl",
            "rust_ffi",
        ];

        for pattern in framework_patterns {
            features.push(FeatureDefinition {
                name: format!("is_{}", pattern),
                index,
                description: format!("Function uses '{}' pattern", pattern),
                category: FeatureCategory::Framework,
                normalization: Normalization::None,
            });
            index += 1;
        }

        // 7. TYPE FEATURES (12)
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
            description: "Function implements a trait/interface".to_string(),
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
            name: "has_receiver".to_string(),
            index,
            description: "Function has a receiver (Go methods)".to_string(),
            category: FeatureCategory::Type,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "has_self".to_string(),
            index,
            description: "Function uses self/this".to_string(),
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
        index += 1;

        features.push(FeatureDefinition {
            name: "has_generics".to_string(),
            index,
            description: "Function/type uses generics".to_string(),
            category: FeatureCategory::Generics,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "generic_count".to_string(),
            index,
            description: "Number of generic parameters".to_string(),
            category: FeatureCategory::Generics,
            normalization: Normalization::MinMax { min: 0.0, max: 5.0 },
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "has_type_annotation".to_string(),
            index,
            description: "Function has explicit type annotations".to_string(),
            category: FeatureCategory::Type,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "has_lifetime".to_string(),
            index,
            description: "Function has lifetime parameters (Rust)".to_string(),
            category: FeatureCategory::Type,
            normalization: Normalization::None,
        });
        index += 1;

        // 8. FILE CONTEXT FEATURES (10)
        let file_patterns = vec![
            "is_in_test_file",
            "is_in_benches",
            "is_in_meta",
            "is_in_examples",
            "is_generated",
            "is_in_lib",
            "is_in_bin",
            "is_in_proto",
            "is_in_migrations",
            "is_in_fixtures",
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

        // 9. DECORATOR FEATURES (19)
        let decorator_patterns = vec![
            "route",
            "get",
            "post",
            "put",
            "delete",
            "patch",
            "override",
            "staticmethod",
            "classmethod",
            "property",
            "cached_property",
            "pytest",
            "fixture",
            "parametrize",
            "test",
        ];

        for pattern in decorator_patterns {
            features.push(FeatureDefinition {
                name: format!("has_decorator_{}", pattern),
                index,
                description: format!("Function has decorator '{}'", pattern),
                category: FeatureCategory::Decorator,
                normalization: Normalization::None,
            });
            index += 1;
        }

        // 10. DYNAMIC BEHAVIOR FEATURES (7)
        features.push(FeatureDefinition {
            name: "has_dynamic_call".to_string(),
            index,
            description: "Function contains dynamic calls (reflection)".to_string(),
            category: FeatureCategory::Dynamic,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "has_ffi".to_string(),
            index,
            description: "Function uses FFI/extern".to_string(),
            category: FeatureCategory::Dynamic,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "has_macro".to_string(),
            index,
            description: "Function uses macros".to_string(),
            category: FeatureCategory::Dynamic,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "has_closure".to_string(),
            index,
            description: "Function contains closures/lambdas".to_string(),
            category: FeatureCategory::Dynamic,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "has_yield".to_string(),
            index,
            description: "Function uses yield (generator)".to_string(),
            category: FeatureCategory::Dynamic,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "has_await".to_string(),
            index,
            description: "Function uses await".to_string(),
            category: FeatureCategory::Dynamic,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "has_thread".to_string(),
            index,
            description: "Function spawns threads/go routines".to_string(),
            category: FeatureCategory::Concurrency,
            normalization: Normalization::None,
        });
        index += 1;

        // 11. ERROR HANDLING FEATURES (6)
        features.push(FeatureDefinition {
            name: "has_try_catch".to_string(),
            index,
            description: "Function has try/catch/error handling".to_string(),
            category: FeatureCategory::ErrorHandling,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "has_result_type".to_string(),
            index,
            description: "Function returns Result/Option/Either".to_string(),
            category: FeatureCategory::ErrorHandling,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "has_throw".to_string(),
            index,
            description: "Function can throw exceptions".to_string(),
            category: FeatureCategory::ErrorHandling,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "has_panic".to_string(),
            index,
            description: "Function can panic/unwrap".to_string(),
            category: FeatureCategory::ErrorHandling,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "has_question_mark".to_string(),
            index,
            description: "Function uses ? operator (Rust)".to_string(),
            category: FeatureCategory::ErrorHandling,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "has_error_propagation".to_string(),
            index,
            description: "Function propagates errors".to_string(),
            category: FeatureCategory::ErrorHandling,
            normalization: Normalization::None,
        });
        index += 1;

        // 12. DOCUMENTATION FEATURES (3)
        features.push(FeatureDefinition {
            name: "has_doc_comment".to_string(),
            index,
            description: "Function has documentation comment".to_string(),
            category: FeatureCategory::Documentation,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "doc_comment_length".to_string(),
            index,
            description: "Length of doc comment".to_string(),
            category: FeatureCategory::Documentation,
            normalization: Normalization::MinMax {
                min: 0.0,
                max: 100.0,
            },
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "has_attr_doc".to_string(),
            index,
            description: "Function has attribute doc".to_string(),
            category: FeatureCategory::Documentation,
            normalization: Normalization::None,
        });
        index += 1;

        // 13. VISIBILITY FEATURES (5)
        let visibility_patterns =
            vec!["pub_crate", "pub_super", "pub_self", "private", "protected"];

        for pattern in visibility_patterns {
            features.push(FeatureDefinition {
                name: format!("vis_{}", pattern),
                index,
                description: format!("Visibility: {}", pattern),
                category: FeatureCategory::Visibility,
                normalization: Normalization::None,
            });
            index += 1;
        }

        // 14. OWNERSHIP FEATURES (4)
        features.push(FeatureDefinition {
            name: "has_borrow".to_string(),
            index,
            description: "Function uses borrow (Rust)".to_string(),
            category: FeatureCategory::Ownership,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "has_mut_ref".to_string(),
            index,
            description: "Function uses mutable reference".to_string(),
            category: FeatureCategory::Ownership,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "has_move".to_string(),
            index,
            description: "Function uses move semantics".to_string(),
            category: FeatureCategory::Ownership,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "has_clone".to_string(),
            index,
            description: "Function clones data".to_string(),
            category: FeatureCategory::Ownership,
            normalization: Normalization::None,
        });
        index += 1;

        // 15. PATTERN FEATURES (6)
        let pattern_types = vec![
            "singleton",
            "factory",
            "builder",
            "observer",
            "strategy",
            "decorator",
        ];

        for pattern in pattern_types {
            features.push(FeatureDefinition {
                name: format!("pattern_{}", pattern),
                index,
                description: format!("Design pattern: {}", pattern),
                category: FeatureCategory::Pattern,
                normalization: Normalization::None,
            });
            index += 1;
        }

        // 16. CONCURRENCY FEATURES (4)
        features.push(FeatureDefinition {
            name: "has_channel".to_string(),
            index,
            description: "Function uses channels".to_string(),
            category: FeatureCategory::Concurrency,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "has_mutex".to_string(),
            index,
            description: "Function uses mutex/lock".to_string(),
            category: FeatureCategory::Concurrency,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "has_atomic".to_string(),
            index,
            description: "Function uses atomic operations".to_string(),
            category: FeatureCategory::Concurrency,
            normalization: Normalization::None,
        });
        index += 1;

        features.push(FeatureDefinition {
            name: "has_parallel".to_string(),
            index,
            description: "Function uses parallel processing".to_string(),
            category: FeatureCategory::Concurrency,
            normalization: Normalization::None,
        });

        // Build the name-to-index map
        let name_to_index = features.iter().map(|f| (f.name.clone(), f.index)).collect();

        Self {
            version: 2,
            features,
            name_to_index,
        }
    }

    // PUBLIC METHODS
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

        for (i, &value) in vector.iter().enumerate() {
            if value.is_nan() {
                return Err(format!("Feature {} has NaN value", i));
            }
            if value.is_infinite() {
                return Err(format!("Feature {} has infinite value", i));
            }

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

    pub fn has_feature(&self, name: &str) -> bool {
        self.name_to_index.contains_key(name)
    }

    pub fn print_summary(&self) {
        println!("📊 Feature Schema v{}", self.version);
        println!("   Total features: {}", self.feature_count());
        println!("   Categories:");

        let categories = vec![
            FeatureCategory::Graph,
            FeatureCategory::Signature,
            FeatureCategory::Complexity,
            FeatureCategory::Name,
            FeatureCategory::Language,
            FeatureCategory::Framework,
            FeatureCategory::Type,
            FeatureCategory::File,
            FeatureCategory::Decorator,
            FeatureCategory::Dynamic,
            FeatureCategory::ErrorHandling,
            FeatureCategory::Documentation,
            FeatureCategory::Visibility,
            FeatureCategory::Ownership,
            FeatureCategory::Generics,
            FeatureCategory::Pattern,
            FeatureCategory::Concurrency,
        ];

        for category in &categories {
            let count = self.get_by_category(category).len();
            if count > 0 {
                println!("      {:?}: {}", category, count);
            }
        }
        println!();
    }
}

// SINGLETON INSTANCE

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

// FEATURE VECTOR BUILDER

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

    pub fn push_language(&mut self, language: &str) -> &mut Self {
        let languages = vec![
            "rust",
            "python",
            "javascript",
            "typescript",
            "go",
            "java",
            "dart",
            "php",
            "cpp",
            "csharp",
        ];
        for lang in languages {
            self.features.push(if lang == language { 1.0 } else { 0.0 });
        }
        self
    }

    pub fn build(self) -> Vec<f64> {
        debug_assert_eq!(
            self.features.len(),
            feature_count(),
            "Feature vector length mismatch: expected {}, got {}",
            feature_count(),
            self.features.len()
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

// TESTS

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_schema_count() {
        let schema = FeatureSchema::new();
        assert_eq!(schema.feature_count(), 235);
        println!("✅ Total features: {}", schema.feature_count());
    }

    #[test]
    fn test_get_feature() {
        let schema = FeatureSchema::new();
        assert!(schema.get_feature("fan_in").is_some());
        assert!(schema.get_feature("is_python").is_some());
        assert!(schema.get_feature("has_decorator_route").is_some());
        assert!(schema.get_feature("nonexistent").is_none());
    }

    #[test]
    fn test_get_by_category() {
        let schema = FeatureSchema::new();
        let graph_features = schema.get_by_category(&FeatureCategory::Graph);
        assert_eq!(graph_features.len(), 4);

        let language_features = schema.get_by_category(&FeatureCategory::Language);
        assert_eq!(language_features.len(), 10);

        let framework_features = schema.get_by_category(&FeatureCategory::Framework);
        assert!(framework_features.len() >= 20);
    }

    #[test]
    fn test_feature_vector_builder() {
        let mut builder = FeatureVectorBuilder::new();
        builder
            .push(1.0)
            .push_bool(true)
            .push_normalized(25.0, 50.0)
            .push_opt(Some(0.5), 0.0)
            .push_language("rust");

        let features = builder.build_unchecked();
        assert!(features.len() >= 5);
    }

    #[test]
    fn test_schema_compatibility() {
        let schema1 = FeatureSchema::new();
        let schema2 = FeatureSchema::new();
        assert!(schema1.assert_compatible(&schema2).is_ok());
    }

    #[test]
    fn test_print_summary() {
        let schema = FeatureSchema::new();
        schema.print_summary();
        // Just verify it doesn't panic
        assert!(true);
    }
}
