// src/graph/mod.rs

pub mod call_graph;
pub mod dependency_graph;
pub mod graph_traits;
pub mod import_graph;
pub mod project_graph;
pub mod traits; // ⭐ NEW
pub mod type_graph;

// Re-export traits
pub use traits::{GraphIndex, GraphMetrics, Indexable};
