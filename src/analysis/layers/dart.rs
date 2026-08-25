// src/analysis/layers/dart.rs

//! Dart/Flutter-specific layer detection

use crate::analysis::layers::{LayerDetector, LayerInfo};
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashMap;

pub struct DartLayerDetector;

impl LayerDetector for DartLayerDetector {
    fn detect_layer(&self, file: &ParsedFile) -> String {
        let path = &file.path;
        let parts: Vec<&str> = path.split('/').collect();

        // Dart/Flutter-specific patterns
        if parts.len() >= 2 {
            match parts[parts.len() - 2] {
                "widgets" | "widget" => "widget".to_string(),
                "pages" | "screens" | "views" => "page".to_string(),
                "models" => "model".to_string(),
                "services" => "service".to_string(),
                "repositories" | "repo" => "repository".to_string(),
                "providers" | "provider" => "provider".to_string(),
                "blocs" | "cubits" | "state" => "state".to_string(),
                "utils" | "util" | "helpers" | "common" => "utility".to_string(),
                "config" | "configuration" => "config".to_string(),
                "api" => "api".to_string(),
                "tests" | "test" => "test".to_string(),
                "themes" | "theme" => "theme".to_string(),
                "assets" => "assets".to_string(),
                _ => {
                    // Dart-specific patterns
                    if path.ends_with("_test.dart") {
                        return "test".to_string();
                    }
                    if path.contains("/lib/src/") {
                        return "internal".to_string();
                    }

                    // Fallback
                    if let Some(lib_idx) = parts.iter().position(|&p| p == "lib") {
                        if lib_idx + 1 < parts.len() - 1 {
                            return parts[lib_idx + 1].to_string();
                        }
                    }
                    "core".to_string()
                }
            }
        } else {
            "root".to_string()
        }
    }

    fn get_layer_info(&self) -> HashMap<String, LayerInfo> {
        let mut info = HashMap::new();

        info.insert(
            "widget".to_string(),
            LayerInfo {
                name: "widget".to_string(),
                description: "Flutter UI widgets".to_string(),
                color: "#38bdf8".to_string(),
            },
        );

        info.insert(
            "page".to_string(),
            LayerInfo {
                name: "page".to_string(),
                description: "Page/screen components".to_string(),
                color: "#10b981".to_string(),
            },
        );

        info.insert(
            "model".to_string(),
            LayerInfo {
                name: "model".to_string(),
                description: "Data models".to_string(),
                color: "#a855f7".to_string(),
            },
        );

        info.insert(
            "service".to_string(),
            LayerInfo {
                name: "service".to_string(),
                description: "Business logic services".to_string(),
                color: "#f59e0b".to_string(),
            },
        );

        info.insert(
            "repository".to_string(),
            LayerInfo {
                name: "repository".to_string(),
                description: "Data access layer".to_string(),
                color: "#f97316".to_string(),
            },
        );

        info.insert(
            "provider".to_string(),
            LayerInfo {
                name: "provider".to_string(),
                description: "Provider/DI containers".to_string(),
                color: "#ec4899".to_string(),
            },
        );

        info.insert(
            "state".to_string(),
            LayerInfo {
                name: "state".to_string(),
                description: "State management (Bloc, Cubit, Riverpod)".to_string(),
                color: "#8b5cf6".to_string(),
            },
        );

        info.insert(
            "utility".to_string(),
            LayerInfo {
                name: "utility".to_string(),
                description: "Shared helper functions".to_string(),
                color: "#64748b".to_string(),
            },
        );

        info.insert(
            "config".to_string(),
            LayerInfo {
                name: "config".to_string(),
                description: "Configuration and settings".to_string(),
                color: "#06b6d4".to_string(),
            },
        );

        info.insert(
            "api".to_string(),
            LayerInfo {
                name: "api".to_string(),
                description: "API interface layer".to_string(),
                color: "#6366f1".to_string(),
            },
        );

        info.insert(
            "test".to_string(),
            LayerInfo {
                name: "test".to_string(),
                description: "Test code".to_string(),
                color: "#d946ef".to_string(),
            },
        );

        info.insert(
            "theme".to_string(),
            LayerInfo {
                name: "theme".to_string(),
                description: "Theme and styling".to_string(),
                color: "#f472b6".to_string(),
            },
        );

        info.insert(
            "assets".to_string(),
            LayerInfo {
                name: "assets".to_string(),
                description: "Static assets".to_string(),
                color: "#94a3b8".to_string(),
            },
        );

        info.insert(
            "internal".to_string(),
            LayerInfo {
                name: "internal".to_string(),
                description: "Internal implementation details".to_string(),
                color: "#475569".to_string(),
            },
        );

        info.insert(
            "core".to_string(),
            LayerInfo {
                name: "core".to_string(),
                description: "Core application code".to_string(),
                color: "#475569".to_string(),
            },
        );

        info.insert(
            "root".to_string(),
            LayerInfo {
                name: "root".to_string(),
                description: "Top-level project files".to_string(),
                color: "#eab308".to_string(),
            },
        );

        info
    }
}
