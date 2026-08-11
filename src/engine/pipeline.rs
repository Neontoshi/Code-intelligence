// src/engine/pipeline.rs

use crate::analysis::context::{ProjectAnalysis, ProjectAnalysisBuilder};
use crate::analysis::features::FeatureExtractor;
use crate::analysis::importance::ImportanceScorer;
use crate::engine::cache::{AnalysisCacheManager, CachedFileEntry, FileCache};
use crate::engine::call_graph_builder::CallGraphBuilder;
use crate::engine::config::PipelineConfig;
use crate::engine::file_collector::FileCollector;
use crate::engine::indexer::IndexBuilder;
use crate::engine::llm_analysis::{LLMAnalysis, LLMAnalyzer};
use crate::engine::stages::{AnalyzedProject, OptimizedProject, ParsedProject, RawProject};
use crate::graph::call_graph::CallGraph;
use crate::graph::project_graph::ProjectGraphBuilder;
use crate::graph::traits::GraphMetrics;
use crate::llm::{create_ollama_phi2, CodeUnderstandingEngine, LLMProvider};
use crate::parser::tree_sitter::{ParsedFile, TreeSitterParser};
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

// ============================================================================
// Pipeline Struct
// ============================================================================
#[allow(dead_code)]
pub struct Pipeline {
    parser: TreeSitterParser,
    scorer: ImportanceScorer,
    cache: FileCache,
    config: PipelineConfig,
    llm_provider: Option<Arc<dyn LLMProvider>>,
    code_understanding: Option<CodeUnderstandingEngine>,
    analysis_cache: Option<AnalysisCacheManager>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            parser: TreeSitterParser::new(),
            scorer: ImportanceScorer::new(),
            cache: FileCache::new(),
            config: PipelineConfig::default(),
            llm_provider: None,
            code_understanding: None,
            analysis_cache: None,
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
    // File Hash Collection for Cache
    // ========================================================================

    fn collect_file_hashes(&self, root: &Path) -> Vec<CachedFileEntry> {
        let mut entries = Vec::new();
        let supported_extensions = ["rs", "py", "js", "jsx", "ts", "tsx", "go", "java"];

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

    // ========================================================================
    // Stage Methods
    // ========================================================================

    pub fn stage_collect(&self, root: &Path) -> RawProject {
        FileCollector::collect(root, &self.config)
    }

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

    // ========================================================================
    // Main Processing Methods
    // ========================================================================

    pub async fn process_project(
        &mut self,
        root: &Path,
    ) -> Result<ProjectAnalysis, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let project_hash = self.cache.hash_content(&format!("{:?}", root));

        // Check if we have cached analysis
        if let Some(cache_mgr) = &self.analysis_cache {
            let file_entries = self.collect_file_hashes(root);

            if cache_mgr.has_valid_analysis(&project_hash, &file_entries) {
                if let Some(cached) = cache_mgr.load_analysis_metadata(&project_hash) {
                    println!("✅ Cache hit! Found cached analysis for {:?}", root);
                    println!("   Functions: {}", cached.function_count);
                    println!("   Edges: {}", cached.edge_count);
                    println!("   Files: {}", cached.file_count);

                    // For now, we still need to do full analysis since we don't store
                    // the complete ProjectAnalysis in cache yet.
                    // We'll add full serialization in a future iteration.
                    println!("   ⚠️ Cache hit only metadata. Full reconstruction coming soon.");
                }
            } else {
                println!("🔍 Cache miss or invalid. Running full analysis...");
            }
        }

        let raw = self.stage_collect(root);
        println!("📁 Found {} source files", raw.files.len());

        if raw.files.len() > self.config.max_files {
            println!(
                "⚠️ Too many files ({}), limiting to {}",
                raw.files.len(),
                self.config.max_files
            );
        }

        println!("🔄 Parsing files in parallel...");
        let parsed = self.stage_parse_parallel(raw);
        println!("✅ Successfully parsed {} files", parsed.files.len());

        if parsed.files.is_empty() {
            let analysis = ProjectAnalysisBuilder::new(root.to_path_buf())
                .with_call_graph(CallGraph::new())
                .build();

            // Cache empty result
            self.save_to_cache(&project_hash, root, &analysis)?;

            return Ok(analysis);
        }

        println!("🔄 Building graphs in parallel...");
        let analyzed = self.stage_analyze_parallel(parsed);
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

        let layers: std::collections::HashSet<String> = analyzed
            .call_graph
            .node_indices()
            .map(|idx| analyzed.call_graph[idx].layer.clone())
            .collect();
        let mut layer_list: Vec<_> = layers.into_iter().collect();
        layer_list.sort();
        println!("   📂 Layers: {:?}", layer_list);

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

        let mut call_graph = optimized.call_graph.clone();
        self.scorer.score_all(&mut call_graph);
        println!("📈 Scored function importance");

        let llm_analysis = if self.config.enable_llm && self.code_understanding.is_some() {
            println!("🤖 Running LLM analysis...");
            let engine = self.code_understanding.as_mut().unwrap();
            let analysis = LLMAnalyzer::analyze(engine, &call_graph, &optimized.files).await;
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

        let final_optimized = OptimizedProject {
            call_graph,
            ..optimized
        };
        let analysis = self.stage_finalize(final_optimized, llm_analysis);

        let duration = start_time.elapsed();
        println!("⏱️ Analysis completed in {:.2}s", duration.as_secs_f64());

        // Save to cache
        self.save_to_cache(&project_hash, root, &analysis)?;

        Ok(analysis)
    }

    pub async fn process_project_with_git(
        &mut self,
        root: &Path,
    ) -> Result<ProjectAnalysis, Box<dyn std::error::Error>> {
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

    // ========================================================================
    // Cache Helpers
    // ========================================================================

    fn save_to_cache(
        &self,
        project_hash: &str,
        root: &Path,
        analysis: &ProjectAnalysis,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
}
