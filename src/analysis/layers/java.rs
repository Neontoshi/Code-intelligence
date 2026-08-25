// src/analysis/layers/java.rs

//! Java-specific layer detection

use crate::analysis::layers::{LayerDetector, LayerInfo};
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashMap;

pub struct JavaLayerDetector;

impl LayerDetector for JavaLayerDetector {
    fn detect_layer(&self, file: &ParsedFile) -> String {
        let path = &file.path;
        let parts: Vec<&str> = path.split('/').collect();

        // Java-specific patterns
        if parts.len() >= 2 {
            match parts[parts.len() - 2] {
                "controller" | "controllers" => "controller".to_string(),
                "service" | "services" => "service".to_string(),
                "repository" | "repositories" | "dao" => "repository".to_string(),
                "model" | "models" | "entity" => "model".to_string(),
                "dto" | "dtos" => "dto".to_string(),
                "config" | "configuration" => "config".to_string(),
                "util" | "utils" | "helper" => "utility".to_string(),
                "exception" | "exceptions" | "error" => "exception".to_string(),
                "aspect" | "aspects" => "aspect".to_string(),
                "interceptor" | "interceptors" => "interceptor".to_string(),
                "filter" | "filters" => "filter".to_string(),
                "listener" | "listeners" => "listener".to_string(),
                "scheduled" | "jobs" | "tasks" => "scheduler".to_string(),
                "security" | "auth" => "security".to_string(),
                "test" | "tests" => "test".to_string(),
                _ => {
                    // Java-specific patterns
                    if path.contains("/src/main/java/") {
                        // Extract package name parts
                        if let Some(java_idx) = parts.iter().position(|&p| p == "java") {
                            if java_idx + 1 < parts.len() - 2 {
                                let package_parts = &parts[java_idx + 1..parts.len() - 1];
                                if !package_parts.is_empty() {
                                    return package_parts.join(".");
                                }
                            }
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
            "controller".to_string(),
            LayerInfo {
                name: "controller".to_string(),
                description: "REST API controllers".to_string(),
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
            "model".to_string(),
            LayerInfo {
                name: "model".to_string(),
                description: "Data models and entities".to_string(),
                color: "#a855f7".to_string(),
            },
        );

        info.insert(
            "dto".to_string(),
            LayerInfo {
                name: "dto".to_string(),
                description: "Data Transfer Objects".to_string(),
                color: "#06b6d4".to_string(),
            },
        );

        info.insert(
            "config".to_string(),
            LayerInfo {
                name: "config".to_string(),
                description: "Configuration and settings".to_string(),
                color: "#ec4899".to_string(),
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
            "exception".to_string(),
            LayerInfo {
                name: "exception".to_string(),
                description: "Exception handling".to_string(),
                color: "#f43f5e".to_string(),
            },
        );

        info.insert(
            "aspect".to_string(),
            LayerInfo {
                name: "aspect".to_string(),
                description: "Aspect-oriented programming".to_string(),
                color: "#8b5cf6".to_string(),
            },
        );

        info.insert(
            "interceptor".to_string(),
            LayerInfo {
                name: "interceptor".to_string(),
                description: "Request/response interceptors".to_string(),
                color: "#f472b6".to_string(),
            },
        );

        info.insert(
            "filter".to_string(),
            LayerInfo {
                name: "filter".to_string(),
                description: "Request filters".to_string(),
                color: "#fb923c".to_string(),
            },
        );

        info.insert(
            "listener".to_string(),
            LayerInfo {
                name: "listener".to_string(),
                description: "Event listeners".to_string(),
                color: "#d946ef".to_string(),
            },
        );

        info.insert(
            "scheduler".to_string(),
            LayerInfo {
                name: "scheduler".to_string(),
                description: "Scheduled jobs and tasks".to_string(),
                color: "#f97316".to_string(),
            },
        );

        info.insert(
            "security".to_string(),
            LayerInfo {
                name: "security".to_string(),
                description: "Security and authentication".to_string(),
                color: "#e11d48".to_string(),
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
