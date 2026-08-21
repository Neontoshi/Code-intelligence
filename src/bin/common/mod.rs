// src/bin/common/mod.rs

pub mod cleanup;
pub mod error_handler;
pub mod exit_codes;
pub mod graceful;
pub mod metrics;
pub mod monitor;
pub mod reporter;

// Re-exports
pub use cleanup::ResourceManager;
pub use error_handler::ErrorHandler;
pub use graceful::{
    with_retries_async, with_retries_blocking, DegradationConfig, DegradationResult,
};
pub use metrics::EvaluationMetrics;
pub use monitor::{Counter, MetricsCollector, Timer};
pub use reporter::Reporter;
