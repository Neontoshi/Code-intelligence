// src/bin/hard_negative_dataset.rs

//! Hard-negative dataset generation
//!
//! This tool creates a dataset of functions that LOOK dead but are actually alive.
//! These are the most valuable training examples for improving the model.

use clap::Parser;
use code_intelligence::analysis::dead_code::filters::is_never_dead;
use code_intelligence::analysis::roots::{ReachabilityAnalyzer, RootDetectionConfig, RootDetector};
use code_intelligence::analysis::training_data::TrainingExample;
use code_intelligence::graph::GraphMetrics;
use code_intelligence::Pipeline;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "Generate hard-negative dataset")]
struct Args {
    /// Project directory to analyze
    project_dir: PathBuf,

    /// Output file for hard-negative dataset
    #[arg(short, long, default_value = "hard_negatives.json")]
    output: PathBuf,

    /// Number of hard negatives to collect
    #[arg(long, default_value = "100")]
    count: usize,

    /// Model path (optional - for confidence filtering)
    #[arg(long)]
    model: Option<PathBuf>,

    /// Include only functions with confidence > this threshold
    #[arg(long, default_value = "0.7")]
    min_confidence: f64,
}

/// Hard-negative categories
#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub enum HardNegativeCategory {
    /// Trait implementations that look dead but are used polymorphically
    TraitImpl,
    /// Framework callbacks (React hooks, Spring handlers, etc.)
    FrameworkCallback,
    /// FFI functions called from external code
    FFI,
    /// Public API functions used by external callers
    PublicAPI,
    /// Dynamic dispatch targets (reflection, plugin systems)
    DynamicDispatch,
    /// Generated code that appears dead but is used by code generation
    GeneratedCode,
    /// Entry points that are called by the runtime
    EntryPoint,
    /// Test/benchmark functions
    TestFunction,
    /// Functions with zero callers but used by macros
    MacroUsed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardNegative {
    pub full_path: String,
    pub function_name: String,
    pub file: String,
    pub line: usize,
    pub category: HardNegativeCategory,
    pub confidence: f64,
    pub reason: String,
    pub evidence: Vec<String>,
    pub language: String,
    pub repository: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if !args.project_dir.is_dir() {
        eprintln!("❌ Project directory not found: {:?}", args.project_dir);
        std::process::exit(1);
    }

    println!(
        "🔍 Generating hard-negative dataset from: {:?}",
        args.project_dir
    );
    println!("📁 Output: {:?}", args.output);
    println!("🎯 Target count: {}", args.count);
    println!("📊 Min confidence: {:.1}%", args.min_confidence * 100.0);

    // Run analysis
    let mut pipeline = Pipeline::new();
    let analysis = pipeline.process_project(&args.project_dir).await?;

    println!(
        "📊 Analysis complete: {} functions",
        analysis.call_graph.node_count()
    );

    // Detect roots and reachability
    let root_config = RootDetectionConfig::default();
    let root_set = RootDetector::detect_roots(&analysis.call_graph, &analysis.files, &root_config);
    let reachability = ReachabilityAnalyzer::compute_reachability(&analysis.call_graph, &root_set);

    // Collect hard negatives
    let mut hard_negatives = Vec::new();

    for idx in analysis.call_graph.node_indices() {
        let func = &analysis.call_graph[idx];

        // Skip if already dead or unknown
        let is_reachable = reachability.is_reachable(&func.full_path);
        let has_callers = func.fan_in > 0;

        // Only consider functions that look dead (no callers, unreachable)
        if is_reachable || has_callers {
            continue;
        }

        // Skip if it's actually dead (not a hard negative)
        if is_never_dead(func) {
            continue;
        }

        // Check categories
        let categories = detect_hard_negative_categories(func, &analysis);
        if categories.is_empty() {
            continue;
        }

        // Calculate confidence (how likely this is a hard negative)
        let confidence = calculate_hard_negative_confidence(func, &categories);

        if confidence < args.min_confidence {
            continue;
        }

        for category in categories {
            let hard_negative = HardNegative {
                full_path: func.full_path.clone(),
                function_name: func.name.clone(),
                file: func.file.clone(),
                line: func.line,
                category: category.clone(),
                confidence,
                reason: format!("Function looks dead but is actually {:?}", category),
                evidence: collect_evidence(func, &category),
                language: TrainingExample::detect_language(&func.file),
                repository: args.project_dir.to_string_lossy().to_string(),
            };
            hard_negatives.push(hard_negative);
        }

        if hard_negatives.len() >= args.count {
            break;
        }
    }

    // Save dataset
    let json = serde_json::to_string_pretty(&hard_negatives)?;
    std::fs::write(&args.output, json)?;

    // Print stats
    println!("\n📊 Hard Negative Dataset Generated:");
    println!("   Total: {}", hard_negatives.len());

    let mut by_category: HashMap<HardNegativeCategory, usize> = HashMap::new();
    for hn in &hard_negatives {
        *by_category.entry(hn.category.clone()).or_insert(0) += 1;
    }

    println!("\n   By Category:");
    for (category, count) in by_category {
        println!("      {:?}: {}", category, count);
    }

    let avg_confidence: f64 =
        hard_negatives.iter().map(|hn| hn.confidence).sum::<f64>() / hard_negatives.len() as f64;

    println!("\n   Avg confidence: {:.1}%", avg_confidence * 100.0);
    println!("\n✅ Dataset saved to: {:?}", args.output);

    Ok(())
}

fn detect_hard_negative_categories(
    func: &code_intelligence::graph::call_graph::FunctionNode,
    _analysis: &code_intelligence::analysis::context::ProjectAnalysis,
) -> Vec<HardNegativeCategory> {
    let mut categories = Vec::new();

    // 1. Trait implementations
    if func.trait_impl.is_some() {
        categories.push(HardNegativeCategory::TraitImpl);
    }

    // 2. Framework callbacks - check file path
    if func.file.contains("/handlers/")
        || func.file.contains("/controllers/")
        || func.file.contains("/routes/")
        || func.file.contains("/components/")
        || func.file.contains("/hooks/")
    {
        categories.push(HardNegativeCategory::FrameworkCallback);
    }

    // 3. FFI
    if func.file.contains("/ffi/")
        || func.file.contains("/extern/")
        || func.name.contains("extern")
        || func.name.contains("ffi")
    {
        categories.push(HardNegativeCategory::FFI);
    }

    // 4. Public API
    if func.is_public && func.fan_in == 0 {
        categories.push(HardNegativeCategory::PublicAPI);
    }

    // 5. Dynamic dispatch - check for reflection patterns in name or file
    if func.file.contains("/plugins/")
        || func.file.contains("/plugin/")
        || func.file.contains("/dynamic/")
        || func.name.contains("plugin")
        || func.name.contains("dynamic")
        || func.name.contains("register")
    {
        categories.push(HardNegativeCategory::DynamicDispatch);
    }

    // 6. Generated code
    if func.file.contains("/generated/")
        || func.file.contains("/gen/")
        || func.file.contains(".gen.go")
        || func.file.contains("_gen.rs")
        || func.file.contains("/protobuf/")
        || func.file.contains("/pb/")
    {
        categories.push(HardNegativeCategory::GeneratedCode);
    }

    // 7. Entry points
    let entry_point_names = ["main", "async_main", "run", "start", "init", "setup"];
    if entry_point_names.contains(&func.name.as_str()) {
        categories.push(HardNegativeCategory::EntryPoint);
    }

    // 8. Test functions
    if func.is_test || func.name.starts_with("test_") || func.name.starts_with("bench_") {
        categories.push(HardNegativeCategory::TestFunction);
    }

    // 9. Macro used - check for macros in doc comment
    if let Some(doc) = &func.doc_comment {
        if doc.contains("macro") || doc.contains("#[derive") || doc.contains("proc_macro") {
            categories.push(HardNegativeCategory::MacroUsed);
        }
    }

    categories
}

fn calculate_hard_negative_confidence(
    func: &code_intelligence::graph::call_graph::FunctionNode,
    categories: &[HardNegativeCategory],
) -> f64 {
    let mut confidence = 0.5;

    // More categories = higher confidence
    confidence += (categories.len() as f64 - 1.0) * 0.1;

    // Public functions are more likely to be hard negatives
    if func.is_public {
        confidence += 0.15;
    }

    // Functions with documentation are more likely to be intentional
    if func.doc_comment.is_some() {
        confidence += 0.1;
    }

    // Trait implementations are very likely hard negatives
    if categories.contains(&HardNegativeCategory::TraitImpl) {
        confidence += 0.15;
    }

    // Framework callbacks are very likely hard negatives
    if categories.contains(&HardNegativeCategory::FrameworkCallback) {
        confidence += 0.15;
    }

    // FFI functions are hard negatives
    if categories.contains(&HardNegativeCategory::FFI) {
        confidence += 0.2;
    }

    confidence.min(1.0)
}

fn collect_evidence(
    func: &code_intelligence::graph::call_graph::FunctionNode,
    category: &HardNegativeCategory,
) -> Vec<String> {
    let mut evidence = Vec::new();

    match category {
        HardNegativeCategory::TraitImpl => {
            if let Some(trait_name) = &func.trait_impl {
                evidence.push(format!("Implements trait: {}", trait_name));
            }
            evidence.push("May be called polymorphically".to_string());
        }
        HardNegativeCategory::FrameworkCallback => {
            evidence.push("Located in framework directory".to_string());
            if func.file.contains("/handlers/") {
                evidence.push("HTTP handler - called by routing framework".to_string());
            }
            if func.file.contains("/components/") {
                evidence.push("React/Vue component - called by rendering framework".to_string());
            }
        }
        HardNegativeCategory::FFI => {
            evidence.push("External function interface".to_string());
            evidence.push("Called from external code (C, Python, etc.)".to_string());
        }
        HardNegativeCategory::PublicAPI => {
            evidence.push("Public function with no internal callers".to_string());
            evidence.push("Used by external consumers of the library".to_string());
        }
        HardNegativeCategory::DynamicDispatch => {
            evidence.push("Uses reflection or dynamic dispatch".to_string());
            evidence.push("Called via plugin system or dependency injection".to_string());
        }
        HardNegativeCategory::GeneratedCode => {
            evidence.push("Generated code - may be used by build system".to_string());
        }
        HardNegativeCategory::EntryPoint => {
            evidence.push("Application entry point".to_string());
        }
        HardNegativeCategory::TestFunction => {
            evidence.push("Test function - called by test runner".to_string());
        }
        HardNegativeCategory::MacroUsed => {
            evidence.push("Used by macros or procedural macros".to_string());
        }
    }

    // Common evidence
    if func.doc_comment.is_some() {
        evidence.push("Has documentation comment".to_string());
    }

    if func.is_public {
        evidence.push("Function is public".to_string());
    }

    evidence
}
