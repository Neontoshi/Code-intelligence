// src/bin/dead_code_check.rs

use clap::Parser;
use code_intelligence::analysis::dead_code::{DeadCodeAnalysis, DeadCodeAnalyzer, DeadFunction};
use code_intelligence::analysis::git_analysis::GitAnalyzer;
use code_intelligence::analysis::training_data::{
    FunctionFeatures, TrainingExample, TrainingLabel,
};
use code_intelligence::ml::classifier::DeadCodeClassifier;
use code_intelligence::Pipeline;
use std::collections::HashSet;
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

    // Load ML model if provided
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
    let mut analyzer = DeadCodeAnalyzer::new();

    // Inject the ML model into the analyzer
    if let Some(model) = ml_model {
        analyzer = analyzer.with_classifier(model);
    }

    let dead_analysis = analyzer.analyze(
        &analysis.call_graph,
        &analysis.type_graph,
        &analysis.import_graph,
        &analysis.dependency_graph,
        &analysis.files,
        git_analysis.as_ref(),
    );

    // Filter using the actual FunctionNode from the call graph
    let filtered_functions: Vec<DeadFunction> = if analyzer.use_ml() {
        // Use ML model to filter
        dead_analysis
            .functions
            .iter()
            .filter(|f| {
                // First check confidence threshold
                if f.score.score < args.threshold {
                    return false;
                }

                // Look up the actual FunctionNode from the call graph
                let actual_node = analysis
                    .call_graph
                    .node_indices()
                    .find(|idx| analysis.call_graph[*idx].full_path == f.full_path)
                    .map(|idx| &analysis.call_graph[idx]);

                let actual_node = match actual_node {
                    Some(node) => node,
                    None => {
                        // If we can't find it, be conservative — keep it
                        if args.verbose {
                            println!("   ⚠️ Could not find actual node for {}, keeping", f.name);
                        }
                        return true;
                    }
                };

                // Create a training example using the REAL FunctionNode
                let example = TrainingExample {
                    function_name: actual_node.name.clone(),
                    full_path: actual_node.full_path.clone(),
                    file: actual_node.file.clone(),
                    language: TrainingExample::detect_language(&actual_node.file),
                    features: FunctionFeatures::from_function(actual_node, &analysis.call_graph),
                    label: TrainingLabel::Unknown,
                    confidence: 0.0,
                    source: "ml".to_string(),
                };

                let prob = analyzer
                    .get_ml_model()
                    .map(|model| model.predict_probability(&example))
                    .unwrap_or(0.5);

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

    // Create filtered analysis
    let filtered_analysis = DeadCodeAnalysis {
        functions: filtered_functions.clone(),
        types: dead_analysis.types,
        modules: dead_analysis.modules,
        reachability: dead_analysis.reachability,
        summary: dead_analysis.summary,
    };

    // ⭐ FIX: Use the analyzer instance to generate the report
    let report = analyzer.generate_report(&filtered_analysis);
    println!("{}", report);

    // Show detailed filtered stats
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

    // Show what was filtered out
    if dead_analysis.functions.len() > filtered_analysis.functions.len() {
        println!("\n📋 Filtered out (false positives):");
        let filtered_names: HashSet<String> = filtered_analysis
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
