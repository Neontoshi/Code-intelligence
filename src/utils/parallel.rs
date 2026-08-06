use rayon::prelude::*;
use std::sync::Arc;

/// Parallel processing utilities
pub struct ParallelUtils;

impl ParallelUtils {
    /// Process items in parallel with a closure
    pub fn parallel_map<T, R, F>(items: Vec<T>, f: F) -> Vec<R>
    where
        T: Send + Sync,
        R: Send + Sync,
        F: Fn(T) -> R + Send + Sync,
    {
        items.into_par_iter().map(f).collect()
    }

    /// Process items in parallel with a fallback function
    pub fn parallel_map_with_fallback<T, R, F, G>(items: Vec<T>, f: F, fallback: G) -> Vec<R>
    where
        T: Send + Sync + Clone + std::panic::RefUnwindSafe,
        R: Send + Sync,
        F: Fn(T) -> R + Send + Sync + std::panic::RefUnwindSafe,
        G: Fn(T) -> R + Send + Sync,
    {
        items
            .into_par_iter()
            .map(|item| {
                std::panic::catch_unwind(|| f(item.clone())).unwrap_or_else(|_| fallback(item))
            })
            .collect()
    }

    /// Process items in parallel with progress tracking
    pub fn parallel_map_with_progress<T, R, F>(
        items: Vec<T>,
        f: F,
        on_progress: impl Fn(usize, usize) + Send + Sync,
    ) -> Vec<R>
    where
        T: Send + Sync,
        R: Send + Sync,
        F: Fn(T) -> R + Send + Sync,
    {
        let total = items.len();
        let results: Vec<R> = items
            .into_par_iter()
            .enumerate()
            .map(|(i, item)| {
                let result = f(item);
                on_progress(i + 1, total);
                result
            })
            .collect();

        results
    }

    /// Parallel filter and map
    pub fn parallel_filter_map<T, R, F, G>(items: Vec<T>, filter: F, map: G) -> Vec<R>
    where
        T: Send + Sync,
        R: Send + Sync,
        F: Fn(&T) -> bool + Send + Sync,
        G: Fn(T) -> R + Send + Sync,
    {
        items
            .into_par_iter()
            .filter(|item| filter(item))
            .map(map)
            .collect()
    }

    /// Parallel reduce - simplified version
    pub fn parallel_reduce<T, F, G>(items: Vec<T>, identity: T, reduce: F, combine: G) -> T
    where
        T: Send + Sync + Clone,
        F: Fn(T, T) -> T + Send + Sync,
        G: Fn(T, T) -> T + Send + Sync,
    {
        items
            .into_par_iter()
            .fold(|| identity.clone(), reduce)
            .reduce(|| identity.clone(), combine)
    }

    /// Process files in parallel
    pub fn process_files<T, F>(paths: Vec<std::path::PathBuf>, process: F) -> Vec<Result<T, String>>
    where
        T: Send + Sync,
        F: Fn(std::path::PathBuf) -> Result<T, String> + Send + Sync,
    {
        paths.into_par_iter().map(process).collect()
    }

    /// Parallel map with a thread pool
    pub fn with_thread_pool<F, R>(num_threads: usize, f: F) -> R
    where
        F: FnOnce(&rayon::ThreadPool) -> R + Send + Sync,
        R: Send + Sync,
    {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .unwrap();

        pool.install(|| f(&pool))
    }

    /// Parallel execution with context
    pub fn parallel_with_context<T, C, R, F>(items: Vec<T>, context: Arc<C>, f: F) -> Vec<R>
    where
        T: Send + Sync,
        C: Send + Sync,
        R: Send + Sync,
        F: Fn(T, Arc<C>) -> R + Send + Sync,
    {
        items
            .into_par_iter()
            .map(|item| f(item, context.clone()))
            .collect()
    }

    /// Chunked parallel processing for memory efficiency
    pub fn chunked_parallel<T, R, F>(items: Vec<T>, chunk_size: usize, f: F) -> Vec<R>
    where
        T: Send + Sync + Clone,
        R: Send + Sync,
        F: Fn(Vec<T>) -> R + Send + Sync,
    {
        let chunks: Vec<Vec<T>> = items
            .chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        chunks.into_par_iter().map(f).collect()
    }
}

/// A parallel work queue
pub struct ParallelQueue<T> {
    items: Vec<T>,
    processed: usize,
}

impl<T> ParallelQueue<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self {
            items,
            processed: 0,
        }
    }

    pub fn process<F, R>(&mut self, f: F) -> Vec<R>
    where
        T: Send + Sync,
        R: Send + Sync,
        F: Fn(T) -> R + Send + Sync,
    {
        let mut results = Vec::with_capacity(self.items.len());
        let _total = self.items.len();

        for item in self.items.drain(..) {
            self.processed += 1;
            results.push(f(item));
        }

        results
    }

    pub fn progress(&self) -> (usize, usize) {
        (self.processed, self.items.len() + self.processed)
    }

    pub fn remaining(&self) -> usize {
        self.items.len()
    }
}
