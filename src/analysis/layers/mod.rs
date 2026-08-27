// src/analysis/layers/mod.rs

//! Unified architectural layer detection

pub mod common;

pub use common::*;

use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LayerInfo {
    pub name: String,
    pub description: String,
    pub color: String,
}

pub struct LayerDetector {
    layer_info: HashMap<String, LayerInfo>,
}

impl LayerDetector {
    pub fn new() -> Self {
        let mut info = HashMap::new();

        let layers = [
            (
                "handler",
                "Receives requests and kicks off the work",
                "#38bdf8",
            ),
            (
                "service",
                "Business logic — the actual rules of the app",
                "#10b981",
            ),
            ("repository", "Reads and writes data storage", "#f59e0b"),
            (
                "middleware",
                "Runs on every request before it reaches handlers",
                "#a855f7",
            ),
            ("config", "Settings and environment setup", "#06b6d4"),
            ("worker", "Background jobs and scheduled tasks", "#f97316"),
            (
                "blockchain",
                "On-chain / smart contract interactions",
                "#f43f5e",
            ),
            ("observability", "Logging, metrics, tracing", "#ec4899"),
            ("auth", "Login, permissions, access control", "#e11d48"),
            ("utility", "Shared helper functions", "#64748b"),
            ("api", "External-facing interface", "#6366f1"),
            ("cli", "Command-line entry points", "#475569"),
            ("test", "Test code", "#d946ef"),
            ("bench", "Benchmark code", "#fb923c"),
            ("library", "Library code", "#8b5cf6"),
            ("example", "Example code", "#f472b6"),
            ("views", "View templates and rendering", "#f472b6"),
            ("templates", "HTML templates", "#fb923c"),
            ("static", "Static assets", "#94a3b8"),
            ("admin", "Admin interface", "#8b5cf6"),
            ("routes", "URL routing", "#06b6d4"),
            ("migration", "Database migrations", "#d946ef"),
            ("internal", "Internal packages", "#64748b"),
            ("package", "Public package code", "#8b5cf6"),
            ("core", "Core application code", "#475569"),
            ("root", "Top-level project files", "#eab308"),
        ];

        for (name, description, color) in layers {
            info.insert(
                name.to_string(),
                LayerInfo {
                    name: name.to_string(),
                    description: description.to_string(),
                    color: color.to_string(),
                },
            );
        }

        Self { layer_info: info }
    }

    pub fn detect_layer(&self, file: &ParsedFile) -> String {
        let path = &file.path;
        let language = file.language.to_lowercase();

        // Language-specific patterns first
        match language.as_str() {
            "rust" => {
                if path.contains("/src/bin/") {
                    return "cli".to_string();
                }
                if path.contains("/src/lib/") || path.ends_with("lib.rs") {
                    return "library".to_string();
                }
                if path.contains("/examples/") {
                    return "example".to_string();
                }
                if path.contains("/benches/") {
                    return "bench".to_string();
                }
            }
            "go" => {
                if path.contains("/cmd/") {
                    return "cli".to_string();
                }
                if path.contains("/internal/") {
                    return "internal".to_string();
                }
                if path.contains("/pkg/") {
                    return "package".to_string();
                }
                if path.ends_with("_test.go") {
                    return "test".to_string();
                }
            }
            "python" => {
                if path.contains("/management/commands/") {
                    return "cli".to_string();
                }
                if path.contains("/admin/") {
                    return "admin".to_string();
                }
                if path.ends_with("urls.py") {
                    return "routes".to_string();
                }
                if path.ends_with("settings.py") {
                    return "config".to_string();
                }
                if path.contains("/views/") {
                    return "views".to_string();
                }
                if path.contains("/templates/") {
                    return "templates".to_string();
                }
                if path.contains("/static/") {
                    return "static".to_string();
                }
                if path.contains("/migrations/") {
                    return "migration".to_string();
                }
            }
            "java" => {
                if path.contains("/controller/") || path.contains("/controllers/") {
                    return "handler".to_string();
                }
                if path.contains("/service/") || path.contains("/services/") {
                    return "service".to_string();
                }
                if path.contains("/repository/") || path.contains("/repositories/") {
                    return "repository".to_string();
                }
            }
            "csharp" => {
                if path.contains("/Controllers/") {
                    return "handler".to_string();
                }
                if path.contains("/Services/") {
                    return "service".to_string();
                }
                if path.contains("/Repositories/") {
                    return "repository".to_string();
                }
            }
            _ => {}
        }

        // Generic detection (fallback)
        detect_layer_generic(file)
    }

    pub fn get_layer_info(&self) -> HashMap<String, LayerInfo> {
        self.layer_info.clone()
    }

    pub fn get_layer_color(&self, layer: &str) -> String {
        self.layer_info
            .get(layer)
            .map(|info| info.color.clone())
            .unwrap_or_else(|| layer_color(layer))
    }

    pub fn get_layer_description(&self, layer: &str) -> String {
        self.layer_info
            .get(layer)
            .map(|info| info.description.clone())
            .unwrap_or_else(|| layer_description(layer))
    }
}

impl Default for LayerDetector {
    fn default() -> Self {
        Self::new()
    }
}
