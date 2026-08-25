// src/parser/mod.rs

pub mod comments;
pub mod languages;
pub mod semantic;
pub mod tree_sitter;

// Re-export everything from tree_sitter
pub use tree_sitter::*;
