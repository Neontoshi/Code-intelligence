// src/analysis/layers/php.rs

//! PHP-specific layer detection

use crate::analysis::layers::{LayerDetector, LayerInfo};
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashMap;

pub struct PhpLayerDetector;

impl LayerDetector for PhpLayerDetector {
    fn detect_layer(&self, file: &ParsedFile) -> String {
        let path = &file.path;
        let parts: Vec<&str> = path.split('/').collect();

        // PHP-specific patterns
        if parts.len() >= 2 {
            match parts[parts.len() - 2] {
                "Controllers" | "controllers" => "controller".to_string(),
                "Services" | "services" => "service".to_string(),
                "Models" | "models" => "model".to_string(),
                "Repositories" | "repositories" => "repository".to_string(),
                "Middleware" | "middleware" => "middleware".to_string(),
                "Config" | "config" => "config".to_string(),
                "Helpers" | "helpers" | "Utils" | "utils" => "utility".to_string(),
                "Providers" | "providers" => "provider".to_string(),
                "Listeners" | "listeners" => "listener".to_string(),
                "Events" | "events" => "event".to_string(),
                "Jobs" | "jobs" => "job".to_string(),
                "Migrations" | "migrations" => "migration".to_string(),
                "Seeders" | "seeders" => "seeder".to_string(),
                "Tests" | "tests" => "test".to_string(),
                "Views" | "views" => "view".to_string(),
                _ => {
                    // PHP-specific patterns (Laravel, Symfony)
                    if path.contains("/app/Http/Controllers/") {
                        return "controller".to_string();
                    }
                    if path.contains("/app/Models/") {
                        return "model".to_string();
                    }
                    if path.contains("/app/Services/") {
                        return "service".to_string();
                    }
                    if path.contains("/app/Repositories/") {
                        return "repository".to_string();
                    }
                    if path.contains("/database/migrations/") {
                        return "migration".to_string();
                    }
                    if path.contains("/database/seeders/") {
                        return "seeder".to_string();
                    }
                    if path.contains("/routes/") {
                        return "route".to_string();
                    }

                    // Fallback
                    if let Some(app_idx) = parts.iter().position(|&p| p == "app") {
                        if app_idx + 1 < parts.len() - 1 {
                            return parts[app_idx + 1].to_string();
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
                description: "HTTP controllers".to_string(),
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
                description: "Data models".to_string(),
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
            "listener".to_string(),
            LayerInfo {
                name: "listener".to_string(),
                description: "Event listeners".to_string(),
                color: "#f472b6".to_string(),
            },
        );

        info.insert(
            "event".to_string(),
            LayerInfo {
                name: "event".to_string(),
                description: "Event definitions".to_string(),
                color: "#fb923c".to_string(),
            },
        );

        info.insert(
            "job".to_string(),
            LayerInfo {
                name: "job".to_string(),
                description: "Queue jobs".to_string(),
                color: "#f97316".to_string(),
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
            "seeder".to_string(),
            LayerInfo {
                name: "seeder".to_string(),
                description: "Database seeders".to_string(),
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
            "view".to_string(),
            LayerInfo {
                name: "view".to_string(),
                description: "View templates".to_string(),
                color: "#94a3b8".to_string(),
            },
        );

        info.insert(
            "route".to_string(),
            LayerInfo {
                name: "route".to_string(),
                description: "Route definitions".to_string(),
                color: "#6366f1".to_string(),
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
