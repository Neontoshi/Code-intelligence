// src/error.rs

//! Error handling for code-intelligence
//!
//! This module provides a unified error handling strategy using `anyhow`.
//! All library functions should return `Result<T, anyhow::Error>`.
//! CLI binaries convert to exit codes at the top level.

use std::path::PathBuf;

/// Main Result type for the code-intelligence library
pub type Result<T> = anyhow::Result<T>;

/// Common error helpers for creating errors
pub mod err {
    use super::*;

    /// Create a parse error
    pub fn parse(
        path: PathBuf,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> anyhow::Error {
        anyhow::anyhow!("Failed to parse file {}: {}", path.display(), source)
    }

    /// Create a graph error
    pub fn graph(message: impl Into<String>) -> anyhow::Error {
        anyhow::anyhow!("Graph error: {}", message.into())
    }

    /// Create a model error
    pub fn model(message: impl Into<String>) -> anyhow::Error {
        anyhow::anyhow!("Model error: {}", message.into())
    }

    /// Create a config error
    pub fn config(message: impl Into<String>) -> anyhow::Error {
        anyhow::anyhow!("Configuration error: {}", message.into())
    }

    /// Create an IO error with context
    pub fn io(
        path: PathBuf,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> anyhow::Error {
        anyhow::anyhow!("IO error on {}: {}", path.display(), source)
    }

    /// Create an analysis error
    pub fn analysis(message: impl Into<String>) -> anyhow::Error {
        anyhow::anyhow!("Analysis error: {}", message.into())
    }

    /// Create an LLM error
    pub fn llm(message: impl Into<String>) -> anyhow::Error {
        anyhow::anyhow!("LLM error: {}", message.into())
    }

    /// Create a training error
    pub fn training(message: impl Into<String>) -> anyhow::Error {
        anyhow::anyhow!("Training error: {}", message.into())
    }

    /// Create a feature extraction error
    pub fn feature(message: impl Into<String>) -> anyhow::Error {
        anyhow::anyhow!("Feature error: {}", message.into())
    }

    /// Create a dataset error
    pub fn dataset(message: impl Into<String>) -> anyhow::Error {
        anyhow::anyhow!("Dataset error: {}", message.into())
    }

    /// Create a cache error
    pub fn cache(message: impl Into<String>) -> anyhow::Error {
        anyhow::anyhow!("Cache error: {}", message.into())
    }

    /// Create a git error
    pub fn git(message: impl Into<String>) -> anyhow::Error {
        anyhow::anyhow!("Git error: {}", message.into())
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> anyhow::Error {
        anyhow::anyhow!("Internal error: {}", message.into())
    }
}

// Re-export anyhow macros for convenience
pub use anyhow::{anyhow, bail, Context, Context as _};
