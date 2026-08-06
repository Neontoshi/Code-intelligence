// src/graph/traits.rs

//! Shared traits for graph implementations

use petgraph::graph::NodeIndex;

/// Common metrics for all graph types
pub trait GraphMetrics {
    /// Get the number of nodes in the graph
    fn node_count(&self) -> usize;

    /// Get the number of edges in the graph
    fn edge_count(&self) -> usize;
}

/// Common indexing for all graph types
pub trait GraphIndex<T> {
    /// Get a reference to a node by its index
    fn get_node(&self, index: T) -> Option<&T>;

    /// Get a mutable reference to a node by its index
    fn get_node_mut(&mut self, index: T) -> Option<&mut T>;
}

/// Implementation of Index trait for graph types
pub trait Indexable {
    type Output;
    type Index;

    fn index(&self, index: Self::Index) -> &Self::Output;
}
