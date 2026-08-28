pub mod call_graph;
pub mod dependency_graph;
pub mod graph_traits;
pub mod import_graph;
pub mod project_graph;
pub mod resolution;
pub mod resolver;
pub mod traits;
pub mod type_graph;
pub mod unresolved_handler;

pub use crate::impl_graph_index;
pub use crate::impl_graph_metrics;
pub use resolution::{ResolutionConfidence, ResolutionMethod, ResolutionStats, ResolvedCall};
pub use resolver::CallResolver;
pub use traits::{GraphIndex, GraphMetrics, Indexable};

#[macro_export]
macro_rules! define_graph {
    ($name:ident, $node_type:ty, $edge_type:ty) => {
        #[derive(Debug)]
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
