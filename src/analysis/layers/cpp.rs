// src/analysis/layers/cpp.rs

//! C++-specific layer detection

use crate::analysis::layers::{LayerDetector, LayerInfo};
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashMap;

pub struct CppLayerDetector;

impl LayerDetector for CppLayerDetector {
    fn detect_layer(&self, file: &ParsedFile) -> String {
        let path = &file.path;
        let parts: Vec<&str> = path.split('/').collect();

        // C++-specific patterns
        if parts.len() >= 2 {
            match parts[parts.len() - 2] {
                "handlers" | "controllers" => "handler".to_string(),
                "services" | "domain" => "service".to_string(),
                "models" | "repositories" => "repository".to_string(),
                "middleware" => "middleware".to_string(),
                "config" | "configuration" => "config".to_string(),
                "workers" | "jobs" => "worker".to_string(),
                "telemetry" | "metrics" => "observability".to_string(),
                "auth" | "security" => "auth".to_string(),
                "utils" | "util" | "helpers" | "common" => "utility".to_string(),
                "api" => "api".to_string(),
                "cli" => "cli".to_string(),
                "tests" | "test" => "test".to_string(),
                "include" => "header".to_string(),
                "src" => "source".to_string(),
                "lib" => "library".to_string(),
                _ => {
                    // C++-specific patterns
                    if path.ends_with(".h") || path.ends_with(".hpp") {
                        if path.contains("/include/") {
                            return "header".to_string();
                        }
                        return "header".to_string();
                    }
                    if path.ends_with(".cpp") || path.ends_with(".cc") || path.ends_with(".cxx") {
                        if path.contains("/src/") {
                            return "source".to_string();
                        }
                        return "source".to_string();
                    }

                    // Fallback
                    if let Some(src_idx) = parts.iter().position(|&p| p == "src") {
                        if src_idx + 1 < parts.len() - 1 {
                            return parts[src_idx + 1].to_string();
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
            "handler".to_string(),
            LayerInfo {
                name: "handler".to_string(),
                description: "HTTP handlers and controllers".to_string(),
                color: "#38bdf8".to_string(),
            },
        );

        info.insert(
            "service".to_string(),
            LayerInfo {
                name: "service".to_string(),
                description: "Business logic services".to_string(),
                color: "#10b981".to_string(),
            },
        );

        info.insert(
            "repository".to_string(),
            LayerInfo {
                name: "repository".to_string(),
                description: "Data access layer".to_string(),
                color: "#f59e0b".to_string(),
            },
        );

        info.insert(
            "middleware".to_string(),
            LayerInfo {
                name: "middleware".to_string(),
                description: "Request/response middleware".to_string(),
                color: "#a855f7".to_string(),
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
            "worker".to_string(),
            LayerInfo {
                name: "worker".to_string(),
                description: "Background job processors".to_string(),
                color: "#f97316".to_string(),
            },
        );

        info.insert(
            "observability".to_string(),
            LayerInfo {
                name: "observability".to_string(),
                description: "Monitoring and telemetry".to_string(),
                color: "#ec4899".to_string(),
            },
        );

        info.insert(
            "auth".to_string(),
            LayerInfo {
                name: "auth".to_string(),
                description: "Authentication and authorization".to_string(),
                color: "#e11d48".to_string(),
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
            "api".to_string(),
            LayerInfo {
                name: "api".to_string(),
                description: "API interface layer".to_string(),
                color: "#6366f1".to_string(),
            },
        );

        info.insert(
            "cli".to_string(),
            LayerInfo {
                name: "cli".to_string(),
                description: "Command-line interface".to_string(),
                color: "#475569".to_string(),
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
            "header".to_string(),
            LayerInfo {
                name: "header".to_string(),
                description: "Header files (declarations)".to_string(),
                color: "#8b5cf6".to_string(),
            },
        );

        info.insert(
            "source".to_string(),
            LayerInfo {
                name: "source".to_string(),
                description: "Source files (implementations)".to_string(),
                color: "#f472b6".to_string(),
            },
        );

        info.insert(
            "library".to_string(),
            LayerInfo {
                name: "library".to_string(),
                description: "Library code".to_string(),
                color: "#fb923c".to_string(),
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
