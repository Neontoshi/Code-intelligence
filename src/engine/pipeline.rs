// src/engine/pipeline.rs

use crate::analysis::context::{ProjectAnalysis, ProjectAnalysisBuilder};
use crate::analysis::features::FeatureExtractor;
use crate::analysis::importance::ImportanceScorer;
use crate::engine::cache::{AnalysisCacheManager, CachedFileEntry, FileCache};
use crate::engine::call_graph_builder::CallGraphBuilder;
use crate::engine::config::PipelineConfig;
use crate::engine::file_collector::FileCollector;
use crate::engine::incremental::{FileTracker, IncrementalResult, RebuildScope};
use crate::engine::indexer::IndexBuilder;
use crate::engine::llm_analysis::{LLMAnalysis, LLMAnalyzer};
use crate::engine::stages::{AnalyzedProject, OptimizedProject, ParsedProject, RawProject};
use crate::error::{err, Result};
use crate::graph::call_graph::CallGraph;
use crate::graph::project_graph::ProjectGraphBuilder;
use crate::graph::traits::GraphMetrics;
use crate::llm::{create_ollama_phi2, CodeUnderstandingEngine, LLMProvider};
use crate::logging::StructuredLogger;
use crate::parser::tree_sitter::{ParsedFile, TreeSitterParser};
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub type ProgressFn = Arc<dyn Fn(&str) + Send + Sync>;

#[derive(Debug, Clone, Default)]
pub struct BuildSummary {
    pub functions: usize,
    pub edges: usize,
    pub names: usize,
    pub files: usize,
    pub nodes: usize,
    pub proj_edges: usize,
    pub duplicates: usize,
}

pub struct Pipeline {
    _parser: TreeSitterParser,
    scorer: ImportanceScorer,
    cache: FileCache,
    config: PipelineConfig,
    llm_provider: Option<Arc<dyn LLMProvider>>,
    code_understanding: Option<CodeUnderstandingEngine>,
    analysis_cache: Option<AnalysisCacheManager>,
    progress: Option<ProgressFn>,
    last_build_summary: Option<BuildSummary>,
    logger: Option<std::sync::Mutex<StructuredLogger>>,
    file_tracker: Option<FileTracker>,
    enable_incremental: bool,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            _parser: TreeSitterParser::new(),
            scorer: ImportanceScorer::new(),
            cache: FileCache::new(),
            config: PipelineConfig::default(),
            llm_provider: None,
            code_understanding: None,
            analysis_cache: None,
            progress: None,
            last_build_summary: None,
            logger: None,
            file_tracker: None,
            enable_incremental: false,
        }
    }

    pub fn get_rebuild_scope(&self) -> Option<RebuildScope> {
        if let Some(_tracker) = &self.file_tracker {
            // This would be set during analysis
            // For now, return None
            None
        } else {
            None
        }
    }

    pub fn with_logging(mut self, logger: StructuredLogger) -> Self {
        self.logger = Some(std::sync::Mutex::new(logger));
        self
    }

    fn log_event(&self, event: &str, fields: HashMap<String, serde_json::Value>) {
        if let Some(logger) = &self.logger {
            if let Ok(mut logger) = logger.lock() {
                logger.info(event, fields);
            }
        }
    }

    fn report(&self, msg: &str) {
        // Progress reporting
        if let Some(f) = &self.progress {
            f(msg);
        }
        // Structured logging
        let mut fields = HashMap::new();
        fields.insert(
            "message".to_string(),
            serde_json::Value::String(msg.to_string()),
        );
        self.log_event("progress", fields);
    }

    // Update stage methods to log events
    pub fn stage_collect(&self, root: &Path) -> RawProject {
        self.log_event("stage_collect_started", {
            let mut fields = HashMap::new();
            fields.insert(
                "root".to_string(),
                serde_json::Value::String(root.to_string_lossy().to_string()),
            );
            fields
        });
        let result = FileCollector::collect(root, &self.config);
        self.log_event("stage_collect_completed", {
            let mut fields = HashMap::new();
            fields.insert(
                "files_found".to_string(),
                serde_json::Value::Number(result.files.len().into()),
            );
            fields
        });
        result
    }

    pub fn with_incremental(mut self, tracker: FileTracker) -> Self {
        self.file_tracker = Some(tracker);
        self.enable_incremental = true;
        self
    }

    pub fn enable_incremental(mut self) -> Self {
        self.enable_incremental = true;
        if self.file_tracker.is_none() {
            self.file_tracker = Some(FileTracker::new());
        }
        self
    }

    pub fn detect_changes(&mut self, files: &[ParsedFile]) -> Option<IncrementalResult> {
        if !self.enable_incremental {
            return None;
        }

        if let Some(tracker) = &mut self.file_tracker {
            let changed_files = tracker.detect_changes(files);
            if !changed_files.is_empty() {
                // Cache hit - we can skip full analysis
                let result = IncrementalResult {
                    changed_files,
                    affected_functions: Vec::new(),
                    removed_functions: Vec::new(),
                    added_functions: Vec::new(),
                    modified_functions: Vec::new(),
                    cache_hit: false,
                    rebuild_scope: RebuildScope::default(),
                };
                Some(result)
            } else {
                Some(IncrementalResult {
                    changed_files: Vec::new(),
                    affected_functions: Vec::new(),
                    removed_functions: Vec::new(),
                    added_functions: Vec::new(),
                    modified_functions: Vec::new(),
                    cache_hit: true,
                    rebuild_scope: RebuildScope::default(),
                })
            }
        } else {
            None
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

    pub fn with_cache_dir(mut self, cache_dir: PathBuf) -> Self {
        self.cache = self.cache.with_persistent_dir(cache_dir.clone());
        self.analysis_cache = Some(AnalysisCacheManager::new(&cache_dir));
        self
    }

    pub fn with_progress_reporter(mut self, f: ProgressFn) -> Self {
        self.progress = Some(f);
        self
    }
    pub fn take_build_summary(&mut self) -> Option<BuildSummary> {
        self.last_build_summary.take()
    }

    pub async fn with_ollama_phi2(mut self) -> Result<Self> {
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

    // File Hash Collection for Cache
    fn collect_file_hashes(&self, root: &Path) -> Vec<CachedFileEntry> {
        let mut entries = Vec::new();
        let supported_extensions = [
            "rs", "py", "js", "jsx", "ts", "tsx", "go", "java", "dart", "php", "cpp", "cc", "cxx",
            "hpp", "h", "cs",
        ];

        for entry in walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
        {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if supported_extensions.contains(&ext) {
                    if let Some(hash) = self.cache.hash_file(path) {
                        entries.push(CachedFileEntry {
                            path: path.to_string_lossy().to_string(),
                            content_hash: hash,
                        });
                    }
                }
            }
        }
        entries
    }

    pub fn stage_parse_parallel(&self, raw: RawProject) -> ParsedProject {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let total = raw.files.len();
        let completed = Arc::new(AtomicUsize::new(0));
        let failed = Arc::new(AtomicUsize::new(0));

        let parsed_files: Vec<ParsedFile> = raw
            .files
            .par_iter()
            .filter_map(|file| {
                let thread_parser = TreeSitterParser::new();
                let result = thread_parser.parse_file(file);

                let count = completed.fetch_add(1, Ordering::Relaxed) + 1;
                self.report(&format!("parsing files ({}/{})", count, total));

                match result {
                    Ok(parsed) => Some(parsed),
                    Err(e) => {
                        failed.fetch_add(1, Ordering::Relaxed);
                        eprintln!("⚠️ Parse failed for {:?}: {}", file, e);
                        None
                    }
                }
            })
            .collect();

        let total_funcs: usize = parsed_files.iter().map(|f| f.functions.len()).sum();
        let total_types: usize = parsed_files.iter().map(|f| f.types.len()).sum();

        println!(
            "🔎 Parsed {} files: {} functions, {} types (failures: {})",
            parsed_files.len(),
            total_funcs,
            total_types,
            failed.load(Ordering::Relaxed)
        );

        ParsedProject {
            root: raw.root,
            files: parsed_files,
        }
    }

    pub fn stage_analyze_parallel(&self, parsed: ParsedProject) -> AnalyzedProject {
        let call_graph = CallGraphBuilder::build(&parsed.files);

        let mut call_graph = call_graph;
        call_graph.calculate_fan_metrics();
        call_graph.detect_layers();
        call_graph.calculate_call_depth();

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
                let edge = crate::graph::call_graph::CallEdge {
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
            call_graph: call_graph.clone(),
            project_graph,
            cycle_detection_skipped: call_graph.cycle_detection_skipped,
            cycle_detection_node_count: call_graph.cycle_detection_node_count,
        }
    }

    pub fn stage_optimize_parallel(&self, analyzed: AnalyzedProject) -> OptimizedProject {
        let functions: Vec<crate::graph::call_graph::FunctionNode> = analyzed
            .call_graph
            .node_indices()
            .map(|idx| analyzed.call_graph[idx].clone())
            .collect();

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

        let mut feature_extractor = FeatureExtractor::new();
        for (path, feature) in features {
            feature_extractor.insert(path, feature);
        }

        let index_builder = IndexBuilder::new();
        let rich_indexes = index_builder.build_from_analysis(&functions, &analyzed.files);

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

    pub async fn process_project(&mut self, root: &Path) -> Result<ProjectAnalysis> {
        let start_time = std::time::Instant::now();
        let project_hash = self.cache.hash_content(&format!("{:?}", root));

        // Check for incremental changes
        let raw = self.stage_collect(root);
        self.report(&format!("found {} files", raw.files.len()));

        let parsed = self.stage_parse_parallel(raw);
        self.report(&format!("parsed {} files", parsed.files.len()));

        // Check if we can use incremental analysis
        if let Some(incremental_result) = self.detect_changes(&parsed.files) {
            if incremental_result.cache_hit {
                self.report("cache hit - using cached analysis");
                // Return cached analysis if available
                if let Some(cached) = self.load_from_cache(&project_hash) {
                    return Ok(cached);
                }
            } else {
                self.report(&format!(
                    "{} files changed, performing incremental analysis",
                    incremental_result.changed_files.len()
                ));
            }
        }

        // Check if we have cached analysis
        if let Some(cache_mgr) = &self.analysis_cache {
            let file_entries = self.collect_file_hashes(root);

            if cache_mgr.has_valid_analysis(&project_hash, &file_entries) {
                if let Some(cached) = cache_mgr.load_analysis_metadata(&project_hash) {
                    self.report(&format!(
                        "cache hit: {} functions, {} edges, {} files",
                        cached.function_count, cached.edge_count, cached.file_count
                    ));
                }
            } else {
                self.report("cache miss, running full analysis...");
            }
        }

        let raw = self.stage_collect(root);
        self.report(&format!("found {} files", raw.files.len()));

        let parsed = self.stage_parse_parallel(raw);
        self.report(&format!("parsed {} files", parsed.files.len()));

        if parsed.files.is_empty() {
            let analysis = ProjectAnalysisBuilder::new(root.to_path_buf())
                .with_call_graph(CallGraph::new())
                .build();

            // Cache empty result
            self.save_to_cache(&project_hash, root, &analysis)?;

            return Ok(analysis);
        }

        self.report("building graphs...");
        let analyzed = self.stage_analyze_parallel(parsed);

        let node_count = analyzed.call_graph.node_count();
        let mut cycle_count = 0;
        if node_count < 1000 {
            let mut call_graph_for_cycles = analyzed.call_graph.clone();
            call_graph_for_cycles.mark_cycle_members();
            cycle_count = call_graph_for_cycles
                .node_indices()
                .filter(|&idx| call_graph_for_cycles[idx].is_cycle)
                .count();
        }

        let layers: std::collections::HashSet<String> = analyzed
            .call_graph
            .node_indices()
            .map(|idx| analyzed.call_graph[idx].layer.clone())
            .collect();
        let mut layer_list: Vec<_> = layers.into_iter().collect();
        layer_list.sort();

        self.report("extracting features...");
        let optimized = self.stage_optimize_parallel(analyzed);

        let mut call_graph = optimized.call_graph.clone();
        self.scorer.score_all(&mut call_graph);

        self.last_build_summary = Some(BuildSummary {
            functions: call_graph.node_count(),
            edges: call_graph.edge_count(),
            names: optimized.rich_indexes.function_name.len(),
            files: optimized.rich_indexes.file_to_functions.len(),
            nodes: optimized.project_graph.node_count(),
            proj_edges: optimized.project_graph.edge_count(),
            duplicates: call_graph.duplicate_functions.len(),
        });
        let _ = cycle_count;
        let _ = layer_list;

        let llm_analysis = if self.config.enable_llm && self.code_understanding.is_some() {
            self.report("running LLM analysis...");
            let engine = self
                .code_understanding
                .as_mut()
                .ok_or_else(|| err::analysis("LLM engine not initialized"))?;
            let analysis = LLMAnalyzer::analyze(engine, &call_graph, &optimized.files).await;
            analysis.ok()
        } else {
            None
        };

        let final_optimized = OptimizedProject {
            call_graph,
            ..optimized
        };
        let analysis = self.stage_finalize(final_optimized, llm_analysis);

        let _duration = start_time.elapsed();

        // Save to cache
        self.save_to_cache(&project_hash, root, &analysis)?;

        Ok(analysis)
    }

    fn load_from_cache(&self, _project_hash: &str) -> Option<ProjectAnalysis> {
        // Implementation to load cached analysis
        None // Placeholder
    }

    pub async fn process_project_with_git(&mut self, root: &Path) -> Result<ProjectAnalysis> {
        let mut intelligence = self.process_project(root).await?;

        if self.config.enable_git {
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

    // Cache Helpers
    fn save_to_cache(
        &self,
        project_hash: &str,
        root: &Path,
        analysis: &ProjectAnalysis,
    ) -> Result<()> {
        if let Some(cache_mgr) = &self.analysis_cache {
            let file_entries = self.collect_file_hashes(root);

            // Store analysis cache metadata
            let cache_entry = crate::engine::cache::AnalysisCache {
                project_hash: project_hash.to_string(),
                files: file_entries.clone(),
                function_count: analysis.call_graph.node_count(),
                edge_count: analysis.call_graph.edge_count(),
                timestamp: chrono::Utc::now().timestamp(),
            };
            cache_mgr.put(project_hash, &cache_entry);

            // Store full analysis metadata
            let _ = cache_mgr.save_analysis(
                project_hash,
                root,
                analysis.call_graph.node_count(),
                analysis.call_graph.edge_count(),
                &file_entries,
            );

            println!("💾 Cached analysis for {:?}", root);
        }
        Ok(())
    }

    pub fn check_memory(&self) -> Result<()> {
        if let Some(limit_mb) = self.config.max_memory_mb {
            let current = self.get_current_memory_usage_mb();

            if current > limit_mb as f64 * 0.85 {
                eprintln!(
                    "⚠️ Memory usage {:.1}MB is approaching limit {}MB ({}%)",
                    current,
                    limit_mb,
                    (current / limit_mb as f64 * 100.0) as u8
                );
            }

            if current > limit_mb as f64 * 0.95 {
                return Err(err::analysis(format!(
                    "Memory limit {}MB nearly exceeded (current: {:.1}MB). \
                     Try reducing --max-files or --max-file-size",
                    limit_mb, current
                )));
            }

            if current > limit_mb as f64 {
                return Err(err::analysis(format!(
                    "Memory limit {}MB exceeded (current: {:.1}MB)",
                    limit_mb, current
                )));
            }
        }
        Ok(())
    }

    /// Gracefully degrade parallelism based on memory pressure
    pub fn adjust_parallelism(&self) -> usize {
        let default_threads = rayon::current_num_threads();

        if let Some(limit_mb) = self.config.max_memory_mb {
            let current = self.get_current_memory_usage_mb();
            let usage_ratio = current / limit_mb as f64;

            if usage_ratio > 0.8 {
                // Reduce threads to half
                let reduced = (default_threads / 2).max(1);
                eprintln!(
                    "🔽 Reducing parallelism from {} to {} threads (memory pressure)",
                    default_threads, reduced
                );
                return reduced;
            }
        }

        default_threads
    }

    fn get_current_memory_usage_mb(&self) -> f64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = std::fs::read_to_string("/proc/self/statm") {
                let parts: Vec<&str> = contents.split_whitespace().collect();
                if let Some(&size) = parts.first() {
                    if let Ok(pages) = size.parse::<f64>() {
                        let page_size = 4096.0; // Typical page size
                        return pages * page_size / 1024.0 / 1024.0;
                    }
                }
            }
        }
        // Fallback: estimate from allocated bytes
        let allocated = self.cache.len() * 1024; // Rough estimate
        allocated as f64 / 1024.0 / 1024.0
    }
}
