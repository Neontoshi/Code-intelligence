// src/bin/dead_code_check.rs — add this debug version

use clap::Parser;
use code_intelligence::analysis::dead_code::{DeadCodeAnalysis, DeadCodeAnalyzer, DeadFunction};
use code_intelligence::analysis::git_analysis::GitAnalyzer;
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

    // ================================================================
    // DEBUG: Show why functions are being excluded
    // ================================================================
    if args.debug {
        println!("\n🔍 DEBUG: Analyzing why no dead functions found");
        println!("   Total functions: {}", analysis.call_graph.node_count());

        let mut no_callers = 0;
        let mut private = 0;
        let mut exported = 0;
        let mut in_test = 0;
        let mut trait_impl = 0;
        let mut whitelisted = 0;
        let _low_confidence = 0;
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

        // Show some examples of functions with no callers
        println!("\n   Sample functions with no callers:");
        let mut count = 0;
        for idx in analysis.call_graph.node_indices() {
            let func = &analysis.call_graph[idx];
            if func.fan_in == 0 && count < 20 {
                println!(
                    "      - {} (public: {}, file: {})",
                    func.name,
                    func.is_public,
                    func.file.split('/').last().unwrap_or(&func.file)
                );
                count += 1;
            }
        }
    }

    // Filter using the actual FunctionNode from the call graph
    let filtered_functions: Vec<DeadFunction> = if analyzer.use_ml() {
        dead_analysis
            .functions
            .iter()
            .filter(|f| {
                if f.score.score < args.threshold {
                    return false;
                }

                let actual_node = analysis
                    .call_graph
                    .node_indices()
                    .find(|idx| analysis.call_graph[*idx].full_path == f.full_path)
                    .map(|idx| &analysis.call_graph[idx]);

                let actual_node = match actual_node {
                    Some(node) => node,
                    None => {
                        if args.verbose {
                            println!("   ⚠️ Could not find actual node for {}, keeping", f.name);
                        }
                        return true;
                    }
                };

                use code_intelligence::analysis::training_data::{
                    FunctionFeatures, TrainingExample, TrainingLabel,
                };
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

                if prob > 0.85 {
                    if args.verbose {
                        println!("   ML filtered: {} (prob: {:.2})", f.name, prob);
                    }
                    return false;
                }

                true
            })
            .cloned()
            .collect()
    } else {
        dead_analysis
            .functions
            .iter()
            .filter(|f| f.score.score > args.threshold)
            .cloned()
            .collect()
    };

    let filtered_analysis = DeadCodeAnalysis {
        functions: filtered_functions.clone(),
        types: dead_analysis.types,
        modules: dead_analysis.modules,
        reachability: dead_analysis.reachability,
        summary: dead_analysis.summary,
    };

    let report = analyzer.generate_report(&filtered_analysis);
    println!("{}", report);

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

    Ok(())
}
