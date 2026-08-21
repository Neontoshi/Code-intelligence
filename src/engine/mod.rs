// src/engine/mod.rs

pub mod cache;
pub mod call_graph_builder;
pub mod config;
pub mod file_collector;
pub mod incremental;
pub mod indexer;
pub mod llm_analysis;
pub mod pipeline;
pub mod stages;

// Re-exports
pub use call_graph_builder::CallGraphBuilder;
pub use config::PipelineConfig;
pub use file_collector::FileCollector;
pub use llm_analysis::{LLMAnalysis, LLMAnalyzer};
pub use pipeline::Pipeline;
pub use stages::{AnalyzedProject, OptimizedProject, ParsedProject, RawProject};

pub use pipeline::Pipeline as ProjectIntelligence;
