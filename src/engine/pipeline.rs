// src/engine/pipeline.rs

use crate::analysis::context::{ProjectAnalysis, ProjectAnalysisBuilder};
use crate::analysis::features::FeatureExtractor;
use crate::analysis::importance::ImportanceScorer;
use crate::engine::cache::{AnalysisCacheManager, FileCache};
use crate::engine::indexer::IndexBuilder;
use crate::graph::call_graph::{CallEdge, CallGraph, FunctionNode};
use crate::graph::project_graph::ProjectGraphBuilder;
use crate::llm::{create_ollama_phi2, CodeUnderstandingEngine, LLMProvider};
use crate::optimize::{SemanticCompressor, TokenEstimator};
use crate::parser::tree_sitter::{ParsedFile, TreeSitterParser};
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::graph::graph_traits::GraphMetrics;

// ============================================================================
// Pipeline Configuration
// ============================================================================

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub enable_llm: bool,
    pub enable_git: bool,
    pub llm_temperature: f32,
    pub llm_max_tokens: usize,
    pub max_files: usize,
    pub max_file_size: u64,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            enable_llm: false,
            enable_git: false,
            llm_temperature: 0.3,
            llm_max_tokens: 1000,
            max_files: 10000,
            max_file_size: 1_000_000, // 1MB
        }
    }
}

// ============================================================================
// Pipeline Stages
// ============================================================================

/// Stage 1: Raw file collection
pub struct RawProject {
    pub root: PathBuf,
    pub files: Vec<PathBuf>,
}

/// Stage 2: Parsed project
pub struct ParsedProject {
    pub root: PathBuf,
    pub files: Vec<ParsedFile>,
}

/// Stage 3: Analyzed project with graph
pub struct AnalyzedProject {
    pub root: PathBuf,
    pub files: Vec<ParsedFile>,
    pub call_graph: CallGraph,
    pub project_graph: crate::graph::project_graph::ProjectGraph,
}

/// Stage 4: Optimized project with features and indexes
pub struct OptimizedProject {
    pub root: PathBuf,
    pub files: Vec<ParsedFile>,
    pub call_graph: CallGraph,
    pub project_graph: crate::graph::project_graph::ProjectGraph,
    pub features: FeatureExtractor,
    pub rich_indexes: crate::engine::indexer::RichIndexes,
    pub metrics: crate::analysis::context::ProjectMetrics,
}

// ============================================================================
// Pipeline
// ============================================================================

pub struct Pipeline {
    _parser: TreeSitterParser,
    scorer: ImportanceScorer,
    _cache: FileCache,
    config: PipelineConfig,
    llm_provider: Option<Arc<dyn LLMProvider>>,
    code_understanding: Option<CodeUnderstandingEngine>,
    _analysis_cache: Option<AnalysisCacheManager>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            _parser: TreeSitterParser::new(),
            scorer: ImportanceScorer::new(),
            _cache: FileCache::new(),
            config: PipelineConfig::default(),
            llm_provider: None,
            code_understanding: None,
            _analysis_cache: None,
        }
    }

    pub fn with_config(mut self, config: PipelineConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_llm(mut self, provider: Arc<dyn LLMProvider>) -> Self {
        self.llm_provider = Some(provider.clone());
        self.code_understanding = Some(CodeUnderstandingEngine::new(provider));
        self.config.enable_llm = true;
        self
    }

    pub async fn with_ollama_phi2(mut self) -> Result<Self, String> {
        match create_ollama_phi2().await {
            Ok(provider) => {
                self.llm_provider = Some(provider.clone());
                self.code_understanding = Some(CodeUnderstandingEngine::new(provider));
                self.config.enable_llm = true;
                Ok(self)
            }
            Err(e) => {
                eprintln!("⚠️ Failed to initialize Ollama phi-2: {}", e);
                eprintln!("   Continuing without LLM support.");
                Ok(self)
            }
        }
    }

    pub fn enable_git(mut self) -> Self {
        self.config.enable_git = true;
        self
    }

    // ========================================================================
    // Immutable Pipeline Stages
    // ========================================================================

    /// Stage 1: Collect files
    pub fn stage_collect(&self, root: &Path) -> RawProject {
        let files = self.collect_files(root);
        RawProject {
            root: root.to_path_buf(),
            files,
        }
    }

    /// Stage 2: Parse files (returns new ParsedProject)
    pub fn stage_parse(&self, raw: RawProject) -> ParsedProject {
        let parsed_files: Vec<ParsedFile> = raw
            .files
            .par_iter()
            .filter_map(|file| {
                let thread_parser = TreeSitterParser::new();
                match thread_parser.parse_file(file) {
                    Ok(parsed) => {
                        if !parsed.functions.is_empty() || !parsed.types.is_empty() {
                            Some(parsed)
                        } else {
                            None
                        }
                    }
                    Err(_e) => None,
                }
            })
            .collect();

        ParsedProject {
            root: raw.root,
            files: parsed_files,
        }
    }

    /// Stage 2b: Parse files in parallel with progress tracking
    pub fn stage_parse_parallel(&self, raw: RawProject) -> ParsedProject {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let total = raw.files.len();
        let completed = Arc::new(AtomicUsize::new(0));

        let parsed_files: Vec<ParsedFile> = raw
            .files
            .par_iter()
            .filter_map(|file| {
                let thread_parser = TreeSitterParser::new();
                let result = thread_parser.parse_file(file);

                let count = completed.fetch_add(1, Ordering::Relaxed) + 1;
                if count % 10 == 0 || count == total {
                    eprintln!("   📄 Parsed {}/{} files", count, total);
                }

                match result {
                    Ok(parsed) => {
                        if !parsed.functions.is_empty() || !parsed.types.is_empty() {
                            Some(parsed)
                        } else {
                            None
                        }
                    }
                    Err(_e) => None,
                }
            })
            .collect();

        ParsedProject {
            root: raw.root,
            files: parsed_files,
        }
    }

    /// Stage 3: Build graphs (returns new AnalyzedProject)
    pub fn stage_analyze(&self, parsed: ParsedProject) -> AnalyzedProject {
        let mut call_graph = self.build_call_graph(&parsed.files);

        // Enhance call graph
        call_graph.calculate_fan_metrics();
        call_graph.detect_layers();
        call_graph.calculate_call_depth();

        // Build project graph
        let mut graph_builder = ProjectGraphBuilder::new();
        for file in &parsed.files {
            graph_builder = graph_builder.add_file(file.path.clone(), file.source.clone());
        }
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            graph_builder = graph_builder.add_function(func.clone(), &func.file);
        }
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            for callee in call_graph.get_callees(idx) {
                let edge = CallEdge {
                    call_type: "direct".to_string(),
                    line: func.line,
                };
                graph_builder = graph_builder.add_call(&func.full_path, &callee.full_path, edge);
            }
        }
        let project_graph = graph_builder.build();

        AnalyzedProject {
            root: parsed.root,
            files: parsed.files,
            call_graph,
            project_graph,
        }
    }
    /// Stage 3b: Build graphs with parallel data collection
    pub fn stage_analyze_parallel(&self, parsed: ParsedProject) -> AnalyzedProject {
        // Collect all function data in parallel - collect Vec of Vecs then flatten
        let all_func_data: Vec<(String, FunctionNode)> = parsed
            .files
            .par_iter()
            .map(|file| {
                let file_path = file.path.clone();
                let mut funcs = Vec::new();
                for func in &file.functions {
                    let full_path = match &func.container {
                        Some(c) => format!("{}::{}::{}", file_path, c, func.name),
                        None => format!("{}::{}", file_path, func.name),
                    };
                    let node = FunctionNode {
                        name: func.name.clone(),
                        full_path: full_path.clone(),
                        file: file_path.clone(),
                        line: func.line,
                        is_public: func.is_public,
                        is_async: func.is_async,
                        params: func.params.iter().map(|p| p.name.clone()).collect(),
                        returns: func.return_type.clone().into_iter().collect(),
                        complexity: 1.0,
                        importance_score: 0.0,
                        doc_comment: func.doc_comment.clone(),
                        writes_to: Vec::new(),
                        reads_from: Vec::new(),
                        errors: Vec::new(),
                        fan_in: 0,
                        fan_out: 0,
                        is_cycle: false,
                        depth: 0,
                        layer: String::new(),
                        trait_impl: func.trait_impl.clone(),
                    };
                    funcs.push((full_path, node));
                }
                funcs
            })
            .reduce(Vec::new, |mut acc, mut v| {
                acc.append(&mut v);
                acc
            });

        // Build call graph (single-threaded, but data collection was parallel)
        let mut call_graph = CallGraph::new();
        let mut path_to_idx = std::collections::HashMap::new();

        for (full_path, node) in all_func_data {
            let idx = call_graph.add_function(node);
            path_to_idx.insert(full_path, idx);
        }

        // Build edges
        for file in &parsed.files {
            let file_path = file.path.clone();
            for func in &file.functions {
                let caller_path = match &func.container {
                    Some(c) => format!("{}::{}::{}", file_path, c, func.name),
                    None => format!("{}::{}", file_path, func.name),
                };
                if let Some(&caller_idx) = path_to_idx.get(&caller_path) {
                    for called_name in &func.calls {
                        // Qualified call (e.g. "GraphVizOutput::new") — try
                        // matching container + method directly first.
                        let mut resolved = false;
                        if let Some((qualifier, method)) = called_name.rsplit_once("::") {
                            let qualified_path =
                                format!("{}::{}::{}", file_path, qualifier, method);
                            if let Some(&callee_idx) = path_to_idx.get(&qualified_path) {
                                call_graph.add_call(
                                    caller_idx,
                                    callee_idx,
                                    CallEdge {
                                        call_type: "direct".to_string(),
                                        line: func.line,
                                    },
                                );
                                resolved = true;
                            }
                        }
                        if resolved {
                            continue;
                        }

                        let simple_name = called_name.rsplit("::").next().unwrap_or(called_name);

                        // Same-file, unqualified: only accept if there's
                        // exactly one candidate in this file with that name —
                        // otherwise we can't tell which one was meant.
                        let same_file_candidates: Vec<_> = path_to_idx
                            .iter()
                            .filter(|(path, _)| {
                                path.starts_with(&format!("{}::", file_path))
                                    && path.ends_with(&format!("::{}", simple_name))
                                    && path != &&caller_path
                            })
                            .collect();
                        if same_file_candidates.len() == 1 {
                            let (_, &callee_idx) = same_file_candidates[0];
                            call_graph.add_call(
                                caller_idx,
                                callee_idx,
                                CallEdge {
                                    call_type: "fuzzy".to_string(),
                                    line: func.line,
                                },
                            );
                        }
                        // If ambiguous (0 or 2+ candidates), skip the edge —
                        // a missing edge is far less misleading than a
                        // guessed-wrong one.
                    }
                }
            }
        }

        // Enhance call graph
        call_graph.calculate_fan_metrics();
        call_graph.detect_layers();
        call_graph.calculate_call_depth();

        // Build project graph
        // Count call types
        // Count call types
        let mut call_types: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for idx in call_graph.node_indices() {
            for edge in call_graph
                .graph
                .edges_directed(idx, petgraph::Direction::Outgoing)
            {
                *call_types
                    .entry(edge.weight().call_type.clone())
                    .or_insert(0) += 1;
            }
        }
        println!("   📞 Call types: {:?}", call_types);

        // Build project graph
        let mut graph_builder = ProjectGraphBuilder::new();
        for file in &parsed.files {
            graph_builder = graph_builder.add_file(file.path.clone(), file.source.clone());
        }
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            graph_builder = graph_builder.add_function(func.clone(), &func.file);
        }
        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            for callee in call_graph.get_callees(idx) {
                let edge = CallEdge {
                    call_type: "direct".to_string(),
                    line: func.line,
                };
                graph_builder = graph_builder.add_call(&func.full_path, &callee.full_path, edge);
            }
        }
        let project_graph = graph_builder.build();

        AnalyzedProject {
            root: parsed.root,
            files: parsed.files,
            call_graph,
            project_graph,
        }
    }

    /// Stage 4: Extract features and build indexes (returns new OptimizedProject)
    pub fn stage_optimize(&self, analyzed: AnalyzedProject) -> OptimizedProject {
        let functions: Vec<FunctionNode> = analyzed
            .call_graph
            .node_indices()
            .map(|idx| analyzed.call_graph[idx].clone())
            .collect();

        // Extract features
        let mut feature_extractor = FeatureExtractor::new();
        feature_extractor.extract_all(&functions, &analyzed.files);

        // Build rich indexes
        let index_builder = IndexBuilder::new();
        let rich_indexes = index_builder.build_from_analysis(&functions, &analyzed.files);

        // Build metrics
        let metrics = ProjectAnalysisBuilder::build_metrics(&analyzed.files, &analyzed.call_graph);

        OptimizedProject {
            root: analyzed.root,
            files: analyzed.files,
            call_graph: analyzed.call_graph,
            project_graph: analyzed.project_graph,
            features: feature_extractor,
            rich_indexes,
            metrics,
        }
    }

    /// Stage 4b: Extract features in parallel
    pub fn stage_optimize_parallel(&self, analyzed: AnalyzedProject) -> OptimizedProject {
        let functions: Vec<FunctionNode> = analyzed
            .call_graph
            .node_indices()
            .map(|idx| analyzed.call_graph[idx].clone())
            .collect();

        // Build source map for parallel access
        let source_map: HashMap<String, String> = analyzed
            .files
            .iter()
            .flat_map(|f| {
                f.functions.iter().map(move |fi| {
                    let full_path = match &fi.container {
                        Some(c) => format!("{}::{}::{}", f.path, c, fi.name),
                        None => format!("{}::{}", f.path, fi.name),
                    };
                    let range = &fi.body_range;
                    let source = &f.source[range.0..range.1];
                    (full_path, source.to_string())
                })
            })
            .collect();

        // Extract features in parallel
        let features: Vec<(String, crate::analysis::features::FunctionFeatures)> = functions
            .par_iter()
            .filter_map(|func| {
                let source = source_map.get(&func.full_path).map(|s| s.as_str());
                let language = analyzed
                    .files
                    .iter()
                    .find(|f| f.path == func.file)
                    .map(|f| f.language.as_str())
                    .unwrap_or("unknown");
                Some((
                    func.full_path.clone(),
                    crate::analysis::features::FunctionFeatures::from_function(
                        func, source, language,
                    ),
                ))
            })
            .collect();

        // Build feature extractor
        let mut feature_extractor = FeatureExtractor::new();
        for (path, feature) in features {
            feature_extractor.insert(path, feature);
        }

        // Build rich indexes
        let index_builder = IndexBuilder::new();
        let rich_indexes = index_builder.build_from_analysis(&functions, &analyzed.files);

        // Build metrics
        let metrics = ProjectAnalysisBuilder::build_metrics(&analyzed.files, &analyzed.call_graph);

        OptimizedProject {
            root: analyzed.root,
            files: analyzed.files,
            call_graph: analyzed.call_graph,
            project_graph: analyzed.project_graph,
            features: feature_extractor,
            rich_indexes,
            metrics,
        }
    }

    /// Stage 5: Build final analysis
    pub fn stage_finalize(
        &self,
        optimized: OptimizedProject,
        llm: Option<LLMAnalysis>,
    ) -> ProjectAnalysis {
        ProjectAnalysisBuilder::new(optimized.root)
            .with_files(optimized.files)
            .with_call_graph(optimized.call_graph)
            .with_project_graph(optimized.project_graph)
            .with_features(optimized.features)
            .with_rich_indexes(optimized.rich_indexes)
            .with_llm_analysis_opt(llm)
            .build()
    }

    // ========================================================================
    // Main Processing Methods
    // ========================================================================

    pub async fn process_project(
        &mut self,
        root: &Path,
    ) -> Result<ProjectAnalysis, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();

        // Stage 1: Collect
        let raw = self.stage_collect(root);
        println!("📁 Found {} source files", raw.files.len());

        if raw.files.len() > self.config.max_files {
            println!(
                "⚠️ Too many files ({}), limiting to {}",
                raw.files.len(),
                self.config.max_files
            );
        }

        // Stage 2: Parse in parallel
        println!("🔄 Parsing files in parallel...");
        let parsed = self.stage_parse_parallel(raw);
        println!("✅ Successfully parsed {} files", parsed.files.len());

        if parsed.files.is_empty() {
            return Ok(ProjectAnalysisBuilder::new(root.to_path_buf())
                .with_call_graph(CallGraph::new())
                .build());
        }

        // Stage 3: Analyze in parallel
        println!("🔄 Building graphs in parallel...");
        let analyzed = self.stage_analyze(parsed);
        println!(
            "📊 Built call graph: {} functions, {} edges",
            analyzed.call_graph.node_count(),
            analyzed.call_graph.edge_count()
        );
        println!(
            "   📇 Indexes: {} names, {} files, {} public, {} async",
            analyzed.call_graph.name_to_functions.len(),
            analyzed.call_graph.file_to_functions.len(),
            analyzed.call_graph.public_functions.len(),
            analyzed.call_graph.async_functions.len()
        );

        let project_graph = &analyzed.project_graph;
        println!(
            "   📊 Project graph: {} nodes, {} edges",
            project_graph.node_count(),
            project_graph.edge_count()
        );

        // Cycle detection - only if graph isn't too large
        let node_count = analyzed.call_graph.node_count();
        if node_count < 1000 {
            let mut call_graph = analyzed.call_graph.clone();
            call_graph.mark_cycle_members();
            let cycle_count = call_graph
                .node_indices()
                .filter(|&idx| call_graph[idx].is_cycle)
                .count();
            println!("   🔄 {} functions in cycles", cycle_count);
        } else {
            println!(
                "   ⏭️ Skipping cycle detection ({} nodes > 1000)",
                node_count
            );
        }

        // Layers
        let layers: std::collections::HashSet<String> = analyzed
            .call_graph
            .node_indices()
            .map(|idx| analyzed.call_graph[idx].layer.clone())
            .collect();
        let mut layer_list: Vec<_> = layers.into_iter().collect();
        layer_list.sort();
        println!("   📂 Layers: {:?}", layer_list);

        // Stage 4: Optimize in parallel
        println!("🔄 Extracting features and building indexes in parallel...");
        let optimized = self.stage_optimize_parallel(analyzed);
        println!(
            "   ✅ Extracted features for {} functions",
            optimized.features.all().len()
        );
        println!(
            "   ✅ Built indexes: {} names, {} files, {} hashes",
            optimized.rich_indexes.function_name.len(),
            optimized.rich_indexes.file_to_functions.len(),
            optimized.rich_indexes.signature_hash.len()
        );

        // Score importance
        let mut call_graph = optimized.call_graph.clone();
        self.scorer.score_all(&mut call_graph);
        println!("📈 Scored function importance");

        // LLM Analysis
        let llm_analysis = if self.config.enable_llm && self.code_understanding.is_some() {
            println!("🤖 Running LLM analysis...");
            let analysis = self.run_llm_analysis(&call_graph, &optimized.files).await;
            if let Ok(ref a) = analysis {
                println!("✅ LLM analysis complete");
                println!("   - Documentation generated: {}", a.has_documentation);
                println!("   - Functions summarized: {}", a.summarized_count);
                println!("   - Issues found: {}", a.issues_count);
            }
            analysis.ok()
        } else {
            None
        };

        // Stage 5: Finalize
        let final_optimized = OptimizedProject {
            call_graph,
            ..optimized
        };
        let analysis = self.stage_finalize(final_optimized, llm_analysis);

        let duration = start_time.elapsed();
        println!("⏱️ Analysis completed in {:.2}s", duration.as_secs_f64());

        Ok(analysis)
    }

    pub async fn process_project_with_git(
        &mut self,
        root: &Path,
    ) -> Result<ProjectAnalysis, Box<dyn std::error::Error>> {
        let mut intelligence = self.process_project(root).await?;

        if self.config.enable_git {
            // Add git analysis
            if let Ok(git_analysis) = crate::analysis::git_analysis::GitAnalyzer::analyze(root) {
                println!("📊 Git Analysis:");
                println!("   Total commits: {}", git_analysis.total_commits);
                println!(
                    "   Top authors: {}",
                    git_analysis
                        .top_authors
                        .iter()
                        .take(3)
                        .map(|(name, count)| format!("{} ({})", name, count))
                        .collect::<Vec<_>>()
                        .join(", ")
                );

                // Add git activity scores to function importance
                for idx in intelligence.call_graph.node_indices() {
                    let func = &intelligence.call_graph[idx];
                    let score = git_analysis.file_activity_score(Path::new(&func.file));
                    if score > 0.0 {
                        if let Some(call_graph) = Arc::get_mut(&mut intelligence.call_graph) {
                            call_graph[idx].importance_score =
                                (call_graph[idx].importance_score + score * 0.3).min(1.0);
                        }
                    }
                }
            }
        }

        Ok(intelligence)
    }

    // ========================================================================
    // LLM Analysis - Now async
    // ========================================================================

    async fn run_llm_analysis(
        &mut self,
        call_graph: &CallGraph,
        files: &[ParsedFile],
    ) -> Result<LLMAnalysis, String> {
        let engine = self
            .code_understanding
            .as_mut()
            .ok_or_else(|| "LLM engine not initialized".to_string())?;

        let mut analysis = LLMAnalysis::default();

        // Only generate documentation (1 call instead of 20+)
        println!("   📝 Generating documentation...");
        match engine.generate_documentation(call_graph, files).await {
            Ok(doc) => {
                analysis.documentation = Some(doc);
                analysis.has_documentation = true;
            }
            Err(e) => {
                eprintln!("   ❌ Failed to generate documentation: {}", e);
            }
        }

        // Summarize only top 3 functions (instead of 10)
        let mut important_functions: Vec<_> = call_graph
            .node_indices()
            .map(|idx| (&call_graph[idx], idx))
            .collect();
        important_functions.sort_by(|a, b| {
            b.0.importance_score
                .partial_cmp(&a.0.importance_score)
                .unwrap()
        });

        let mut source_map = std::collections::HashMap::new();
        for file in files {
            source_map.insert(file.path.clone(), file.source.clone());
        }

        let mut summaries = Vec::new();
        let mut issues = Vec::new();

        // Keep this small on constrained hardware — each function costs
        // 1-2 sequential LLM round-trips. 3 is a reasonable default;
        // raise it once you've confirmed acceptable latency.
        const MAX_FUNCTIONS_TO_ANALYZE: usize = 3;
        const RUN_BUG_ANALYSIS: bool = false; // toggle on if you want it

        for (func, _idx) in important_functions.iter().take(MAX_FUNCTIONS_TO_ANALYZE) {
            if let Some(source) = source_map.get(&func.file) {
                // Summarize
                match engine.summarize_function(func, source).await {
                    Ok(summary) => {
                        summaries.push((func.name.clone(), summary));
                        analysis.summarized_count += 1;
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to summarize {}: {}", func.name, e);
                    }
                }

                // Find issues (optional — doubles the LLM calls when on)
                if RUN_BUG_ANALYSIS {
                    match engine.analyze_bugs(func, source).await {
                        Ok(issues_list) => {
                            for issue in issues_list {
                                issues.push((func.name.clone(), issue));
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to analyze {}: {}", func.name, e);
                        }
                    }
                }
            }
        }

        analysis.function_summaries = summaries;
        analysis.issues_count = issues.len();
        analysis.issues = issues;
        Ok(analysis)
    }

    // ========================================================================
    // File Collection
    // ========================================================================

    fn collect_files(&self, root: &Path) -> Vec<PathBuf> {
        use walkdir::WalkDir;

        let skip_dirs = [
            ".git",
            "target",
            "node_modules",
            "__pycache__",
            ".venv",
            "venv",
            "dist",
            "build",
            ".idea",
            ".vscode",
            ".dart_tool",
            ".pub",
            ".gradle",
            "vendor",
        ];

        let supported_extensions = ["rs", "py", "js", "jsx", "ts", "tsx", "go", "java"];

        let skip_files = [
            "package-lock.json",
            "yarn.lock",
            "Cargo.lock",
            "Gemfile.lock",
            "poetry.lock",
            "Pipfile.lock",
        ];

        WalkDir::new(root)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_str().unwrap_or("");
                !skip_dirs.contains(&name)
            })
            .filter_map(|e| e.ok())
            .filter(|e| {
                if !e.path().is_file() {
                    return false;
                }
                if let Some(name) = e.path().file_name().and_then(|n| n.to_str()) {
                    if skip_files.contains(&name) {
                        return false;
                    }
                }
                if let Some(ext) = e.path().extension().and_then(|e| e.to_str()) {
                    if supported_extensions.contains(&ext) {
                        if let Ok(meta) = e.metadata() {
                            if meta.len() == 0 || meta.len() > self.config.max_file_size {
                                return false;
                            }
                        }
                        return true;
                    }
                }
                false
            })
            .take(self.config.max_files)
            .map(|e| e.path().to_path_buf())
            .collect()
    }

    // ========================================================================
    // Call Graph Building
    // ========================================================================

    fn build_call_graph(&self, files: &[ParsedFile]) -> CallGraph {
        use std::collections::HashMap;

        let mut call_graph = CallGraph::new();
        let mut func_index: HashMap<String, petgraph::graph::NodeIndex> = HashMap::new();
        let mut func_by_name: HashMap<String, Vec<String>> = HashMap::new();
        let mut import_map: HashMap<String, Vec<String>> = HashMap::new();

        // First pass: Build import map
        for file in files {
            for import in &file.imports {
                let module = &import.module;
                for item in &import.items {
                    let full_path = format!("{}::{}", module, item);
                    import_map.entry(item.clone()).or_default().push(full_path);
                }
                import_map
                    .entry(import.module.clone())
                    .or_default()
                    .push(module.clone());
            }
        }

        // Second pass: Add all functions and index them
        for file in files {
            let file_path = file.path.clone();
            for func in &file.functions {
                // Include the enclosing impl block's type (if any) in the
                // identity. Without this, two different types both defining
                // a same-named method (e.g. two `new()`s in one file) would
                // collide on the exact same key below and silently overwrite
                // each other in `func_index` — which is what was producing
                // the self-loop edges.
                let full_path = match &func.container {
                    Some(c) => format!("{}::{}::{}", file_path, c, func.name),
                    None => format!("{}::{}", file_path, func.name),
                };
                let node = FunctionNode {
                    name: func.name.clone(),
                    full_path: full_path.clone(),
                    file: file_path.clone(),
                    line: func.line,
                    is_public: func.is_public,
                    is_async: func.is_async,
                    params: func.params.iter().map(|p| p.name.clone()).collect(),
                    returns: func.return_type.clone().into_iter().collect(),
                    complexity: 1.0,
                    importance_score: 0.0,
                    doc_comment: func.doc_comment.clone(),
                    writes_to: Vec::new(),
                    reads_from: Vec::new(),
                    errors: Vec::new(),
                    fan_in: 0,
                    fan_out: 0,
                    is_cycle: false,
                    depth: 0,
                    layer: String::new(),
                    trait_impl: func.trait_impl.clone(),
                };
                let idx = call_graph.add_function(node);
                func_index.insert(full_path.clone(), idx);
                func_by_name
                    .entry(func.name.clone())
                    .or_default()
                    .push(full_path);
            }
        }

        // Trait-method index for operator-overload resolution (Tier OP,
        // below). Deliberately conservative: without full type inference we
        // can't pin down which impl an operand actually uses, so any
        // function whose enclosing impl matches the expected trait+method
        // is treated as a possible target — a blanket edge rather than a
        // guessed-specific one.
        let mut trait_method_index: HashMap<(String, String), Vec<petgraph::graph::NodeIndex>> =
            HashMap::new();
        for idx in call_graph.node_indices() {
            let node = &call_graph[idx];
            if let Some(trait_name) = &node.trait_impl {
                let base = Self::base_trait_name(trait_name);
                trait_method_index
                    .entry((base, node.name.clone()))
                    .or_default()
                    .push(idx);
            }
        }

        // Third pass: Build edges with import resolution
        for file in files {
            let file_path = file.path.clone();
            for func in &file.functions {
                let caller_path = match &func.container {
                    Some(c) => format!("{}::{}::{}", file_path, c, func.name),
                    None => format!("{}::{}", file_path, func.name),
                };
                if let Some(&caller_idx) = func_index.get(&caller_path) {
                    for called_name in &func.calls {
                        let mut found = false;

                        // ============================================================
                        // TIER OP: Operator overloads (index/add/sub/mul/div/rem)
                        // ============================================================
                        if called_name.starts_with("op::") {
                            let method = called_name.trim_start_matches("op::");
                            let expected: &[(&str, &str)] = match method {
                                "index" => &[("Index", "index"), ("IndexMut", "index_mut")],
                                "add" => &[("Add", "add")],
                                "sub" => &[("Sub", "sub")],
                                "mul" => &[("Mul", "mul")],
                                "div" => &[("Div", "div")],
                                "rem" => &[("Rem", "rem")],
                                _ => &[],
                            };
                            for (trait_name, method_name) in expected {
                                if let Some(idxs) = trait_method_index
                                    .get(&(trait_name.to_string(), method_name.to_string()))
                                {
                                    for &callee_idx in idxs {
                                        call_graph.add_call(
                                            caller_idx,
                                            callee_idx,
                                            CallEdge {
                                                call_type: "operator_overload".to_string(),
                                                line: func.line,
                                            },
                                        );
                                    }
                                }
                            }
                        }

                        // ============================================================
                        // TIER 0: Method call on self (self.method_name)
                        // ============================================================
                        if !found && called_name.starts_with("self::") {
                            let method_name = called_name.trim_start_matches("self::");
                            // Look for a method with this name in the same impl block
                            if let Some(container) = &func.container {
                                let full_path =
                                    format!("{}::{}::{}", file_path, container, method_name);
                                if let Some(&callee_idx) = func_index.get(&full_path) {
                                    call_graph.add_call(
                                        caller_idx,
                                        callee_idx,
                                        CallEdge {
                                            call_type: "self_method".to_string(),
                                            line: func.line,
                                        },
                                    );
                                    found = true;
                                }
                            }
                        }

                        // 1. Qualified call (e.g. "GraphVizOutput::new") —
                        // match container + method directly. This is the
                        // most reliable tier now that tree_sitter keeps the
                        // qualifier instead of stripping it.
                        if let Some((qualifier, method)) = called_name.rsplit_once("::") {
                            let qualified_path =
                                format!("{}::{}::{}", file_path, qualifier, method);
                            if let Some(&callee_idx) = func_index.get(&qualified_path) {
                                call_graph.add_call(
                                    caller_idx,
                                    callee_idx,
                                    CallEdge {
                                        call_type: "exact".to_string(),
                                        line: func.line,
                                    },
                                );
                                found = true;
                            }
                        }

                        let simple_name = called_name.rsplit("::").next().unwrap_or(called_name);

                        // 2. Unqualified, same file: only accept if there's
                        // exactly one candidate with this name in this file.
                        // An ambiguous match (0 or 2+ candidates) is skipped
                        // rather than guessed — a missing edge is far less
                        // misleading than a wrong one.
                        if !found {
                            let same_file_candidates: Vec<_> = func_index
                                .iter()
                                .filter(|(path, _)| {
                                    path.starts_with(&format!("{}::", file_path))
                                        && path.ends_with(&format!("::{}", simple_name))
                                        && path != &&caller_path
                                })
                                .collect();
                            if same_file_candidates.len() == 1 {
                                let (_, &callee_idx) = same_file_candidates[0];
                                call_graph.add_call(
                                    caller_idx,
                                    callee_idx,
                                    CallEdge {
                                        call_type: "exact".to_string(),
                                        line: func.line,
                                    },
                                );
                                found = true;
                            }
                        }

                        // 3. Import resolution
                        if !found {
                            if let Some(imported_paths) = import_map.get(simple_name) {
                                for imported_path in imported_paths {
                                    if let Some(&callee_idx) = func_index.get(imported_path) {
                                        call_graph.add_call(
                                            caller_idx,
                                            callee_idx,
                                            CallEdge {
                                                call_type: "imported".to_string(),
                                                line: func.line,
                                            },
                                        );
                                        found = true;
                                        break;
                                    }
                                }
                            }
                        }

                        // 4. Name match across files — only if unambiguous
                        // (exactly one function anywhere has this name).
                        // Previously this took the *first* match found,
                        // which silently picked an arbitrary same-named
                        // function across the whole codebase.
                        if !found {
                            if let Some(paths) = func_by_name.get(simple_name) {
                                let candidates: Vec<_> =
                                    paths.iter().filter(|p| *p != &caller_path).collect();
                                if candidates.len() == 1 {
                                    if let Some(&callee_idx) = func_index.get(candidates[0]) {
                                        call_graph.add_call(
                                            caller_idx,
                                            callee_idx,
                                            CallEdge {
                                                call_type: "by_name".to_string(),
                                                line: func.line,
                                            },
                                        );
                                        found = true;
                                    }
                                }
                            }
                        }

                        // Tier 4 ("partial", substring match against any
                        // full_path) intentionally removed — it was the
                        // least reliable resolution and the most likely
                        // source of nonsensical edges. An unresolved call
                        // is simply dropped now instead of guessed.
                        let _ = found;
                    }
                }
            }
        }

        call_graph
    }
    /// Normalizes a captured trait name for matching — strips generics
    /// ("Index<usize>" → "Index") and path qualifiers ("std::ops::Add" → "Add").
    fn base_trait_name(raw: &str) -> String {
        let no_generics = raw.split('<').next().unwrap_or(raw).trim();
        no_generics
            .rsplit("::")
            .next()
            .unwrap_or(no_generics)
            .to_string()
    }

    #[allow(dead_code)]
    fn hash_project(files: &[PathBuf]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        for file in files {
            if let Some(name) = file.to_str() {
                hasher.update(name.as_bytes());
            }
            if let Ok(meta) = std::fs::metadata(file) {
                if let Ok(modified) = meta.modified() {
                    if let Ok(duration) = modified.elapsed() {
                        hasher.update(duration.as_secs().to_string().as_bytes());
                    }
                }
            }
        }
        format!("{:x}", hasher.finalize())
    }
}

// ============================================================================
// Project Intelligence (deprecated - kept for compatibility)
// ============================================================================

pub struct ProjectIntelligence {
    pub call_graph: CallGraph,
    pub files: Vec<ParsedFile>,
    pub root: PathBuf,
    pub llm_analysis: Option<LLMAnalysis>,
}

impl ProjectIntelligence {
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
            self.call_graph.node_count()
        ));
        output.push_str(&format!("- **Files**: {}\n", self.files.len()));
        output.push_str(&format!(
            "- **Relationships**: {}\n\n",
            self.call_graph.edge_count()
        ));

        // LLM Analysis
        if let Some(ref llm) = self.llm_analysis {
            output.push_str("## 🤖 LLM Analysis\n\n");

            if let Some(ref doc) = llm.documentation {
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
        functions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

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
            serde_json::Value::Number(self.call_graph.node_count().into()),
        );
        report.insert(
            "total_files".to_string(),
            serde_json::Value::Number(self.files.len().into()),
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
            if let Some(ref doc) = llm.documentation {
                output.push_str(&doc);
                output.push_str("\n\n");
            }
        }

        output.push_str(&full);
        output
    }
}

// ============================================================================
// LLM Analysis Results
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct LLMAnalysis {
    pub has_documentation: bool,
    pub documentation: Option<String>,
    pub function_summaries: Vec<(String, String)>,
    pub issues: Vec<(String, crate::llm::CodeIssue)>,
    pub summarized_count: usize,
    pub issues_count: usize,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_config_default() {
        let config = PipelineConfig::default();
        assert!(!config.enable_llm);
        assert!(!config.enable_git);
        assert_eq!(config.llm_temperature, 0.3);
        assert_eq!(config.max_files, 10000);
    }

    #[test]
    fn test_pipeline_new() {
        let pipeline = Pipeline::new();
        assert!(!pipeline.config.enable_llm);
        assert!(pipeline.llm_provider.is_none());
        assert!(pipeline.code_understanding.is_none());
    }

    #[test]
    fn test_pipeline_with_config() {
        let config = PipelineConfig {
            enable_llm: true,
            max_files: 100,
            ..Default::default()
        };
        let pipeline = Pipeline::new().with_config(config);
        assert!(pipeline.config.enable_llm);
        assert_eq!(pipeline.config.max_files, 100);
    }
}
