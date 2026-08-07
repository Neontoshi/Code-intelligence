// src/graph/traits.rs

/// Common metrics for all graph types
pub trait GraphMetrics {
    fn node_count(&self) -> usize;
    fn edge_count(&self) -> usize;
}

/// Common indexing for all graph types
pub trait GraphIndex<T> {
    fn get_node(&self, index: T) -> Option<&T>;
    fn get_node_mut(&mut self, index: T) -> Option<&mut T>;
}

/// Implementation of Index trait for graph types
pub trait Indexable {
    type Output;
    type Index;

    fn index(&self, index: Self::Index) -> &Self::Output;
}

// ============================================================
// ⭐ MACROS for implementing GraphMetrics and Index
// ============================================================

#[macro_export]
macro_rules! impl_graph_metrics {
    ($type:ty) => {
        impl crate::graph::traits::GraphMetrics for $type {
            fn node_count(&self) -> usize {
                self.graph.node_count()
            }

            fn edge_count(&self) -> usize {
                self.graph.edge_count()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_graph_index {
    ($type:ty, $node_type:ty) => {
        impl std::ops::Index<petgraph::graph::NodeIndex> for $type {
            type Output = $node_type;

            fn index(&self, index: petgraph::graph::NodeIndex) -> &Self::Output {
                &self.graph[index]
            }
        }

        impl std::ops::IndexMut<petgraph::graph::NodeIndex> for $type {
            fn index_mut(&mut self, index: petgraph::graph::NodeIndex) -> &mut Self::Output {
                &mut self.graph[index]
            }
        }
    };
}
