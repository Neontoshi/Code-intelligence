use crate::analysis::features::FeatureExtractor;
use crate::engine::cache::FileCache;
use crate::engine::indexer::RichIndexes;
use crate::engine::llm_analysis::LLMAnalysis;
use crate::graph::call_graph::CallGraph;
use crate::graph::call_graph::FunctionNode;
use crate::graph::dependency_graph::DependencyGraph;
use crate::graph::import_graph::ImportGraph;
use crate::graph::project_graph::ProjectGraph;
use crate::graph::traits::GraphMetrics;
use crate::graph::type_graph::TypeGraph;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ProjectAnalysis {
    pub root: PathBuf,
    pub files: Arc<Vec<ParsedFile>>,
    pub project_graph: Arc<ProjectGraph>,
    pub call_graph: Arc<CallGraph>,
    pub type_graph: Arc<TypeGraph>,
    pub import_graph: Arc<ImportGraph>,
    pub dependency_graph: Arc<DependencyGraph>,
    pub indexes: Arc<AnalysisIndexes>,
    pub rich_indexes: Arc<RichIndexes>,
    pub metrics: Arc<ProjectMetrics>,
    pub features: Arc<FeatureExtractor>,
    pub cache: Arc<FileCache>,
    pub llm_analysis: Option<LLMAnalysis>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub version: u32,
}

#[derive(Debug, Clone, Default)]
pub struct AnalysisIndexes {
    pub name_to_functions: HashMap<String, Vec<String>>,
    pub file_to_functions: HashMap<String, Vec<String>>,
    pub signature_hash_to_functions: HashMap<String, Vec<String>>,
    pub ast_hash_to_functions: HashMap<String, Vec<String>>,
    pub body_hash_to_functions: HashMap<String, Vec<String>>,
    pub type_to_definitions: HashMap<String, Vec<String>>,
    pub import_to_files: HashMap<String, Vec<String>>,
    pub symbol_to_definitions: HashMap<String, Vec<String>>,
}

/// Project-level metrics
#[derive(Debug, Clone, Default)]
pub struct ProjectMetrics {
    pub total_functions: usize,
    pub total_types: usize,
    pub total_files: usize,
    pub total_call_edges: usize,
    pub total_import_edges: usize,
    pub total_dependency_edges: usize,
    pub average_complexity: f64,
    pub max_complexity: f64,
    pub total_lines: usize,
    pub languages: Vec<String>,
    pub layers: Vec<String>,
}

// Builder
pub struct ProjectAnalysisBuilder {
    root: PathBuf,
    files: Vec<ParsedFile>,
    project_graph: Option<ProjectGraph>,
    call_graph: Option<CallGraph>,
    type_graph: Option<TypeGraph>,
    import_graph: Option<ImportGraph>,
    dependency_graph: Option<DependencyGraph>,
    cache: FileCache,
    llm_analysis: Option<LLMAnalysis>,
    features: Option<FeatureExtractor>,
    rich_indexes: Option<RichIndexes>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnalysisMetadata {
    pub analysis_id: String,
    pub model_version: String,
    pub feature_schema_version: u32,
    pub source_commit: String,
    pub analysis_timestamp: i64,
    pub total_functions: usize,
    pub dead_candidates: usize,
}

impl ProjectAnalysisBuilder {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            files: Vec::new(),
            project_graph: None,
            call_graph: None,
            type_graph: None,
            import_graph: None,
            dependency_graph: None,
            cache: FileCache::new(),
            llm_analysis: None,
            features: None,
            rich_indexes: None,
        }
    }

    pub fn with_files(mut self, files: Vec<ParsedFile>) -> Self {
        self.files = files;
        self
    }

    pub fn with_project_graph(mut self, project_graph: ProjectGraph) -> Self {
        self.project_graph = Some(project_graph);
        self
    }

    pub fn with_call_graph(mut self, call_graph: CallGraph) -> Self {
        self.call_graph = Some(call_graph);
        self
    }

    pub fn with_type_graph(mut self, type_graph: TypeGraph) -> Self {
        self.type_graph = Some(type_graph);
        self
    }

    pub fn with_import_graph(mut self, import_graph: ImportGraph) -> Self {
        self.import_graph = Some(import_graph);
        self
    }

    pub fn with_dependency_graph(mut self, dependency_graph: DependencyGraph) -> Self {
        self.dependency_graph = Some(dependency_graph);
        self
    }

    pub fn with_cache(mut self, cache: FileCache) -> Self {
        self.cache = cache;
        self
    }

    pub fn with_llm_analysis(mut self, llm_analysis: LLMAnalysis) -> Self {
        self.llm_analysis = Some(llm_analysis);
        self
    }

    pub fn with_llm_analysis_opt(mut self, llm_analysis: Option<LLMAnalysis>) -> Self {
        self.llm_analysis = llm_analysis;
        self
    }

    pub fn with_features(mut self, features: FeatureExtractor) -> Self {
        self.features = Some(features);
        self
    }
    pub fn with_rich_indexes(mut self, rich_indexes: RichIndexes) -> Self {
        self.rich_indexes = Some(rich_indexes);
        self
    }

    pub fn build(self) -> ProjectAnalysis {
        let project_graph = Arc::new(self.project_graph.unwrap_or_else(ProjectGraph::new));
        let call_graph = Arc::new(self.call_graph.unwrap_or_else(CallGraph::new));
        let type_graph = Arc::new(self.type_graph.unwrap_or_else(TypeGraph::new));
        let import_graph = Arc::new(self.import_graph.unwrap_or_else(ImportGraph::new));
        let dependency_graph = Arc::new(self.dependency_graph.unwrap_or_else(DependencyGraph::new));

        // Build rich indexes BEFORE moving files into Arc
        let rich_indexes = Arc::new(self.rich_indexes.unwrap_or_else(|| {
            let builder = crate::engine::indexer::IndexBuilder::new();
            let functions: Vec<FunctionNode> = call_graph
                .node_indices()
                .map(|idx| call_graph[idx].clone())
                .collect();
            builder.build_from_analysis(&functions, &self.files)
        }));

        // Build features BEFORE moving files into Arc
        let features = Arc::new(self.features.unwrap_or_else(|| {
            let mut extractor = FeatureExtractor::new();
            let functions: Vec<FunctionNode> = call_graph
                .node_indices()
                .map(|idx| call_graph[idx].clone())
                .collect();
            extractor.extract_all(&functions, &self.files);
            extractor
        }));

        let files = Arc::new(self.files);
        let cache = Arc::new(self.cache);

        // Build indexes
        let indexes = Arc::new(Self::build_indexes(&files, &call_graph));

        // Build metrics
        let metrics = Arc::new(Self::build_metrics(&files, &call_graph));

        ProjectAnalysis {
            root: self.root,
            files,
            project_graph,
            call_graph,
            type_graph,
            import_graph,
            dependency_graph,
            indexes,
            rich_indexes,
            metrics,
            features,
            cache,
            llm_analysis: self.llm_analysis,
            created_at: chrono::Utc::now(),
            version: 1,
        }
    }

    fn build_indexes(files: &[ParsedFile], call_graph: &CallGraph) -> AnalysisIndexes {
        let mut indexes = AnalysisIndexes::default();

        // Build name → functions index
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            indexes
                .name_to_functions
                .entry(func.name.clone())
                .or_default()
                .push(func.full_path.clone());

            indexes
                .file_to_functions
                .entry(func.file.clone())
                .or_default()
                .push(func.full_path.clone());
        }

        // Build type → definitions index
        for file in files {
            for type_info in &file.types {
                indexes
                    .type_to_definitions
                    .entry(type_info.name.clone())
                    .or_default()
                    .push(format!("{}::{}", file.path, type_info.name));
            }
        }

        // Build import → files index
        for file in files {
            for import in &file.imports {
                indexes
                    .import_to_files
                    .entry(import.module.clone())
                    .or_default()
                    .push(file.path.clone());
            }
        }

        indexes
    }

    pub fn build_metrics(files: &[ParsedFile], call_graph: &CallGraph) -> ProjectMetrics {
        let mut metrics = ProjectMetrics::default();

        metrics.total_functions = call_graph.node_count();
        metrics.total_files = files.len();
        metrics.total_call_edges = call_graph.edge_count();

        // Languages
        let mut langs: Vec<String> = files.iter().map(|f| f.language.clone()).collect();
        langs.sort();
        langs.dedup();
        metrics.languages = langs;

        // Total lines
        metrics.total_lines = files.iter().map(|f| f.source.lines().count()).sum();

        metrics
    }
}

// Query Helpers

impl ProjectAnalysis {
    /// Get a function by full path
    pub fn get_function(&self, full_path: &str) -> Option<&crate::graph::call_graph::FunctionNode> {
        self.call_graph
            .name_index
            .get(full_path)
            .map(|&idx| &self.call_graph[idx])
    }

    /// Get functions by name
    pub fn get_functions_by_name(
        &self,
        name: &str,
    ) -> Vec<&crate::graph::call_graph::FunctionNode> {
        let mut result = Vec::new();
        if let Some(paths) = self.indexes.name_to_functions.get(name) {
            for path in paths {
                if let Some(func) = self.get_function(path) {
                    result.push(func);
                }
            }
        }
        result
    }

    /// Get functions by file
    pub fn get_functions_by_file(
        &self,
        file: &str,
    ) -> Vec<&crate::graph::call_graph::FunctionNode> {
        let mut result = Vec::new();
        if let Some(paths) = self.indexes.file_to_functions.get(file) {
            for path in paths {
                if let Some(func) = self.get_function(path) {
                    result.push(func);
                }
            }
        }
        result
    }

    /// Get all function names
    pub fn function_names(&self) -> Vec<String> {
        self.indexes.name_to_functions.keys().cloned().collect()
    }

    /// Get all files
    pub fn file_paths(&self) -> Vec<String> {
        self.files.iter().map(|f| f.path.clone()).collect()
    }

    /// Check if a function exists
    pub fn has_function(&self, full_path: &str) -> bool {
        self.get_function(full_path).is_some()
    }

    /// Get function count
    pub fn function_count(&self) -> usize {
        self.metrics.total_functions
    }

    /// Get file count
    pub fn file_count(&self) -> usize {
        self.metrics.total_files
    }

    /// Get call edge count
    pub fn call_edge_count(&self) -> usize {
        self.metrics.total_call_edges
    }
}

// Output Methods

impl ProjectAnalysis {
    pub fn to_markdown(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "# Code Intelligence: {}\n\n",
            self.root.file_name().unwrap_or_default().to_string_lossy()
        ));

        // Stats
        output.push_str("## 📊 Statistics\n\n");
        output.push_str(&format!(
            "- **Functions**: {}\n",
            self.metrics.total_functions
        ));
        output.push_str(&format!("- **Files**: {}\n", self.metrics.total_files));
        output.push_str(&format!(
            "- **Relationships**: {}\n\n",
            self.metrics.total_call_edges
        ));

        // LLM Analysis
        if let Some(ref llm) = self.llm_analysis {
            output.push_str("## 🤖 LLM Analysis\n\n");

            if let Some(doc) = &llm.documentation {
                output.push_str(&doc);
                output.push_str("\n\n");
            }

            if !llm.function_summaries.is_empty() {
                output.push_str("### 📝 Function Summaries\n\n");
                for (name, summary) in &llm.function_summaries {
                    output.push_str(&format!("- **{}**: {}\n", name, summary));
                }
                output.push('\n');
            }

            if !llm.issues.is_empty() {
                output.push_str("### 🐛 Issues Found\n\n");
                for (name, issue) in &llm.issues {
                    output.push_str(&format!(
                        "- **{}**: [{}] {} → {}\n",
                        name, issue.severity, issue.description, issue.suggestion
                    ));
                }
                output.push('\n');
            }
        }

        // Important functions
        output.push_str("## 🔥 Important Functions\n\n");
        let mut functions: Vec<_> = self
            .call_graph
            .node_indices()
            .map(|idx| (idx, self.call_graph[idx].importance_score))
            .collect();
        functions.sort_by(|a, b| b.1.total_cmp(&a.1));

        for (idx, score) in functions.iter().take(10) {
            let func = &self.call_graph[*idx];
            let emoji = if *score > 0.8 {
                "🔥"
            } else if *score > 0.5 {
                "📌"
            } else {
                "📄"
            };
            output.push_str(&format!(
                "- {} **{}** (importance: {:.2})\n",
                emoji, func.name, score
            ));
            output.push_str(&format!("  - File: {}\n", func.file));
            output.push_str(&format!("  - Line: {}\n", func.line));
            output.push_str(&format!("  - Calls: {}\n", func.fan_out));
            output.push_str(&format!("  - Called by: {}\n", func.fan_in));
        }

        output
    }

    pub fn to_json(&self) -> String {
        let mut report = serde_json::Map::new();

        report.insert(
            "project".to_string(),
            serde_json::Value::String(
                self.root
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            ),
        );
        report.insert(
            "total_functions".to_string(),
            serde_json::Value::Number(self.metrics.total_functions.into()),
        );
        report.insert(
            "total_files".to_string(),
            serde_json::Value::Number(self.metrics.total_files.into()),
        );
        report.insert(
            "total_edges".to_string(),
            serde_json::Value::Number(self.metrics.total_call_edges.into()),
        );

        if let Some(ref llm) = self.llm_analysis {
            report.insert(
                "llm_analysis".to_string(),
                serde_json::json!({
                    "has_documentation": llm.has_documentation,
                    "summarized_count": llm.summarized_count,
                    "issues_count": llm.issues_count,
                }),
            );
        }

        serde_json::to_string_pretty(&report).unwrap_or_default()
    }

    pub fn to_training_json(&self) -> String {
        crate::output::JsonOutput::generate(&self.call_graph, &self.files, &self.root)
    }

    pub fn to_graphviz(&self) -> String {
        self.call_graph.to_dot()
    }

    pub fn to_full_report(&self) -> String {
        use crate::optimize::{SemanticCompressor, TokenEstimator};

        let compressor = SemanticCompressor::new();
        let full = compressor.full_report(&self.call_graph, &self.files);

        let original_content: String = self
            .files
            .iter()
            .map(|f| f.source.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let (orig_tokens, comp_tokens, reduction) =
            TokenEstimator::compare(&original_content, &full);

        let mut output = String::new();
        output.push_str(&format!(
            "# Code Intelligence: {}\n\n",
            self.root.file_name().unwrap_or_default().to_string_lossy()
        ));
        output.push_str(&format!(
            "> 📊 Original: ~{} tokens | Compressed: ~{} tokens | **{:.1}% reduction**\n\n",
            orig_tokens, comp_tokens, reduction
        ));
        output.push_str("---\n\n");

        // LLM Analysis first
        if let Some(ref llm) = self.llm_analysis {
            output.push_str("## 🤖 LLM Analysis\n\n");
            if let Some(doc) = &llm.documentation {
                output.push_str(&doc);
                output.push_str("\n\n");
            }
        }

        output.push_str(&full);
        output
    }
}
