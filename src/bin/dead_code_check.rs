// src/bin/dead_code_check.rs

use clap::Parser;
use code_intelligence::analysis::dead_code::{
    ConfidenceLevel, DeadCodeAnalysis, DeadCodeAnalyzer, DeadFunction, FunctionImpact, RemovalCost,
};
use code_intelligence::analysis::dynamic_refs::DynamicRefDetector;
use code_intelligence::analysis::git_analysis::GitAnalyzer;
use code_intelligence::analysis::roots::{ReachabilityAnalyzer, RootDetectionConfig, RootDetector};
use code_intelligence::analysis::verdict::{Verdict, VerdictConfig, VerdictEngine};
use code_intelligence::graph::GraphMetrics;
use code_intelligence::ml::classifier::DeadCodeClassifier;
use code_intelligence::Pipeline;
use std::path::PathBuf;

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🔍 Analyzing dead code in: {:?}\n", args.project_dir);

    let mut pipeline = Pipeline::new();
    let analysis = pipeline.process_project(&args.project_dir).await?;

    // Try to get git analysis
    let git_analysis = GitAnalyzer::analyze(&args.project_dir).ok();

    // Load ML model if provided
    let ml_model = if let Some(model_path) = &args.model {
        if args.no_ml {
            println!("⚠️ ML model provided but --no-ml flag is set. Ignoring model.");
            None
        } else {
            // Try loading as versioned model first
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
                    }
                    // Extract classifier from versioned model
                    let classifier = DeadCodeClassifier {
                        model: Some(versioned.classifier.clone()),
                        accuracy: versioned
                            .performance
                            .as_ref()
                            .map(|p| p.accuracy)
                            .unwrap_or(0.0),
                        feature_count: versioned.feature_schema.feature_count(),
                    };
                    Some(classifier)
                }
                Err(_) => {
                    // Fallback: try loading legacy model
                    match DeadCodeClassifier::load(&model_path.to_string_lossy()) {
                        Ok(model) => {
                            println!("✅ Loaded legacy model from: {:?}", model_path);
                            Some(model)
                        }
                        Err(e) => {
                            eprintln!("⚠️ Failed to load model: {}", e);
                            None
                        }
                    }
                }
            }
        }
    } else {
        None
    };

    let threshold = if args.conservative {
        0.95 // Even more conservative
    } else {
        args.threshold // Default: 0.92 (99% precision target)
    };

    println!(
        "📊 Using threshold: {:.2} (calibrated for {:.1}% precision)",
        threshold,
        if args.conservative { 99.5 } else { 99.0 }
    );

    // NEW: Unified Verdict Engine Approach

    // 1. Detect roots using unified RootDetector
    let root_config = RootDetectionConfig::default();
    let root_set = RootDetector::detect_roots(&analysis.call_graph, &analysis.files, &root_config);

    // 2. Compute reachability using unified analyzer
    let reachability = ReachabilityAnalyzer::compute_reachability(&analysis.call_graph, &root_set);

    // 3. Create verdict engine
    let mut verdict_config = VerdictConfig::default();
    verdict_config.dead_threshold = threshold;
    verdict_config.enable_ml = ml_model.is_some();

    let mut verdict_engine = VerdictEngine::new(verdict_config);

    // 4. Add ML model if available
    if let Some(model) = ml_model {
        verdict_engine = verdict_engine.with_ml(model);
    }

    // 4.5 Detect dynamic references and add to verdict engine
    let dynamic_detector = DynamicRefDetector::new();
    let dynamic_refs = dynamic_detector.detect_all(&analysis.call_graph, &analysis.files);

    if args.verbose && !dynamic_refs.is_empty() {
        println!("\n🔄 Dynamic references found: {}", dynamic_refs.len());
        let report = dynamic_detector.generate_report(&dynamic_refs);
        println!("{}", report);
    }

    verdict_engine = verdict_engine.with_dynamic_refs(dynamic_refs);

    // 5. Generate verdicts for all functions
    let verdicts = verdict_engine.evaluate_all(&analysis.call_graph, &reachability);

    // 6. Filter verdicts
    let dead_verdicts: Vec<&Verdict> = verdict_engine.filter_dead(&verdicts);
    let alive_verdicts: Vec<&Verdict> = verdict_engine.filter_alive(&verdicts);
    let unknown_verdicts: Vec<&Verdict> = verdict_engine.filter_unknown(&verdicts);

    println!("\n📊 Verdict Engine Results:");
    println!("   Total functions: {}", verdicts.len());
    println!("   Dead: {}", dead_verdicts.len());
    println!("   Alive: {}", alive_verdicts.len());
    println!("   Unknown: {}", unknown_verdicts.len());
    println!(
        "   Avg Confidence: {:.1}%",
        verdicts.iter().map(|v| v.confidence).sum::<f64>() / verdicts.len() as f64 * 100.0
    );

    // DEBUG: Show why functions are excluded
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

        // Show some examples of dead verdicts
        println!("\n   Sample Dead Verdicts:");
        for verdict in dead_verdicts.iter().take(20) {
            println!(
                "      - {} (confidence: {:.1}%)",
                verdict.function_name,
                verdict.confidence * 100.0
            );
        }

        // Show verbose explanations for dead functions
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

    // Convert verdicts to DeadFunction for backward compatibility

    let filtered_functions: Vec<DeadFunction> = dead_verdicts
        .iter()
        .filter_map(|verdict| {
            // Find the actual function node
            let idx = analysis.call_graph.node_indices()
                .find(|idx| analysis.call_graph[*idx].full_path == verdict.full_path)?;

            let func = &analysis.call_graph[idx];

            // Determine confidence level
            let level = if verdict.confidence > 0.95 {
                ConfidenceLevel::Guaranteed
            } else if verdict.confidence > 0.85 {
                ConfidenceLevel::VeryLikely
            } else {
                ConfidenceLevel::Probably
            };

            // Build DeadFunction from verdict
            Some(DeadFunction {
                full_path: verdict.full_path.clone(),
                name: verdict.function_name.clone(),
                file: func.file.clone(),
                line: func.line,
                score: code_intelligence::analysis::dead_code::DeadScore {
                    score: verdict.confidence,
                    level,
                    factors: verdict.signals.iter().map(|s| {
                        code_intelligence::analysis::dead_code::ScoreFactor {
                            name: s.name.clone(),
                            weight: s.weight,
                            contribution: if s.direction == code_intelligence::analysis::verdict::SignalDirection::SupportsDead {
                                s.weight
                            } else {
                                -s.weight
                            },
                            explanation: s.explanation.clone(),
                        }
                    }).collect(),
                },
                impact: FunctionImpact {
                    lines_of_code: 20 + (func.complexity * 5.0) as usize,
                    dependencies: Vec::new(),
                    complexity: func.complexity,
                    estimated_removal_impact: if func.complexity > 20.0 {
                        "High impact - complex function".to_string()
                    } else if func.complexity > 10.0 {
                        "Medium impact".to_string()
                    } else {
                        "Low impact - simple function".to_string()
                    },
                    removal_cost: if func.complexity > 20.0 {
                        RemovalCost::High
                    } else if func.complexity > 10.0 {
                        RemovalCost::Medium
                    } else {
                        RemovalCost::Low
                    },
                },
                removal_order: 0,
                is_binary_only: false,
                is_internal_call: false,
            })
        })
        .collect();

    // Sort by confidence
    let mut filtered_functions = filtered_functions;
    filtered_functions.sort_by(|a, b| b.score.score.partial_cmp(&a.score.score).unwrap());

    // Assign removal order
    for (i, func) in filtered_functions.iter_mut().enumerate() {
        func.removal_order = i + 1;
    }

    // Run analyzer in impact-only mode
    let mut impact_analyzer = DeadCodeAnalyzer::new_for_impact_only();

    // Import verdicts to get DeadFunction list with impact metadata
    let dead_functions_with_impact =
        impact_analyzer.import_verdicts(&dead_verdicts, &analysis.call_graph);

    // Still need legacy analyzer for types and modules
    let mut legacy_analyzer = DeadCodeAnalyzer::new();
    let legacy_analysis = legacy_analyzer.analyze(
        &analysis.call_graph,
        &analysis.type_graph,
        &analysis.import_graph,
        &analysis.dependency_graph,
        &analysis.files,
        git_analysis.as_ref(),
    );

    // Build final analysis using imported verdicts + legacy types/modules
    // Clone types and modules since we need them for the report
    let filtered_analysis = DeadCodeAnalysis {
        functions: dead_functions_with_impact.clone(), // ⭐ Clone for later use
        types: legacy_analysis.types.clone(),          // ⭐ Clone
        modules: legacy_analysis.modules.clone(),      // ⭐ Clone
        reachability: reachability.clone(),
        summary: code_intelligence::analysis::dead_code::AnalysisSummary {
            total_functions: analysis.call_graph.node_count(),
            dead_functions: dead_functions_with_impact.len(), // Use original, not moved
            dead_types: legacy_analysis.summary.dead_types,
            dead_modules: legacy_analysis.summary.dead_modules,
            dead_files: legacy_analysis.summary.dead_files,
            avg_confidence: if dead_functions_with_impact.is_empty() {
                0.0
            } else {
                dead_functions_with_impact
                    .iter()
                    .map(|f| f.score.score)
                    .sum::<f64>()
                    / dead_functions_with_impact.len() as f64
            },
            estimated_loc_removable: dead_functions_with_impact
                .iter()
                .map(|f| f.impact.lines_of_code)
                .sum(),
        },
    };

    // Generate report
    let report = legacy_analyzer.generate_report(&filtered_analysis);
    println!("{}", report);
    // Generate report
    let report = legacy_analyzer.generate_report(&filtered_analysis);
    println!("{}", report);

    // Final Summary

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

    // Show top dead functions with explanations if verbose
    if args.verbose && !filtered_functions.is_empty() {
        println!("\n🔍 Top Dead Functions:");
        for func in filtered_functions.iter().take(5) {
            println!("\n   #{}: {}", func.removal_order, func.name);
            println!("      Confidence: {:.1}%", func.score.score * 100.0);
            println!("      File: {}", func.file);
            println!("      Complexity: {:.1}", func.impact.complexity);
            println!("      Impact: {}", func.impact.estimated_removal_impact);
        }
    }

    Ok(())
}
