// src/analysis/layers/javascript.rs

//! JavaScript-specific layer detection

use crate::analysis::layers::{LayerDetector, LayerInfo};
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashMap;

pub struct JavaScriptLayerDetector;

impl LayerDetector for JavaScriptLayerDetector {
    fn detect_layer(&self, file: &ParsedFile) -> String {
        let path = &file.path;
        let parts: Vec<&str> = path.split('/').collect();

        // JavaScript-specific patterns
        if parts.len() >= 2 {
            match parts[parts.len() - 2] {
                "components" => "component".to_string(),
                "pages" | "views" => "page".to_string(),
                "hooks" => "hook".to_string(),
                "services" => "service".to_string(),
                "utils" | "util" | "helpers" | "common" => "utility".to_string(),
                "api" => "api".to_string(),
                "routes" => "route".to_string(),
                "middleware" => "middleware".to_string(),
                "config" | "configuration" => "config".to_string(),
                "stores" | "store" | "state" => "state".to_string(),
                "models" | "types" => "model".to_string(),
                "tests" | "test" | "__tests__" => "test".to_string(),
                "workers" | "jobs" => "worker".to_string(),
                _ => {
                    // JavaScript-specific patterns
                    if path.ends_with(".test.js") || path.ends_with(".spec.js") {
                        return "test".to_string();
                    }
                    if path.ends_with(".stories.js") || path.ends_with(".stories.jsx") {
                        return "story".to_string();
                    }
                    if path.ends_with("/index.js") || path.ends_with("/index.jsx") {
                        if path.contains("/components/") {
                            return "component".to_string();
                        }
                        if path.contains("/pages/") {
                            return "page".to_string();
                        }
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
            "component".to_string(),
            LayerInfo {
                name: "component".to_string(),
                description: "UI components".to_string(),
                color: "#38bdf8".to_string(),
            },
        );

        info.insert(
            "page".to_string(),
            LayerInfo {
                name: "page".to_string(),
                description: "Page-level components".to_string(),
                color: "#10b981".to_string(),
            },
        );

        info.insert(
            "hook".to_string(),
            LayerInfo {
                name: "hook".to_string(),
                description: "React hooks and custom hooks".to_string(),
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
            "route".to_string(),
            LayerInfo {
                name: "route".to_string(),
                description: "Route definitions".to_string(),
                color: "#06b6d4".to_string(),
            },
        );

        info.insert(
            "middleware".to_string(),
            LayerInfo {
                name: "middleware".to_string(),
                description: "Request/response middleware".to_string(),
                color: "#ec4899".to_string(),
            },
        );

        info.insert(
            "config".to_string(),
            LayerInfo {
                name: "config".to_string(),
                description: "Configuration and settings".to_string(),
                color: "#f472b6".to_string(),
            },
        );

        info.insert(
            "state".to_string(),
            LayerInfo {
                name: "state".to_string(),
                description: "State management (Redux, Zustand, etc.)".to_string(),
                color: "#8b5cf6".to_string(),
            },
        );

        info.insert(
            "model".to_string(),
            LayerInfo {
                name: "model".to_string(),
                description: "Data models and type definitions".to_string(),
                color: "#f97316".to_string(),
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
            "story".to_string(),
            LayerInfo {
                name: "story".to_string(),
                description: "Storybook stories".to_string(),
                color: "#fb923c".to_string(),
            },
        );

        info.insert(
            "worker".to_string(),
            LayerInfo {
                name: "worker".to_string(),
                description: "Web workers and background jobs".to_string(),
                color: "#e11d48".to_string(),
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
