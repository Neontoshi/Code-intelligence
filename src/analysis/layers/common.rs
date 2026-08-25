// src/analysis/layers/common.rs

//! Common utilities for layer detection

use crate::parser::tree_sitter::ParsedFile;

/// Generic layer detection (fallback for unknown languages)
pub fn detect_layer_generic(file: &ParsedFile) -> String {
    let path = &file.path;
    let parts: Vec<&str> = path.split('/').collect();

    if parts.len() >= 2 {
        match parts[parts.len() - 2] {
            "handlers" | "controllers" | "routes" => "handler".to_string(),
            "services" | "domain" | "business" => "service".to_string(),
            "db" | "repository" | "repositories" | "models" | "dao" => "repository".to_string(),
            "middleware" => "middleware".to_string(),
            "config" | "configuration" => "config".to_string(),
            "workers" | "jobs" | "tasks" => "worker".to_string(),
            "solana" | "blockchain" | "chain" => "blockchain".to_string(),
            "telemetry" | "metrics" | "tracing" | "observability" => "observability".to_string(),
            "auth" | "authentication" | "authorization" => "auth".to_string(),
            "utils" | "util" | "helpers" | "common" => "utility".to_string(),
            "api" | "rest" | "graphql" => "api".to_string(),
            "cli" | "cmd" | "bin" => "cli".to_string(),
            "tests" | "test" | "integration" => "test".to_string(),
            _ => "core".to_string(),
        }
    } else {
        "root".to_string()
    }
}

/// Get layer color
pub fn layer_color(layer: &str) -> String {
    match layer {
        "handler" => "#38bdf8".to_string(),
        "service" => "#10b981".to_string(),
        "repository" => "#f59e0b".to_string(),
        "middleware" => "#a855f7".to_string(),
        "config" => "#06b6d4".to_string(),
        "worker" => "#f97316".to_string(),
        "blockchain" => "#f43f5e".to_string(),
        "observability" => "#ec4899".to_string(),
        "auth" => "#e11d48".to_string(),
        "utility" => "#64748b".to_string(),
        "api" => "#6366f1".to_string(),
        "cli" => "#475569".to_string(),
        "test" => "#d946ef".to_string(),
        "core" => "#475569".to_string(),
        "root" => "#eab308".to_string(),
        other => hash_color(other),
    }
}

/// Get layer description
pub fn layer_description(layer: &str) -> String {
    match layer {
        "handler" => "Receives requests and kicks off the work".to_string(),
        "service" => "Business logic — the actual rules of the app".to_string(),
        "repository" => "Reads and writes data storage".to_string(),
        "middleware" => "Runs on every request before it reaches handlers".to_string(),
        "config" => "Settings and environment setup".to_string(),
        "worker" => "Background jobs and scheduled tasks".to_string(),
        "blockchain" => "On-chain / smart contract interactions".to_string(),
        "observability" => "Logging, metrics, tracing".to_string(),
        "auth" => "Login, permissions, access control".to_string(),
        "utility" => "Shared helper functions".to_string(),
        "api" => "External-facing interface".to_string(),
        "cli" => "Command-line entry points".to_string(),
        "test" => "Test code".to_string(),
        "core" => "Core application code".to_string(),
        "root" => "Top-level project files".to_string(),
        other => format!("Code under the `{}` module", other),
    }
}

/// Hash-based color generation for unknown layers
fn hash_color(name: &str) -> String {
    let mut hash: u32 = 5381;
    for b in name.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(b as u32);
    }
    let hue = hash % 360;
    format!("hsl({}, 70%, 54%)", hue)
}
