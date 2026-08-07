// src/analysis/dead_code/whitelist.rs
use std::collections::HashSet;
use std::sync::LazyLock;

pub static WHITELIST: LazyLock<Whitelist> = LazyLock::new(Whitelist::new);

pub struct Whitelist {
    functions: HashSet<String>,
    patterns: Vec<String>,
}

impl Whitelist {
    pub fn new() -> Self {
        let mut functions = HashSet::new();
        let mut patterns = Vec::new();

        // ESSENTIAL ENTRY POINTS
        functions.insert("main".to_string());
        functions.insert("async_main".to_string());
        functions.insert("run".to_string());
        functions.insert("start".to_string());
        functions.insert("init".to_string());

        // COMMON TRAIT METHODS
        functions.insert("default".to_string());
        functions.insert("clone".to_string());
        functions.insert("drop".to_string());

        // PATTERNS (minimal)
        patterns.push("^test_".to_string());
        patterns.push("^bench_".to_string());

        Self {
            functions,
            patterns,
        }
    }

    /// Check if a function is whitelisted by exact name
    pub fn is_whitelisted(&self, name: &str) -> bool {
        // Exact matches
        if self.functions.contains(name) {
            return true;
        }

        // Pattern matches
        for pattern in &self.patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(name) {
                    return true;
                }
            }
        }

        false
    }

    /// Check if a function is whitelisted by file path
    /// (Mostly handled by static analysis now)
    pub fn is_whitelisted_path(&self, full_path: &str) -> bool {
        if full_path.contains("build.rs") {
            return true;
        }

        false
    }

    /// Add a function to the whitelist dynamically
    pub fn add_function(&mut self, name: &str) {
        self.functions.insert(name.to_string());
    }

    /// Remove a function from the whitelist
    pub fn remove_function(&mut self, name: &str) {
        self.functions.remove(name);
    }
}

impl Default for Whitelist {
    fn default() -> Self {
        Self::new()
    }
}
