use bumpalo::Bump;

/// Thread-local bump allocator for fast, arena-based allocation
pub struct Allocator {
    bump: Bump,
}

impl Allocator {
    pub fn new() -> Self {
        Self { bump: Bump::new() }
    }

    /// Allocate a single value
    pub fn allocate<T>(&mut self, value: T) -> &mut T
    where
        T: Copy,
    {
        self.bump.alloc(value)
    }

    /// Allocate a slice
    pub fn allocate_slice<T>(&mut self, values: &[T]) -> &mut [T]
    where
        T: Clone + Copy,
    {
        self.bump.alloc_slice_copy(values)
    }

    /// Allocate a string
    pub fn allocate_str(&mut self, s: &str) -> &str {
        let bytes = self.bump.alloc_slice_copy(s.as_bytes());
        std::str::from_utf8(bytes).unwrap()
    }

    pub fn reset(&mut self) {
        self.bump.reset();
    }

    pub fn allocated_bytes(&self) -> usize {
        self.bump.allocated_bytes()
    }
}

impl Default for Allocator {
    fn default() -> Self {
        Self::new()
    }
}

/// A simple memory pool for large allocations
pub struct MemoryPool {
    pools: Vec<Vec<u8>>,
    current_pool: Vec<u8>,
    current_pos: usize,
    pool_size: usize,
}

impl MemoryPool {
    pub fn new(pool_size: usize) -> Self {
        Self {
            pools: Vec::new(),
            current_pool: Vec::with_capacity(pool_size),
            current_pos: 0,
            pool_size,
        }
    }

    pub fn allocate<'a>(&'a mut self, size: usize) -> Option<&'a mut [u8]> {
        if self.current_pos + size > self.current_pool.capacity() {
            let new_pool = Vec::with_capacity(self.pool_size);
            let old_pool = std::mem::replace(&mut self.current_pool, new_pool);
            if !old_pool.is_empty() {
                self.pools.push(old_pool);
            }
            self.current_pos = 0;
        }

        let start = self.current_pos;
        self.current_pos += size;

        // SAFETY: This unsafe block is sound because:
        // 1. We ensure the current pool has enough capacity by checking and
        //    allocating a new pool above if needed.
        // 2. We extend the pool to at least `current_pos` bytes, ensuring the
        //    memory we're about to access is valid and initialized.
        // 3. The returned slice has lifetime `'a` which is tied to `&mut self`,
        //    so it cannot outlive the pool itself.
        // 4. We only ever hand out mutable references to disjoint regions of
        //    the pool because we advance `current_pos` and never reuse it.
        // 5. The pool is never reallocated while slices exist (we only grow
        //    it, never shrink or move).
        unsafe {
            self.current_pool
                .resize(self.current_pool.len().max(self.current_pos), 0);
            let ptr = self.current_pool.as_mut_ptr().add(start);
            Some(std::slice::from_raw_parts_mut(ptr, size))
        }
    }

    pub fn reset(&mut self) {
        self.pools.clear();
        self.current_pool.clear();
        self.current_pos = 0;
    }
}

impl Default for MemoryPool {
    fn default() -> Self {
        Self::new(1024 * 1024) // 1MB default
    }
}
