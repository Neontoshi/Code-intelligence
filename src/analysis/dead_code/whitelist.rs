// src/analysis/dead_code/whitelist.rs
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

pub static WHITELIST: LazyLock<Whitelist> = LazyLock::new(Whitelist::new);

pub struct Whitelist {
    functions: HashSet<String>,
    patterns: Vec<Regex>,
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

        // PATTERNS (minimal) — compiled once at construction
        patterns.push(Regex::new("^test_").expect("valid regex"));
        patterns.push(Regex::new("^bench_").expect("valid regex"));

        Self {
            functions,
            patterns,
        }
    }

    pub fn is_whitelisted(&self, name: &str) -> bool {
        // Exact matches
        if self.functions.contains(name) {
            return true;
        }

        // Pattern matches (precompiled)
        self.patterns.iter().any(|re| re.is_match(name))
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
