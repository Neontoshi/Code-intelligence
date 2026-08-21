// src/error.rs

//! Error taxonomy for the code-intelligence engine
//!
//! This module defines all error types used throughout the codebase.

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for the code-intelligence engine
#[derive(Error, Debug)]
pub enum CodeIntelError {
    // ============================================================
    // Parse Errors
    // ============================================================
    #[error("Failed to parse file: {path} - {source}")]
    ParseError {
        path: PathBuf,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Unsupported language: {lang}")]
    UnsupportedLanguage { lang: String },

    #[error("Failed to parse tree-sitter: {source}")]
    TreeSitterError {
        #[from]
        source: tree_sitter::LanguageError,
    },

    // ============================================================
    // Graph Errors
    // ============================================================
    #[error("Call graph error: {message}")]
    GraphError { message: String },

    #[error("Cycle detected in graph: {nodes:?}")]
    CycleDetected { nodes: Vec<String> },

    #[error("Node not found: {node}")]
    NodeNotFound { node: String },

    // ============================================================
    // Model Errors
    // ============================================================
    #[error("Model error: {message}")]
    ModelError { message: String },

    #[error("Model file not found: {path}")]
    ModelNotFound { path: PathBuf },

    #[error("Model schema mismatch: expected {expected}, got {got}")]
    ModelSchemaMismatch { expected: usize, got: usize },

    #[error("Model version mismatch: {message}")]
    ModelVersionMismatch { message: String },

    // ============================================================
    // Dataset Errors
    // ============================================================
    #[error("Dataset error: {message}")]
    DatasetError { message: String },

    #[error("Missing repository ID for example: {function}")]
    MissingRepositoryId { function: String },

    #[error("Dataset split error: {message}")]
    SplitError { message: String },

    // ============================================================
    // Cache Errors
    // ============================================================
    #[error("Cache error: {message}")]
    CacheError { message: String },

    #[error("Cache miss: {key}")]
    CacheMiss { key: String },

    #[error("Cache corruption: {path}")]
    CacheCorruption { path: PathBuf },

    // ============================================================
    // Config Errors
    // ============================================================
    #[error("Config error: {message}")]
    ConfigError { message: String },

    #[error("Config file not found: {path}")]
    ConfigNotFound { path: PathBuf },

    #[error("Invalid config value: {key} = {value}")]
    InvalidConfigValue { key: String, value: String },

    // ============================================================
    // Git Errors
    // ============================================================
    #[error("Git error: {message}")]
    GitError { message: String },

    #[error("Not a git repository: {path}")]
    NotGitRepo { path: PathBuf },

    // ============================================================
    // IO Errors
    // ============================================================
    #[error("IO error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: PathBuf },

    // ============================================================
    // Analysis Errors
    // ============================================================
    #[error("Analysis error: {message}")]
    AnalysisError { message: String },

    #[error("Analysis timeout: {duration}s exceeded")]
    AnalysisTimeout { duration: u64 },

    #[error("Analysis cancelled")]
    AnalysisCancelled,

    #[error("Memory limit exceeded: {limit}MB")]
    MemoryLimitExceeded { limit: usize },

    // ============================================================
    // Serialization Errors
    // ============================================================
    #[error("Serialization error: {source}")]
    SerializationError {
        #[from]
        source: serde_json::Error,
    },

    #[error("Deserialization error: {message}")]
    DeserializationError { message: String },

    // ============================================================
    // LLM Errors
    // ============================================================
    #[error("LLM error: {message}")]
    LlmError { message: String },

    #[error("LLM provider not available: {provider}")]
    LlmProviderUnavailable { provider: String },

    #[error("LLM rate limit exceeded")]
    LlmRateLimitExceeded,

    // ============================================================
    // Feature Extraction Errors
    // ============================================================
    #[error("Feature extraction error: {message}")]
    FeatureError { message: String },

    #[error("Feature not found: {name}")]
    FeatureNotFound { name: String },

    #[error("Feature vector length mismatch: expected {expected}, got {got}")]
    FeatureLengthMismatch { expected: usize, got: usize },

    // ============================================================
    // Training Errors
    // ============================================================
    #[error("Training error: {message}")]
    TrainingError { message: String },

    #[error("Insufficient training data: {message}")]
    InsufficientTrainingData { message: String },

    #[error("Training data imbalance: alive={alive}, dead={dead}")]
    DataImbalance { alive: usize, dead: usize },

    // ============================================================
    // Internal Errors
    // ============================================================
    #[error("Internal error: {message}")]
    InternalError { message: String },

    #[error("Unreachable code reached: {message}")]
    Unreachable { message: String },

    #[error("Not implemented: {feature}")]
    NotImplemented { feature: String },
}

/// Result type alias for the code-intelligence engine
pub type Result<T> = std::result::Result<T, CodeIntelError>;

/// Context for errors - adds source location
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
    pub function: &'static str,
}

impl ErrorContext {
    pub fn new(file: &'static str, line: u32, column: u32, function: &'static str) -> Self {
        Self {
            file,
            line,
            column,
            function,
        }
    }
}

/// Macro for creating errors with context
#[macro_export]
macro_rules! context_err {
    ($err:expr, $file:expr, $line:expr, $column:expr, $function:expr) => {
        $crate::error::ErrorWithContext {
            error: Box::new($err),
            context: $crate::error::ErrorContext::new($file, $line, $column, $function),
        }
    };
}

/// Error with context for better debugging
#[derive(Debug)]
pub struct ErrorWithContext {
    pub error: Box<CodeIntelError>,
    pub context: ErrorContext,
}

impl std::fmt::Display for ErrorWithContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (at {}:{}:{})",
            self.error, self.context.file, self.context.line, self.context.column
        )
    }
}

impl std::error::Error for ErrorWithContext {}

impl From<ErrorWithContext> for CodeIntelError {
    fn from(err: ErrorWithContext) -> Self {
        *err.error
    }
}
