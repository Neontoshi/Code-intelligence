// src/bin/dead_code_check.rs

use clap::Parser; // ⭐ NEW - for better argument parsing
use code_intelligence::analysis::dead_code::{DeadCodeAnalysis, DeadCodeDetector};
use code_intelligence::analysis::git_analysis::GitAnalyzer;
use code_intelligence::ml::classifier::DeadCodeClassifier; // ⭐ NEW
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
    #[arg(long, default_value = "0.70")]
    threshold: f64,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🔍 Analyzing dead code in: {:?}\n", args.project_dir);

    let mut pipeline = Pipeline::new();
    let analysis = pipeline.process_project(&args.project_dir).await?;

    // Try to get git analysis
    let git_analysis = GitAnalyzer::analyze(&args.project_dir).ok();

    // ⭐ NEW: Load ML model if provided
    let ml_model = if let Some(model_path) = &args.model {
        if args.no_ml {
            println!("⚠️ ML model provided but --no-ml flag is set. Ignoring model.");
            None
        } else {
            match DeadCodeClassifier::load(&model_path.to_string_lossy()) {
                Ok(model) => {
                    println!("✅ Loaded ML model from: {:?}", model_path);
                    Some(model)
                }
                Err(e) => {
                    eprintln!("⚠️ Failed to load ML model: {}", e);
                    eprintln!("   Continuing without ML support.");
                    None
                }
            }
        }
    } else {
        None
    };

    // Run comprehensive dead code analysis
    let dead_analysis = DeadCodeDetector::analyze(
        &analysis.call_graph,
        &analysis.type_graph,
        &analysis.import_graph,
        &analysis.dependency_graph,
        &analysis.files,
        git_analysis.as_ref(),
    );

    // ⭐ NEW: Filter with ML model if available
    let filtered_functions: Vec<_> = if let Some(model) = ml_model {
        // Use ML model to filter
        dead_analysis
            .functions
            .iter()
            .filter(|f| {
                // First check confidence threshold
                if f.score.score < args.threshold {
                    return false;
                }

                // Then use ML model if available
                use code_intelligence::analysis::training_data::{
                    FunctionFeatures, TrainingExample, TrainingLabel,
                };

                let example = TrainingExample {
                    function_name: f.name.clone(),
                    full_path: f.full_path.clone(),
                    file: f.file.clone(),
                    language: TrainingExample::detect_language(&f.file),
                    features: FunctionFeatures::from_function(
                        &code_intelligence::graph::call_graph::FunctionNode {
                            name: f.name.clone(),
                            full_path: f.full_path.clone(),
                            file: f.file.clone(),
                            line: f.line,
                            is_public: false, // Would need actual value
                            is_async: false,
                            params: vec![],
                            returns: vec![],
                            complexity: f.impact.complexity,
                            importance_score: 0.0,
                            doc_comment: None,
                            writes_to: vec![],
                            reads_from: vec![],
                            errors: vec![],
                            fan_in: 0,
                            fan_out: 0,
                            is_cycle: false,
                            depth: 0,
                            layer: String::new(),
                            trait_impl: None,
                        },
                        &analysis.call_graph,
                    ),
                    label: TrainingLabel::Unknown,
                    confidence: 0.0,
                    source: "ml".to_string(),
                };

                let prob = model.predict_probability(&example);

                // If ML says it's highly likely alive, skip it
                if prob > 0.85 {
                    if args.verbose {
                        println!("   ML filtered: {} (prob: {:.2})", f.name, prob);
                    }
                    return false;
                }

                // Keep if ML says it's dead or uncertain
                true
            })
            .cloned()
            .collect()
    } else {
        // Use traditional filtering without ML
        dead_analysis
            .functions
            .iter()
            .filter(|f| f.score.score > args.threshold)
            .cloned()
            .collect()
    };

    // ⭐ Create filtered analysis
    let filtered_analysis = DeadCodeAnalysis {
        functions: filtered_functions.clone(),
        types: dead_analysis.types,
        modules: dead_analysis.modules,
        reachability: dead_analysis.reachability,
        summary: dead_analysis.summary,
    };

    // Generate report with filtered results
    let report = DeadCodeDetector::generate_report(&filtered_analysis);
    println!("{}", report);

    // ⭐ Show detailed filtered stats
    println!("\n📊 Filtered Results:");
    println!(
        "   Original dead functions: {}",
        dead_analysis.functions.len()
    );
    println!(
        "   Remaining dead functions: {}",
        filtered_analysis.functions.len()
    );
    println!(
        "   Filtered false positives: {}",
        dead_analysis.functions.len() - filtered_analysis.functions.len()
    );
    println!("   Confidence threshold: > {:.0}%", args.threshold * 100.0);

    if args.model.is_some() && !args.no_ml {
        println!("   ML Model: enabled ✅");
    } else {
        println!("   ML Model: disabled");
    }

    // ⭐ Show what was filtered out
    if dead_analysis.functions.len() > filtered_analysis.functions.len() {
        println!("\n📋 Filtered out (false positives):");
        let filtered_names: std::collections::HashSet<String> = filtered_analysis
            .functions
            .iter()
            .map(|f| f.full_path.clone())
            .collect();

        for f in dead_analysis.functions.iter() {
            if !filtered_names.contains(&f.full_path) {
                println!(
                    "   - {} (from {})",
                    f.name,
                    f.file.split('/').last().unwrap_or("")
                );
            }
        }
    }

    Ok(())
}
