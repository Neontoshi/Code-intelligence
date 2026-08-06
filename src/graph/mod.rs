// src/graph/mod.rs

pub mod call_graph;
pub mod dependency_graph;
pub mod graph_traits;
pub mod import_graph;
pub mod project_graph;
pub mod traits;
pub mod type_graph;

// Re-export traits
pub use traits::{GraphIndex, GraphMetrics, Indexable};

#[macro_export]
macro_rules! define_graph {
    ($name:ident, $node_type:ty, $edge_type:ty) => {
        pub struct $name {
            pub graph: DiGraph<$node_type, $edge_type>,
            pub node_index: HashMap<String, NodeIndex>,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    graph: DiGraph::new(),
                    node_index: HashMap::new(),
                }
            }
        }
    };
}
