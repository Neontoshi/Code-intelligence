//! Common traits for graph implementations

/// Trait for graphs that have node and edge counts
pub trait GraphMetrics {
    fn node_count(&self) -> usize;
    fn edge_count(&self) -> usize;
}

/// Trait for graphs that support indexing by NodeIndex
pub trait GraphIndex<T> {
    fn get_node(&self, index: T) -> Option<&T>;
    fn get_node_mut(&mut self, index: T) -> Option<&mut T>;
}
