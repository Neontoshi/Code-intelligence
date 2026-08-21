// src/bin/dead_code_check.rs

use clap::Parser;
use code_intelligence::analysis::dead_code::DeadCodeAnalyzer;
use code_intelligence::analysis::dynamic_refs::DynamicRefDetector;
use code_intelligence::analysis::git_analysis::GitAnalyzer;
use code_intelligence::analysis::roots::{ReachabilityAnalyzer, RootDetectionConfig, RootDetector};
use code_intelligence::analysis::verdict::{Verdict, VerdictConfig, VerdictEngine};
use code_intelligence::analysis::AnalysisMetadata;
use code_intelligence::graph::GraphMetrics;
use code_intelligence::ml::classifier::DeadCodeClassifier;
use code_intelligence::Pipeline;
use serde_json;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(author, version, about = "Dead Code Analyzer with ML Support")]
struct Args {
    /// Project directory to analyze
    project_dir: PathBuf,

    /// Path to ML model file (optional)
    #[arg(long)]
    model: Option<PathBuf>,

    /// Disable ML model (use only whitelist)
    #[arg(long)]
    no_ml: bool,

    /// Confidence threshold (0.0 - 1.0)
    #[arg(long, default_value = "0.92")]
    threshold: f64,

    /// Use conservative mode (higher threshold for fewer false positives)
    #[arg(long)]
    conservative: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Show debug info about why functions are excluded
    #[arg(long)]
    debug: bool,

    /// Enable disk cache for faster repeat runs
    #[arg(long)]
    cache: bool,

    /// Cache directory (default: <project>/.code-intelligence-cache)
    #[arg(long)]
    cache_dir: Option<PathBuf>,
}

fn get_current_commit(project_dir: &Path) -> String {
    let output = Command::new("git")
        .current_dir(project_dir)
        .args(["rev-parse", "HEAD"])
        .output();

    match output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        _ => "unknown".to_string(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let file_count = std::fs::read_dir(&args.project_dir)
        .map(|d| d.filter_map(|e| e.ok()).count())
        .unwrap_or(0);

    if file_count > 1000 {
        eprintln!(
            "⚠️ Large project detected ({} files). This may take a while.",
            file_count
        );
        eprintln!("   Consider using --max-files to limit analysis.");
    }

    println!("🔍 Analyzing dead code in: {:?}\n", args.project_dir);

    let mut pipeline = Pipeline::new();

    if args.cache {
        let cache_path = args
            .cache_dir
            .clone()
            .unwrap_or_else(|| args.project_dir.join(".code-intelligence-cache"));
        pipeline = pipeline.with_cache_dir(cache_path.clone());
        if args.verbose {
            println!("💾 Cache enabled: {:?}", cache_path);
        }
    }

    let analysis = pipeline.process_project(&args.project_dir).await?;

    println!("\n📊 Call Resolution Statistics:");
    let stats = analysis.call_graph.resolution_stats();
    println!("   Resolution rate: {:.1}%", stats.resolution_rate * 100.0);
    println!("   Exact matches: {}", stats.exact_count);
    println!("   Inferred: {}", stats.inferred_count);
    println!("   Heuristic: {}", stats.heuristic_count);
    println!("   Ambiguous: {}", stats.ambiguous_count);
    println!("   Unresolved: {}", stats.unresolved_calls);

    if stats.unresolved_calls > 0 {
        println!(
            "\n⚠️ {} unresolved calls detected - may affect accuracy",
            stats.unresolved_calls
        );
    }

    let _git_analysis = GitAnalyzer::analyze(&args.project_dir).ok();

    let ml_model = if let Some(model_path) = &args.model {
        if args.no_ml {
            println!("⚠️ ML model provided but --no-ml flag is set. Ignoring model.");
            None
        } else {
            match code_intelligence::ml::model_serialization::VersionedModel::load(
                &model_path.to_string_lossy(),
            ) {
                Ok(versioned) => {
                    println!("✅ Loaded versioned model from: {:?}", model_path);
                    println!("   Version: {}", versioned.version);
                    println!("   Created: {}", versioned.created_at);

                    if let Some(perf) = versioned.get_performance() {
                        println!(
                            "   Performance: F1={:.1}%, Precision={:.1}%",
                            perf.f1 * 100.0,
                            perf.precision * 100.0
                        );
                        println!("   Stored Threshold: {:.2}", perf.threshold);
                    }

                    if let Some(cal) = versioned.get_calibration() {
                        println!("   Calibration: {:?}", cal.method);
                        println!("   Calibration samples: {}", cal.num_samples);
                    }

                    let classifier = DeadCodeClassifier {
                        model: Some(versioned.classifier.clone()),
                        accuracy: versioned
                            .performance
                            .as_ref()
                            .map(|p| p.accuracy)
                            .unwrap_or(0.0),
                        feature_count: versioned.feature_schema.feature_count(),
                        calibration: versioned.calibration.clone(),
                    };

                    let model_threshold = versioned.get_threshold();
                    println!("   Using model threshold: {:.2}", model_threshold);

                    Some((classifier, model_threshold))
                }
                Err(_) => match DeadCodeClassifier::load(&model_path.to_string_lossy()) {
                    Ok(model) => {
                        println!("✅ Loaded legacy model from: {:?}", model_path);
                        Some((model, args.threshold))
                    }
                    Err(e) => {
                        eprintln!("⚠️ Failed to load model: {}", e);
                        None
                    }
                },
            }
        }
    } else {
        None
    };

    let (_model, model_threshold) = match &ml_model {
        Some((m, t)) => (Some(m), Some(*t)),
        None => (None, None),
    };

    let cli_threshold = if args.conservative {
        0.95
    } else {
        args.threshold
    };
    let effective_threshold = model_threshold.unwrap_or(cli_threshold);
    let threshold = effective_threshold;

    println!(
        "📊 Using threshold: {:.2} (calibrated for {:.1}% precision)",
        threshold,
        if args.conservative { 99.5 } else { 99.0 }
    );

    if let Some((model, _)) = &ml_model {
        // Check schema compatibility
        if !model.is_schema_compatible() {
            eprintln!("⚠️ Warning: Model schema is incompatible with current feature schema");
            eprintln!("   {}", model.schema_info());
            eprintln!("   Continuing may produce incorrect results.");
        } else {
            println!("✅ Model schema compatible: {}", model.schema_info());
        }

        // Validate feature vectors
        if args.verbose {
            println!("🔍 Validating feature vectors...");
            let mut valid_count = 0;
            let mut invalid_count = 0;

            for idx in analysis.call_graph.node_indices() {
                let func = &analysis.call_graph[idx];
                let features =
                    code_intelligence::analysis::training_data::FunctionFeatures::from_function(
                        func,
                        &analysis.call_graph,
                    );
                let vec = features.to_feature_vector();

                match model.validate_features(&vec) {
                    Ok(_) => valid_count += 1,
                    Err(e) => {
                        invalid_count += 1;
                        if args.debug {
                            eprintln!("   Invalid features for {}: {}", func.name, e);
                        }
                    }
                }
            }

            println!("   Valid feature vectors: {}", valid_count);
            if invalid_count > 0 {
                println!("   ⚠️ Invalid feature vectors: {}", invalid_count);
            }
        }
    }

    let root_config = RootDetectionConfig::default();
    let root_set = RootDetector::detect_roots(&analysis.call_graph, &analysis.files, &root_config);
    let reachability = ReachabilityAnalyzer::compute_reachability(&analysis.call_graph, &root_set);

    let mut verdict_config = VerdictConfig::default();
    verdict_config.enable_ml = ml_model.is_some();

    let mut verdict_engine = VerdictEngine::new(verdict_config);

    if let Some((model, _)) = &ml_model {
        if let Some(_cal) = &model.calibration {
            verdict_engine = verdict_engine.with_dead_threshold(effective_threshold);
        } else {
            verdict_engine = verdict_engine.with_dead_threshold(effective_threshold);
        }
    } else {
        verdict_engine = verdict_engine.with_dead_threshold(effective_threshold);
    }

    if let Some((model, _)) = ml_model {
        verdict_engine = verdict_engine.with_ml(model);
    }

    let dynamic_detector = DynamicRefDetector::new();
    let dynamic_refs = dynamic_detector.detect_all(&analysis.call_graph, &analysis.files);

    if args.verbose && !dynamic_refs.is_empty() {
        println!("\n🔄 Dynamic references found: {}", dynamic_refs.len());
        let report = dynamic_detector.generate_report(&dynamic_refs);
        println!("{}", report);
    }

    verdict_engine = verdict_engine.with_dynamic_refs(dynamic_refs);

    let verdicts = verdict_engine.evaluate_all(&analysis.call_graph, &reachability);

    use code_intelligence::analysis::dead_code::{filter_reason, is_never_dead};

    let dead_verdicts: Vec<&Verdict> = verdict_engine.filter_dead(&verdicts);
    let alive_verdicts: Vec<&Verdict> = verdict_engine.filter_alive(&verdicts);
    let unknown_verdicts: Vec<&Verdict> = verdict_engine.filter_unknown(&verdicts);

    let filtered_dead_verdicts: Vec<&Verdict> = dead_verdicts
        .into_iter()
        .filter(|v| {
            if let Some(func) = analysis.get_function(&v.full_path) {
                if is_never_dead(func) {
                    if args.verbose {
                        println!(
                            "   ⏭️ Filtered out: {} ({})",
                            v.function_name,
                            filter_reason(func).unwrap_or("unknown")
                        );
                    }
                    return false;
                }
            }
            true
        })
        .collect();

    let dead_verdicts = filtered_dead_verdicts;

    if args.model.is_some() && !args.no_ml {
        let project_name = args
            .project_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let mut tracker = code_intelligence::analysis::OutcomeTracker::new(&args.project_dir);
        let tracked = tracker.import_verdicts(&dead_verdicts, &project_name);

        if tracked > 0 {
            println!("📝 Tracked {} dead functions in {}", tracked, project_name);
            if args.verbose {
                println!("   Use `cargo run --bin update_outcome` to mark them as removed/false positive");
                println!("   Or check .code-intelligence-outcomes.json");
            }
        }
    }

    println!("\n📊 Verdict Engine Results:");
    println!("   Total functions: {}", verdicts.len());
    println!("   Dead: {}", dead_verdicts.len());
    println!("   Alive: {}", alive_verdicts.len());
    println!("   Unknown: {}", unknown_verdicts.len());
    println!(
        "   Avg Confidence: {:.1}%",
        verdicts.iter().map(|v| v.confidence).sum::<f64>() / verdicts.len() as f64 * 100.0
    );

    if args.debug {
        println!("\n🔍 DEBUG: Function Analysis");
        println!("   Total functions: {}", analysis.call_graph.node_count());

        let mut no_callers = 0;
        let mut private = 0;
        let mut exported = 0;
        let mut in_test = 0;
        let mut trait_impl = 0;
        let mut whitelisted = 0;
        let mut has_callers = 0;

        for idx in analysis.call_graph.node_indices() {
            let func = &analysis.call_graph[idx];

            if func.fan_in == 0 {
                no_callers += 1;
            }

            if !func.is_public {
                private += 1;
            }

            if func.is_public && func.fan_in == 0 {
                exported += 1;
            }

            if func.file.contains("/tests/") || func.file.ends_with("_test.rs") {
                in_test += 1;
            }

            if func.trait_impl.is_some() {
                trait_impl += 1;
            }

            if code_intelligence::analysis::dead_code::WHITELIST.is_whitelisted(&func.name) {
                whitelisted += 1;
            }

            if func.fan_in > 0 {
                has_callers += 1;
            }
        }

        println!("\n   Functions with no callers: {}", no_callers);
        println!("   Private functions: {}", private);
        println!("   Exported (public + no callers): {}", exported);
        println!("   Test functions: {}", in_test);
        println!("   Trait implementations: {}", trait_impl);
        println!("   Whitelisted functions: {}", whitelisted);
        println!("   Functions with callers: {}", has_callers);

        let filtered_count = dead_verdicts
            .iter()
            .filter(|v| {
                analysis
                    .get_function(&v.full_path)
                    .map(|f| is_never_dead(f))
                    .unwrap_or(false)
            })
            .count();
        println!("   Filtered (framework/traits): {}", filtered_count);

        println!("\n   Sample Dead Verdicts:");
        for verdict in dead_verdicts.iter().take(20) {
            println!(
                "      - {} (confidence: {:.1}%)",
                verdict.function_name,
                verdict.confidence * 100.0
            );
        }

        if args.verbose {
            println!("\n   🔍 Detailed Dead Function Explanations:");
            for verdict in dead_verdicts.iter().take(10) {
                println!("\n{}", verdict.format_explanation());
            }
            if dead_verdicts.len() > 10 {
                println!("   ... and {} more", dead_verdicts.len() - 10);
            }
        }
    }

    let mut analyzer = DeadCodeAnalyzer::new();
    let dead_functions_with_impact = analyzer.import_verdicts(&dead_verdicts, &analysis.call_graph);

    let filtered_analysis = analyzer.analyze_structural_components(
        dead_functions_with_impact.clone(),
        reachability,
        &analysis.call_graph,
        &analysis.type_graph,
        &analysis.import_graph,
    );

    let report = analyzer.generate_report(&filtered_analysis);
    println!("{}", report);

    let metadata = AnalysisMetadata {
        analysis_id: format!("analysis_{}", chrono::Utc::now().timestamp()),
        model_version: args
            .model
            .as_ref()
            .map(|p| {
                p.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })
            .unwrap_or_else(|| "unknown".to_string()),
        feature_schema_version: 1,
        source_commit: get_current_commit(&args.project_dir),
        analysis_timestamp: chrono::Utc::now().timestamp(),
        total_functions: analysis.call_graph.node_count(),
        dead_candidates: filtered_analysis.functions.len(),
    };

    let metadata_path = args.project_dir.join(".code-intelligence-metadata.json");
    if let Ok(json) = serde_json::to_string_pretty(&metadata) {
        let _ = std::fs::write(&metadata_path, json);
        if args.verbose {
            println!("💾 Analysis metadata saved to: {:?}", metadata_path);
        }
    }

    println!("\n📊 Final Results:");
    println!("   Dead functions: {}", filtered_analysis.functions.len());
    println!("   Dead types: {}", filtered_analysis.summary.dead_types);
    println!(
        "   Dead modules: {}",
        filtered_analysis.summary.dead_modules
    );
    println!("   Dead files: {}", filtered_analysis.summary.dead_files);
    println!(
        "   Avg confidence: {:.1}%",
        filtered_analysis.summary.avg_confidence * 100.0
    );
    println!(
        "   Estimated LOC removable: {}",
        filtered_analysis.summary.estimated_loc_removable
    );

    if args.model.is_some() && !args.no_ml {
        println!("   ML Model: enabled ✅");
    } else {
        println!("   ML Model: disabled");
    }

    if args.verbose && !dead_functions_with_impact.is_empty() {
        println!("\n🔍 Top Dead Functions:");
        for func in dead_functions_with_impact.iter().take(5) {
            println!("\n   #{}: {}", func.removal_order, func.name);
            println!("      Confidence: {:.1}%", func.score.score * 100.0);
            println!("      File: {}", func.file);
            println!("      Complexity: {:.1}", func.impact.complexity);
            println!("      Impact: {}", func.impact.estimated_removal_impact);
        }
    }
    use code_intelligence::bin::common::exit_codes::ExitCode;

    // Exit with proper code based on results
    if filtered_analysis.functions.len() > 0 {
        std::process::exit(ExitCode::AnalysisFoundIssues.as_i32());
    } else {
        std::process::exit(ExitCode::Success.as_i32());
    }
}
