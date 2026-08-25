// src/analysis/layers/csharp.rs

//! C#/.NET-specific layer detection

use crate::analysis::layers::{LayerDetector, LayerInfo};
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashMap;

pub struct CSharpLayerDetector;

impl LayerDetector for CSharpLayerDetector {
    fn detect_layer(&self, file: &ParsedFile) -> String {
        let path = &file.path;
        let parts: Vec<&str> = path.split('/').collect();

        // C#/.NET-specific patterns
        if parts.len() >= 2 {
            match parts[parts.len() - 2] {
                "Controllers" | "controllers" => "controller".to_string(),
                "Services" | "services" => "service".to_string(),
                "Models" | "models" => "model".to_string(),
                "Repositories" | "repositories" | "Data" => "repository".to_string(),
                "Middleware" | "middleware" => "middleware".to_string(),
                "Configuration" | "config" => "config".to_string(),
                "Helpers" | "helpers" | "Utils" | "utils" => "utility".to_string(),
                "Providers" | "providers" => "provider".to_string(),
                "Migrations" | "migrations" => "migration".to_string(),
                "Tests" | "tests" => "test".to_string(),
                "Views" | "views" => "view".to_string(),
                "Pages" | "pages" => "page".to_string(),
                "Components" | "components" => "component".to_string(),
                "Events" | "events" => "event".to_string(),
                "Jobs" | "jobs" | "Workers" | "workers" => "worker".to_string(),
                "Hubs" | "hubs" => "hub".to_string(),
                "Filters" | "filters" => "filter".to_string(),
                _ => {
                    // C#/.NET-specific patterns
                    if path.ends_with(".cs") && path.contains("/Controllers/") {
                        return "controller".to_string();
                    }
                    if path.ends_with(".cs") && path.contains("/Models/") {
                        return "model".to_string();
                    }
                    if path.ends_with(".cs") && path.contains("/Services/") {
                        return "service".to_string();
                    }
                    if path.ends_with(".cs") && path.contains("/Data/") {
                        return "repository".to_string();
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
            "controller".to_string(),
            LayerInfo {
                name: "controller".to_string(),
                description: "API controllers".to_string(),
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
            "model".to_string(),
            LayerInfo {
                name: "model".to_string(),
                description: "Data models and entities".to_string(),
                color: "#a855f7".to_string(),
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
                color: "#ec4899".to_string(),
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
            "utility".to_string(),
            LayerInfo {
                name: "utility".to_string(),
                description: "Shared helper functions".to_string(),
                color: "#64748b".to_string(),
            },
        );

        info.insert(
            "provider".to_string(),
            LayerInfo {
                name: "provider".to_string(),
                description: "Service providers".to_string(),
                color: "#8b5cf6".to_string(),
            },
        );

        info.insert(
            "migration".to_string(),
            LayerInfo {
                name: "migration".to_string(),
                description: "Database migrations".to_string(),
                color: "#d946ef".to_string(),
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
            "view".to_string(),
            LayerInfo {
                name: "view".to_string(),
                description: "View templates and Razor pages".to_string(),
                color: "#94a3b8".to_string(),
            },
        );

        info.insert(
            "page".to_string(),
            LayerInfo {
                name: "page".to_string(),
                description: "Razor Pages".to_string(),
                color: "#f472b6".to_string(),
            },
        );

        info.insert(
            "component".to_string(),
            LayerInfo {
                name: "component".to_string(),
                description: "Blazor components".to_string(),
                color: "#fb923c".to_string(),
            },
        );

        info.insert(
            "event".to_string(),
            LayerInfo {
                name: "event".to_string(),
                description: "Event definitions".to_string(),
                color: "#f97316".to_string(),
            },
        );

        info.insert(
            "worker".to_string(),
            LayerInfo {
                name: "worker".to_string(),
                description: "Background workers and jobs".to_string(),
                color: "#e11d48".to_string(),
            },
        );

        info.insert(
            "hub".to_string(),
            LayerInfo {
                name: "hub".to_string(),
                description: "SignalR hubs".to_string(),
                color: "#6366f1".to_string(),
            },
        );

        info.insert(
            "filter".to_string(),
            LayerInfo {
                name: "filter".to_string(),
                description: "Action filters".to_string(),
                color: "#f43f5e".to_string(),
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
