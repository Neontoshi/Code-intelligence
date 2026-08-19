use crate::graph::call_graph::FunctionNode;
use crate::optimize::dedup::core::{compute_ast_hash, compute_exact_hash, compute_signature_hash};
use crate::parser::tree_sitter::ParsedFile;
use dashmap::DashMap;
use std::sync::Arc;

/// Comprehensive indexes for fast lookups
#[derive(Debug, Clone)]
pub struct RichIndexes {
    /// Hash → list of function full paths
    pub signature_hash: Arc<DashMap<String, Vec<String>>>,
    pub ast_hash: Arc<DashMap<String, Vec<String>>>,
    pub body_hash: Arc<DashMap<String, Vec<String>>>,

    /// Name → list of function full paths
    pub function_name: Arc<DashMap<String, Vec<String>>>,

    /// File → list of function full paths
    pub file_to_functions: Arc<DashMap<String, Vec<String>>>,

    /// Type → list of definitions
    pub type_to_definitions: Arc<DashMap<String, Vec<String>>>,

    /// Import → list of files
    pub import_to_files: Arc<DashMap<String, Vec<String>>>,

    /// Symbol → list of definitions
    pub symbol_to_definitions: Arc<DashMap<String, Vec<String>>>,

    /// Caller → list of callees
    pub call_index: Arc<DashMap<String, Vec<String>>>,

    /// Callee → list of callers
    pub caller_index: Arc<DashMap<String, Vec<String>>>,

    /// Complexity range → list of functions
    pub complexity_index: Arc<DashMap<String, Vec<String>>>,

    /// Language → list of files
    pub language_index: Arc<DashMap<String, Vec<String>>>,

    /// Layer → list of functions
    pub layer_index: Arc<DashMap<String, Vec<String>>>,
}

impl RichIndexes {
    pub fn new() -> Self {
        Self {
            signature_hash: Arc::new(DashMap::new()),
            ast_hash: Arc::new(DashMap::new()),
            body_hash: Arc::new(DashMap::new()),
            function_name: Arc::new(DashMap::new()),
            file_to_functions: Arc::new(DashMap::new()),
            type_to_definitions: Arc::new(DashMap::new()),
            import_to_files: Arc::new(DashMap::new()),
            symbol_to_definitions: Arc::new(DashMap::new()),
            call_index: Arc::new(DashMap::new()),
            caller_index: Arc::new(DashMap::new()),
            complexity_index: Arc::new(DashMap::new()),
            language_index: Arc::new(DashMap::new()),
            layer_index: Arc::new(DashMap::new()),
        }
    }

    /// Get functions by signature hash
    pub fn get_by_signature_hash(&self, hash: &str) -> Vec<String> {
        self.signature_hash
            .get(hash)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// Get functions by AST hash
    pub fn get_by_ast_hash(&self, hash: &str) -> Vec<String> {
        self.ast_hash
            .get(hash)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// Get functions by body hash
    pub fn get_by_body_hash(&self, hash: &str) -> Vec<String> {
        self.body_hash
            .get(hash)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// Get functions by name
    pub fn get_by_name(&self, name: &str) -> Vec<String> {
        self.function_name
            .get(name)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// Get functions by file
    pub fn get_by_file(&self, file: &str) -> Vec<String> {
        self.file_to_functions
            .get(file)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// Get callers of a function
    pub fn get_callers(&self, full_path: &str) -> Vec<String> {
        self.caller_index
            .get(full_path)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// Get callees of a function
    pub fn get_callees(&self, full_path: &str) -> Vec<String> {
        self.call_index
            .get(full_path)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// Get functions by complexity range
    pub fn get_by_complexity(&self, range: &str) -> Vec<String> {
        self.complexity_index
            .get(range)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// Get functions by layer
    pub fn get_by_layer(&self, layer: &str) -> Vec<String> {
        self.layer_index
            .get(layer)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// Get files by language
    pub fn get_by_language(&self, language: &str) -> Vec<String> {
        self.language_index
            .get(language)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// Search functions by name (contains)
    pub fn search_by_name(&self, query: &str) -> Vec<String> {
        let mut results = Vec::new();
        for entry in self.function_name.iter() {
            if entry.key().contains(query) {
                results.extend(entry.value().clone());
            }
        }
        results
    }

    /// Search functions by file (contains)
    pub fn search_by_file(&self, query: &str) -> Vec<String> {
        let mut results = Vec::new();
        for entry in self.file_to_functions.iter() {
            if entry.key().contains(query) {
                results.extend(entry.value().clone());
            }
        }
        results
    }
}

impl Default for RichIndexes {
    fn default() -> Self {
        Self::new()
    }
}

pub struct IndexBuilder {
    indexes: RichIndexes,
}

impl IndexBuilder {
    pub fn new() -> Self {
        Self {
            indexes: RichIndexes::new(),
        }
    }

    /// Build all indexes from parsed files and call graph
    pub fn build(
        mut self,
        functions: &[FunctionNode],
        files: &[ParsedFile],
        sources: &crate::optimize::dedup::core::SourceIndex,
    ) -> RichIndexes {
        // Index functions
        for func in functions {
            self.index_function(func, sources);
        }

        // Index files
        for file in files {
            self.index_file(file);
        }

        // Build call indexes
        self.build_call_indexes(functions);

        self.indexes
    }

    fn index_function(
        &mut self,
        func: &FunctionNode,
        sources: &crate::optimize::dedup::core::SourceIndex,
    ) {
        let full_path = &func.full_path;

        // Name index
        self.indexes
            .function_name
            .entry(func.name.clone())
            .or_default()
            .push(full_path.clone());

        // File index
        self.indexes
            .file_to_functions
            .entry(func.file.clone())
            .or_default()
            .push(full_path.clone());

        // Hash indexes
        let sig_hash = compute_signature_hash(func);
        self.indexes
            .signature_hash
            .entry(sig_hash)
            .or_default()
            .push(full_path.clone());

        if let Some(source) = sources.get(full_path) {
            let ast_hash = compute_ast_hash(func, source);
            self.indexes
                .ast_hash
                .entry(ast_hash)
                .or_default()
                .push(full_path.clone());

            let body_hash = compute_exact_hash(func, Some(source));
            self.indexes
                .body_hash
                .entry(body_hash)
                .or_default()
                .push(full_path.clone());
        }

        // Complexity index
        let complexity_bucket = Self::complexity_bucket(func.complexity);
        self.indexes
            .complexity_index
            .entry(complexity_bucket)
            .or_default()
            .push(full_path.clone());

        // Layer index
        if !func.layer.is_empty() {
            self.indexes
                .layer_index
                .entry(func.layer.clone())
                .or_default()
                .push(full_path.clone());
        }
    }

    fn index_file(&mut self, file: &ParsedFile) {
        // Language index
        self.indexes
            .language_index
            .entry(file.language.clone())
            .or_default()
            .push(file.path.clone());

        // Type index
        for type_info in &file.types {
            self.indexes
                .type_to_definitions
                .entry(type_info.name.clone())
                .or_default()
                .push(format!("{}::{}", file.path, type_info.name));
        }

        // Import index
        for import in &file.imports {
            self.indexes
                .import_to_files
                .entry(import.module.clone())
                .or_default()
                .push(file.path.clone());
        }
    }

    fn build_call_indexes(&mut self, functions: &[FunctionNode]) {
        for _func in functions {
            // This is a placeholder - real implementation would use CallGraph
        }
    }

    fn complexity_bucket(complexity: f64) -> String {
        if complexity <= 5.0 {
            "simple".to_string()
        } else if complexity <= 10.0 {
            "moderate".to_string()
        } else if complexity <= 20.0 {
            "complex".to_string()
        } else {
            "very_complex".to_string()
        }
    }

    pub fn build_from_analysis(
        self,
        functions: &[FunctionNode],
        files: &[ParsedFile],
    ) -> RichIndexes {
        let sources = crate::optimize::dedup::core::SourceIndex::build(functions, files);
        self.build(functions, files, &sources)
    }
}

impl Default for IndexBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct IndexCache {
    cache: DashMap<String, RichIndexes>,
}

impl IndexCache {
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
        }
    }

    pub fn get_or_build<F>(&self, key: &str, build: F) -> RichIndexes
    where
        F: FnOnce() -> RichIndexes,
    {
        if let Some(indexes) = self.cache.get(key) {
            return indexes.clone();
        }

        let indexes = build();
        self.cache.insert(key.to_string(), indexes.clone());
        indexes
    }

    pub fn clear(&self) {
        self.cache.clear();
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }
}

impl Default for IndexCache {
    fn default() -> Self {
        Self::new()
    }
}
