// src/bin/phase2_evaluation.rs

//! Phase 2 Evaluation - Comprehensive evaluation suite
//!
//! Runs:
//! 1. Feature ablation study (224 features)
//! 2. Repository-isolated evaluation
//! 3. Temporal evaluation
//! 4. Model comparison: static only vs ML only vs static + ML
//! 5. Calibration metrics

use clap::Parser;
use code_intelligence::analysis::training_data::{TrainingExample, TrainingLabel};
use code_intelligence::error::Result;
use code_intelligence::ml::classifier::{DeadCodeClassifier, LinearClassifier};
use code_intelligence::ml::feature_schema::{FeatureCategory, FEATURE_SCHEMA};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(author, version, about = "Phase 2 Evaluation Suite")]
struct Args {
    /// Training data file
    #[arg(short, long, default_value = "data/train.json")]
    train_data: PathBuf,

    /// Validation data file
    #[arg(short, long, default_value = "data/val.json")]
    val_data: PathBuf,

    /// Test data file
    #[arg(short, long, default_value = "data/test.json")]
    test_data: PathBuf,

    /// Output directory
    #[arg(short, long, default_value = "phase2_results")]
    output_dir: PathBuf,

    /// Skip ablation study (faster)
    #[arg(long)]
    skip_ablation: bool,

    /// Skip temporal evaluation (requires timestamps)
    #[arg(long)]
    skip_temporal: bool,

    /// Number of temporal windows
    #[arg(long, default_value = "5")]
    temporal_windows: usize,

    /// Minimum examples per temporal window
    #[arg(long, default_value = "50")]
    min_window_examples: usize,

    /// Seed for reproducibility
    #[arg(long, default_value = "42")]
    seed: u64,
}

// ============ RESULT TYPES ============

#[derive(Debug, Clone, Serialize)]
struct Phase2Results {
    ablation: Option<Vec<AblationResult>>,
    repository_isolated: Option<RepoIsolationResult>,
    temporal: Option<Vec<TemporalWindowResult>>,
    model_comparison: ModelComparisonResult,
    calibration: CalibrationMetrics,
    summary: SummaryReport,
    reproducibility: ReproducibilityInfo,
    per_language: Vec<LanguageMetrics>,
    leakage: LeakageReport,
    unseen_repo: Option<UnseenRepoResult>,
}

#[derive(Debug, Clone, Serialize)]
struct ReproducibilityInfo {
    command: String,
    train_data: String,
    val_data: String,
    test_data: String,
    feature_schema_version: u32,
    timestamp: i64,
    seed: u64,
}

#[derive(Debug, Clone, Serialize)]
struct AblationResult {
    feature_set: String,
    feature_count: usize,
    accuracy: f64,
    precision: f64,
    recall: f64,
    f1: f64,
    train_time_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
struct UnseenRepoResult {
    train_repos: Vec<String>,
    test_repos: Vec<String>,
    accuracy: f64,
    precision: f64,
    recall: f64,
    f1: f64,
    fpr: f64,
    fnr: f64,
    specificity: f64,
}

#[derive(Debug, Clone, Serialize)]
struct RepoIsolationResult {
    by_repository: Vec<RepoMetrics>,
    average_accuracy: f64,
    average_f1: f64,
    std_accuracy: f64,
    std_f1: f64,
    leakage_detected: bool,
}

#[derive(Debug, Clone, Serialize)]
struct RepoMetrics {
    repository: String,
    train_examples: usize,
    test_examples: usize,
    accuracy: f64,
    precision: f64,
    recall: f64,
    f1: f64,
}

#[derive(Debug, Clone, Serialize)]
struct LanguageMetrics {
    language: String,
    examples: usize,
    accuracy: f64,
    precision: f64,
    recall: f64,
    f1: f64,
    fpr: f64,
    fnr: f64,
    specificity: f64,
}

#[derive(Debug, Clone, Serialize)]
struct TemporalWindowResult {
    window: usize,
    train_period: String,
    test_period: String,
    train_examples: usize,
    test_examples: usize,
    accuracy: f64,
    precision: f64,
    recall: f64,
    f1: f64,
    degradation_from_first: f64,
}

#[derive(Debug, Clone, Serialize)]
struct ModelComparisonResult {
    static_only: ModelMetrics,
    ml_only: ModelMetrics,
    static_plus_ml: ModelMetrics,
}

#[derive(Debug, Clone, Serialize)]
struct LeakageTestResult {
    test_name: String,
    leaked: bool,
    details: String,
    severity: String,
}

#[derive(Debug, Clone, Serialize)]
struct LeakageReport {
    tests: Vec<LeakageTestResult>,
    total_leaks: usize,
    passed: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ModelMetrics {
    accuracy: f64,
    precision: f64,
    recall: f64,
    f1: f64,
    fpr: f64,
    fnr: f64,
    specificity: f64,
    pr_auc: f64,
    roc_auc: f64,
}

#[derive(Debug, Clone, Serialize)]
struct CalibrationMetrics {
    expected_calibration_error: f64,
    maximum_calibration_error: f64,
    brier_score: f64,
    log_loss: f64,
    reliability_curve: Vec<ReliabilityBin>,
}

#[derive(Debug, Clone, Serialize)]
struct ReliabilityBin {
    confidence_bin: String,
    expected_accuracy: f64,
    observed_accuracy: f64,
    count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct SummaryReport {
    best_ablation_config: Option<String>,
    repository_isolation_score: Option<f64>,
    temporal_degradation: Option<f64>,
    best_model: String,
    calibration_quality: String,
    recommendations: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("🔬 Phase 2 Evaluation Suite");
    println!("===========================\n");

    // Create output directory
    std::fs::create_dir_all(&args.output_dir)?;

    // Load data
    println!("📊 Loading data...");
    let train_examples = load_examples(&args.train_data)?;
    let val_examples = load_examples(&args.val_data)?;
    let test_examples = load_examples(&args.test_data)?;

    println!("   Train: {} examples", train_examples.len());
    println!("   Val: {} examples", val_examples.len());
    println!("   Test: {} examples\n", test_examples.len());

    // 1. Ablation Study
    let ablation = if !args.skip_ablation {
        println!("{}", "=".repeat(60));
        println!("🧪 STEP 1: Feature Ablation Study");
        println!("{}", "=".repeat(60));
        Some(run_ablation(&train_examples, &val_examples)?)
    } else {
        println!("⏭️  Skipping ablation study");
        None
    };

    // 2. Repository-Isolated Evaluation
    println!("\n{}", "=".repeat(60));
    println!("🏗️  STEP 2: Repository-Isolated Evaluation");
    println!("{}", "=".repeat(60));
    let repository_isolated = run_repository_isolated(&train_examples, &test_examples)?;

    // 3. Temporal Evaluation
    let temporal = if !args.skip_temporal {
        println!("\n{}", "=".repeat(60));
        println!("⏰ STEP 3: Temporal Evaluation");
        println!("{}", "=".repeat(60));
        run_temporal(
            &train_examples,
            &test_examples,
            args.temporal_windows,
            args.min_window_examples,
        )?
    } else {
        println!("\n⏭️  Skipping temporal evaluation");
        None
    };

    // 3.5 Per-Language Benchmark
    println!("\n{}", "=".repeat(60));
    println!("🌍 STEP 3.5: Per-Language Benchmark");
    println!("{}", "=".repeat(60));
    let per_language = run_per_language_benchmark(&train_examples, &test_examples)?;

    // 3.6 Leakage Tests
    println!("\n{}", "=".repeat(60));
    println!("🔒 STEP 3.6: Leakage Tests");
    println!("{}", "=".repeat(60));
    let leakage = run_leakage_tests(&train_examples, &test_examples);

    // 3.7 Unseen Repository Benchmark
    println!("\n{}", "=".repeat(60));
    println!("🔮 STEP 3.7: Unseen Repository Benchmark");
    println!("{}", "=".repeat(60));
    let unseen_repo = run_unseen_repo_benchmark(&train_examples, &test_examples);

    // 4. Model Comparison
    println!("\n{}", "=".repeat(60));
    println!("⚖️  STEP 4: Model Comparison (Static vs ML vs Static+ML)");
    println!("{}", "=".repeat(60));
    let model_comparison = run_model_comparison(&train_examples, &val_examples, &test_examples)?;

    // 5. Calibration Metrics
    println!("\n{}", "=".repeat(60));
    println!("🎯 STEP 5: Calibration Metrics");
    println!("{}", "=".repeat(60));
    let calibration = run_calibration_metrics(&train_examples, &val_examples)?;

    // Generate summary
    let summary = generate_summary(
        &ablation,
        &repository_isolated,
        &temporal,
        &model_comparison,
        &calibration,
    );

    // Save results
    let results = Phase2Results {
        ablation,
        repository_isolated: Some(repository_isolated),
        temporal,
        model_comparison,
        calibration,
        summary,
        reproducibility: ReproducibilityInfo {
            command: format!(
                "cargo run --bin phase2_evaluation -- --train-data {} --val-data {} --test-data {}",
                args.train_data.display(),
                args.val_data.display(),
                args.test_data.display()
            ),
            train_data: args.train_data.display().to_string(),
            val_data: args.val_data.display().to_string(),
            test_data: args.test_data.display().to_string(),
            feature_schema_version: 1,
            timestamp: chrono::Utc::now().timestamp(),
            seed: args.seed,
        },
        per_language,
        leakage,
        unseen_repo,
    };

    let results_path = args.output_dir.join("phase2_results.json");
    std::fs::write(&results_path, serde_json::to_string_pretty(&results)?)?;
    println!("\n📁 Results saved to: {:?}", results_path);

    // Generate markdown report
    generate_report(&results, &args.output_dir)?;

    Ok(())
}

// ============ STEP 1: ABLATION STUDY ============

fn run_ablation(
    train_examples: &[TrainingExample],
    val_examples: &[TrainingExample],
) -> Result<Vec<AblationResult>> {
    let mut results = Vec::new();

    let all_features: Vec<usize> = (0..FEATURE_SCHEMA.feature_count()).collect();

    let mut feature_sets: Vec<(String, Vec<usize>)> = vec![
        (
            "Graph Only".to_string(),
            get_category_features(&FeatureCategory::Graph),
        ),
        (
            "Signature Only".to_string(),
            get_category_features(&FeatureCategory::Signature),
        ),
        (
            "Complexity Only".to_string(),
            get_category_features(&FeatureCategory::Complexity),
        ),
        (
            "Name Only".to_string(),
            get_category_features(&FeatureCategory::Name),
        ),
        (
            "File Context Only".to_string(),
            get_category_features(&FeatureCategory::File),
        ),
        (
            "Type Context Only".to_string(),
            get_category_features(&FeatureCategory::Type),
        ),
        (
            "Language Only".to_string(),
            get_category_features(&FeatureCategory::Language),
        ),
        (
            "Framework Only".to_string(),
            get_category_features(&FeatureCategory::Framework),
        ),
        (
            "Decorator Only".to_string(),
            get_category_features(&FeatureCategory::Decorator),
        ),
        (
            "Dynamic Only".to_string(),
            get_category_features(&FeatureCategory::Dynamic),
        ),
        ("Graph + Signature".to_string(), {
            let mut v = get_category_features(&FeatureCategory::Graph);
            v.extend(get_category_features(&FeatureCategory::Signature));
            v
        }),
        ("Graph + Signature + Complexity".to_string(), {
            let mut v = get_category_features(&FeatureCategory::Graph);
            v.extend(get_category_features(&FeatureCategory::Signature));
            v.extend(get_category_features(&FeatureCategory::Complexity));
            v
        }),
        ("All Features".to_string(), all_features.clone()),
    ];

    // Leave-one-out variants
    let categories = vec![
        ("Graph", FeatureCategory::Graph),
        ("Signature", FeatureCategory::Signature),
        ("Complexity", FeatureCategory::Complexity),
        ("Name", FeatureCategory::Name),
        ("File", FeatureCategory::File),
        ("Type", FeatureCategory::Type),
        ("Language", FeatureCategory::Language),
        ("Framework", FeatureCategory::Framework),
        ("Decorator", FeatureCategory::Decorator),
        ("Dynamic", FeatureCategory::Dynamic),
    ];

    for (name, category) in &categories {
        let excluded: std::collections::HashSet<usize> =
            get_category_features(category).into_iter().collect();
        let leave_one_out: Vec<usize> = all_features
            .iter()
            .filter(|&&i| !excluded.contains(&i))
            .cloned()
            .collect();

        feature_sets.push((format!("All minus {}", name), leave_one_out));
    }

    for (name, feature_indices) in &feature_sets {
        println!("\n🧪 Testing: {}", name);
        println!("   Features: {}", feature_indices.len());

        let start = Instant::now();
        let result = train_and_evaluate_subset(train_examples, val_examples, feature_indices, name);
        let elapsed = start.elapsed().as_millis() as u64;

        println!("   Accuracy: {:.1}%", result.accuracy * 100.0);
        println!("   F1: {:.1}%", result.f1 * 100.0);
        println!("   Time: {}ms", elapsed);

        let mut result = result;
        result.train_time_ms = elapsed;
        results.push(result);
    }

    for (name, feature_indices) in &feature_sets {
        println!("\n🧪 Testing: {}", name);
        println!("   Features: {}", feature_indices.len());

        let start = Instant::now();
        let result = train_and_evaluate_subset(train_examples, val_examples, feature_indices, name);
        let elapsed = start.elapsed().as_millis() as u64;

        println!("   Accuracy: {:.1}%", result.accuracy * 100.0);
        println!("   F1: {:.1}%", result.f1 * 100.0);
        println!("   Time: {}ms", elapsed);

        let mut result = result;
        result.train_time_ms = elapsed;
        results.push(result);
    }

    Ok(results)
}

fn get_category_features(category: &FeatureCategory) -> Vec<usize> {
    FEATURE_SCHEMA
        .get_by_category(category)
        .iter()
        .map(|f| f.index)
        .collect()
}

fn train_and_evaluate_subset(
    train_examples: &[TrainingExample],
    val_examples: &[TrainingExample],
    feature_indices: &[usize],
    name: &str,
) -> AblationResult {
    // Extract subset features for training
    let train_features: Vec<Vec<f64>> = train_examples
        .iter()
        .filter(|e| e.label != TrainingLabel::Unknown)
        .map(|e| {
            let full = e.features.to_feature_vector();
            feature_indices.iter().map(|&i| full[i]).collect()
        })
        .collect();

    let train_labels: Vec<f64> = train_examples
        .iter()
        .filter(|e| e.label != TrainingLabel::Unknown)
        .map(|e| match e.label {
            TrainingLabel::Alive => 1.0,
            TrainingLabel::Dead => 0.0,
            TrainingLabel::Unknown => 0.5,
        })
        .collect();

    // Train model on SUBSET features (manual training loop)
    let mut classifier = LinearClassifier::new(feature_indices.len())
        .with_learning_rate(0.01)
        .with_epochs(50);

    // Manual training loop with subset features
    for _epoch in 0..50 {
        for (features, &label) in train_features.iter().zip(train_labels.iter()) {
            let dot: f64 = features
                .iter()
                .zip(&classifier.weights)
                .map(|(f, w)| f * w)
                .sum();
            let z = (dot + classifier.bias).clamp(-20.0, 20.0);
            let prediction = 1.0 / (1.0 + (-z).exp());
            let error = prediction - label;

            for (i, &feature) in features.iter().enumerate() {
                if i < classifier.weights.len() {
                    classifier.weights[i] -= classifier.learning_rate * error * feature;
                }
            }
            classifier.bias -= classifier.learning_rate * error;
        }
    }

    // Evaluate on validation set with subset features
    let metrics = evaluate_subset_classifier(&classifier, val_examples, feature_indices);

    AblationResult {
        feature_set: name.to_string(),
        feature_count: feature_indices.len(),
        accuracy: metrics.accuracy,
        precision: metrics.precision,
        recall: metrics.recall,
        f1: metrics.f1,
        train_time_ms: 0,
    }
}

fn run_unseen_repo_benchmark(
    train_examples: &[TrainingExample],
    test_examples: &[TrainingExample],
) -> Option<UnseenRepoResult> {
    println!("\n🔮 Unseen Repository Benchmark");
    println!("===============================");

    let train_repos: Vec<String> = train_examples
        .iter()
        .filter_map(|e| e.repository_id.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let test_repos: Vec<String> = test_examples
        .iter()
        .filter_map(|e| e.repository_id.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    if train_repos.is_empty() || test_repos.is_empty() {
        println!("   ⚠️  Missing repository_id in data, skipping");
        return None;
    }

    let unseen_repos: Vec<String> = test_repos
        .iter()
        .filter(|r| !train_repos.contains(r))
        .cloned()
        .collect();

    if unseen_repos.is_empty() {
        println!("   ⚠️  No unseen repositories found (all test repos are in training)");
        return None;
    }

    println!("   Training repos: {:?}", train_repos);
    println!("   Unseen test repos: {:?}", unseen_repos);

    let unseen_test: Vec<TrainingExample> = test_examples
        .iter()
        .filter(|e| {
            e.repository_id
                .as_ref()
                .map(|r| unseen_repos.contains(r))
                .unwrap_or(false)
        })
        .cloned()
        .collect();

    if unseen_test.is_empty() {
        println!("   ⚠️  No test examples from unseen repos");
        return None;
    }

    let mut classifier = DeadCodeClassifier::new();
    if classifier.train(train_examples).is_err() {
        println!("   ⚠️  Training failed");
        return None;
    }

    let metrics = evaluate_classifier_full(&classifier, &unseen_test);

    println!("   Examples: {}", unseen_test.len());
    println!("   F1: {:.1}%", metrics.f1 * 100.0);
    println!("   Precision: {:.1}%", metrics.precision * 100.0);

    Some(UnseenRepoResult {
        train_repos,
        test_repos: unseen_repos,
        accuracy: metrics.accuracy,
        precision: metrics.precision,
        recall: metrics.recall,
        f1: metrics.f1,
        fpr: metrics.fpr,
        fnr: metrics.fnr,
        specificity: metrics.specificity,
    })
}

fn run_leakage_tests(
    train_examples: &[TrainingExample],
    test_examples: &[TrainingExample],
) -> LeakageReport {
    println!("\n🔒 Leakage Tests");
    println!("=================");

    let mut tests = Vec::new();

    // 1. Repository identity leakage
    let train_repos: std::collections::HashSet<String> = train_examples
        .iter()
        .filter_map(|e| e.repository_id.clone())
        .collect();
    let test_repos: std::collections::HashSet<String> = test_examples
        .iter()
        .filter_map(|e| e.repository_id.clone())
        .collect();
    let repo_overlap: Vec<String> = train_repos.intersection(&test_repos).cloned().collect();

    tests.push(LeakageTestResult {
        test_name: "Repository Identity".to_string(),
        leaked: !repo_overlap.is_empty(),
        details: if repo_overlap.is_empty() {
            "No repository overlap".to_string()
        } else {
            format!("Repositories in both train and test: {:?}", repo_overlap)
        },
        severity: if repo_overlap.is_empty() {
            "None".to_string()
        } else {
            "CRITICAL".to_string()
        },
    });

    // 2. File path leakage
    let train_files: std::collections::HashSet<String> =
        train_examples.iter().map(|e| e.file.clone()).collect();
    let test_files: std::collections::HashSet<String> =
        test_examples.iter().map(|e| e.file.clone()).collect();
    let file_overlap: Vec<String> = train_files
        .intersection(&test_files)
        .cloned()
        .take(10)
        .collect();

    tests.push(LeakageTestResult {
        test_name: "File Path".to_string(),
        leaked: !file_overlap.is_empty(),
        details: if file_overlap.is_empty() {
            "No file path overlap".to_string()
        } else {
            format!("Files in both train and test: {:?}...", file_overlap)
        },
        severity: if file_overlap.is_empty() {
            "None".to_string()
        } else {
            "HIGH".to_string()
        },
    });

    // 3. Symbol name leakage
    let train_symbols: std::collections::HashSet<String> =
        train_examples.iter().map(|e| e.full_path.clone()).collect();
    let test_symbols: std::collections::HashSet<String> =
        test_examples.iter().map(|e| e.full_path.clone()).collect();
    let symbol_overlap: Vec<String> = train_symbols
        .intersection(&test_symbols)
        .cloned()
        .take(10)
        .collect();

    tests.push(LeakageTestResult {
        test_name: "Symbol Name".to_string(),
        leaked: !symbol_overlap.is_empty(),
        details: if symbol_overlap.is_empty() {
            "No symbol overlap".to_string()
        } else {
            format!("Symbols in both train and test: {:?}...", symbol_overlap)
        },
        severity: if symbol_overlap.is_empty() {
            "None".to_string()
        } else {
            "HIGH".to_string()
        },
    });

    // 4. Duplicate function detection
    let mut duplicate_pairs = 0;
    for train_ex in train_examples.iter().take(100) {
        for test_ex in test_examples.iter().take(100) {
            if train_ex.features.body_hash == test_ex.features.body_hash
                && !train_ex.features.body_hash.is_empty()
            {
                duplicate_pairs += 1;
            }
        }
    }

    tests.push(LeakageTestResult {
        test_name: "Duplicate Functions".to_string(),
        leaked: duplicate_pairs > 0,
        details: if duplicate_pairs == 0 {
            "No duplicate function bodies found".to_string()
        } else {
            format!("Found {} duplicate function pairs", duplicate_pairs)
        },
        severity: if duplicate_pairs == 0 {
            "None".to_string()
        } else {
            "MEDIUM".to_string()
        },
    });

    // 5. Generated code leakage
    let train_generated: std::collections::HashSet<String> = train_examples
        .iter()
        .filter(|e| e.features.is_generated)
        .map(|e| e.file.clone())
        .collect();
    let test_generated: std::collections::HashSet<String> = test_examples
        .iter()
        .filter(|e| e.features.is_generated)
        .map(|e| e.file.clone())
        .collect();
    let gen_overlap: Vec<String> = train_generated
        .intersection(&test_generated)
        .cloned()
        .collect();

    tests.push(LeakageTestResult {
        test_name: "Generated Code".to_string(),
        leaked: !gen_overlap.is_empty(),
        details: if gen_overlap.is_empty() {
            "No generated code overlap".to_string()
        } else {
            format!("Generated files in both: {:?}", gen_overlap)
        },
        severity: if gen_overlap.is_empty() {
            "None".to_string()
        } else {
            "MEDIUM".to_string()
        },
    });

    let total_leaks = tests.iter().filter(|t| t.leaked).count();
    let passed = total_leaks == 0;

    println!("   Total leaks detected: {}", total_leaks);
    for test in &tests {
        if test.leaked {
            println!("   🔴 {}: LEAKED - {}", test.test_name, test.details);
        } else {
            println!("   ✅ {}: Clean", test.test_name);
        }
    }

    LeakageReport {
        tests,
        total_leaks,
        passed,
    }
}

fn run_per_language_benchmark(
    train_examples: &[TrainingExample],
    test_examples: &[TrainingExample],
) -> Result<Vec<LanguageMetrics>> {
    println!("\n🌍 Per-Language Benchmark");
    println!("=========================");

    let mut results = Vec::new();
    let languages = [
        "rust",
        "python",
        "javascript",
        "typescript",
        "go",
        "java",
        "dart",
        "php",
        "cpp",
        "csharp",
    ];

    for language in languages {
        let train_lang: Vec<TrainingExample> = train_examples
            .iter()
            .filter(|e| e.language == language)
            .cloned()
            .collect();

        let test_lang: Vec<TrainingExample> = test_examples
            .iter()
            .filter(|e| e.language == language)
            .cloned()
            .collect();

        if test_lang.is_empty() {
            println!("   {}: No test examples, skipping", language);
            continue;
        }

        println!("\n   Testing: {}", language);
        println!(
            "   Train: {} examples, Test: {} examples",
            train_lang.len(),
            test_lang.len()
        );

        if train_lang.is_empty() {
            println!("   ⚠️  No training examples for {}, skipping", language);
            continue;
        }

        let mut classifier = DeadCodeClassifier::new();
        if classifier.train(&train_lang).is_err() {
            println!("   ⚠️  Training failed for {}, skipping", language);
            continue;
        }

        let metrics = evaluate_classifier_full(&classifier, &test_lang);
        println!(
            "   F1: {:.1}%, Precision: {:.1}%",
            metrics.f1 * 100.0,
            metrics.precision * 100.0
        );

        results.push(LanguageMetrics {
            language: language.to_string(),
            examples: test_lang.len(),
            accuracy: metrics.accuracy,
            precision: metrics.precision,
            recall: metrics.recall,
            f1: metrics.f1,
            fpr: metrics.fpr,
            fnr: metrics.fnr,
            specificity: metrics.specificity,
        });
    }

    Ok(results)
}

fn evaluate_subset_classifier(
    classifier: &LinearClassifier,
    examples: &[TrainingExample],
    feature_indices: &[usize],
) -> ModelMetrics {
    let mut tp = 0;
    let mut tn = 0;
    let mut fp = 0;
    let mut fn_ = 0;

    for example in examples {
        if example.label == TrainingLabel::Unknown {
            continue;
        }

        let full = example.features.to_feature_vector();
        let subset: Vec<f64> = feature_indices.iter().map(|&i| full[i]).collect();
        let pred = classifier.predict(&subset);
        let pred_label = if pred >= 0.5 {
            TrainingLabel::Dead
        } else {
            TrainingLabel::Alive
        };

        match (pred_label, &example.label) {
            (TrainingLabel::Dead, TrainingLabel::Dead) => tp += 1,
            (TrainingLabel::Alive, TrainingLabel::Alive) => tn += 1,
            (TrainingLabel::Alive, TrainingLabel::Dead) => fn_ += 1,
            (TrainingLabel::Dead, TrainingLabel::Alive) => fp += 1,
            _ => {}
        }
    }

    compute_metrics_from_confusion(tp, tn, fp, fn_)
}

// ============ STEP 2: REPOSITORY-ISOLATED EVALUATION ============

fn run_repository_isolated(
    train_examples: &[TrainingExample],
    test_examples: &[TrainingExample],
) -> Result<RepoIsolationResult> {
    // Group examples by repository
    let mut train_by_repo: HashMap<String, Vec<TrainingExample>> = HashMap::new();
    let mut test_by_repo: HashMap<String, Vec<TrainingExample>> = HashMap::new();

    for example in train_examples {
        if let Some(repo) = &example.repository_id {
            train_by_repo
                .entry(repo.clone())
                .or_default()
                .push(example.clone());
        }
    }

    for example in test_examples {
        if let Some(repo) = &example.repository_id {
            test_by_repo
                .entry(repo.clone())
                .or_default()
                .push(example.clone());
        }
    }

    // Check for leakage (same repo in both train and test)
    let mut leakage_detected = false;
    for repo in train_by_repo.keys() {
        if test_by_repo.contains_key(repo) {
            leakage_detected = true;
            println!(
                "   ⚠️  WARNING: Repository '{}' appears in both train and test!",
                repo
            );
        }
    }

    // Train on ALL training repos, test on EACH test repo separately
    let mut by_repository = Vec::new();

    for (repo, test_repo_examples) in &test_by_repo {
        println!("\n🏗️  Evaluating on repository: {}", repo);

        // Train on all training data
        let mut classifier = DeadCodeClassifier::new();
        let train_result = classifier.train(train_examples);

        if train_result.is_err() {
            println!("   ⚠️  Training failed, skipping");
            continue;
        }

        // Evaluate on this repo only
        let metrics = evaluate_classifier_full(&classifier, test_repo_examples);

        println!("   Examples: {}", test_repo_examples.len());
        println!("   Accuracy: {:.1}%", metrics.accuracy * 100.0);
        println!("   F1: {:.1}%", metrics.f1 * 100.0);

        by_repository.push(RepoMetrics {
            repository: repo.clone(),
            train_examples: train_examples.len(),
            test_examples: test_repo_examples.len(),
            accuracy: metrics.accuracy,
            precision: metrics.precision,
            recall: metrics.recall,
            f1: metrics.f1,
        });
    }

    // Calculate averages
    let avg_accuracy =
        by_repository.iter().map(|r| r.accuracy).sum::<f64>() / by_repository.len() as f64;
    let avg_f1 = by_repository.iter().map(|r| r.f1).sum::<f64>() / by_repository.len() as f64;

    let std_accuracy = calculate_std(&by_repository.iter().map(|r| r.accuracy).collect::<Vec<_>>());
    let std_f1 = calculate_std(&by_repository.iter().map(|r| r.f1).collect::<Vec<_>>());

    Ok(RepoIsolationResult {
        by_repository,
        average_accuracy: avg_accuracy,
        average_f1: avg_f1,
        std_accuracy,
        std_f1,
        leakage_detected,
    })
}

// ============ STEP 3: TEMPORAL EVALUATION ============

fn run_temporal(
    train_examples: &[TrainingExample],
    test_examples: &[TrainingExample],
    windows: usize,
    min_examples: usize,
) -> Result<Option<Vec<TemporalWindowResult>>> {
    // Combine all examples and sort by timestamp
    let mut all_examples = Vec::new();
    all_examples.extend(train_examples.iter().cloned());
    all_examples.extend(test_examples.iter().cloned());

    // Extract timestamps
    let mut with_time: Vec<(TrainingExample, i64)> = all_examples
        .into_iter()
        .filter_map(|e| {
            let time = if let Some(ts) = &e.created_at {
                Some(*ts)
            } else if let Some(hash) = &e.commit_hash {
                parse_timestamp_from_hash(hash)
            } else {
                None
            };
            time.map(|t| (e, t))
        })
        .collect();

    if with_time.is_empty() {
        println!("   ⚠️  No timestamp data available for temporal evaluation");
        return Ok(None);
    }

    // Sort by time
    with_time.sort_by(|a, b| a.1.cmp(&b.1));

    let total = with_time.len();
    if total < min_examples * 2 {
        println!("   ⚠️  Not enough timestamped examples for temporal evaluation");
        return Ok(None);
    }

    let window_size = total / windows;
    if window_size < min_examples {
        println!("   ⚠️  Not enough examples per temporal window");
        return Ok(None);
    }

    let mut results = Vec::new();
    let first_f1: Option<f64> = None;

    for i in 0..windows {
        let test_start = i * window_size;
        let test_end = if i == windows - 1 {
            total
        } else {
            (i + 1) * window_size
        };

        // Train on all data before this window
        let train_window: Vec<TrainingExample> = with_time[..test_start]
            .iter()
            .map(|(e, _)| e.clone())
            .collect();

        let test_window: Vec<TrainingExample> = with_time[test_start..test_end]
            .iter()
            .map(|(e, _)| e.clone())
            .collect();

        if train_window.len() < min_examples {
            continue;
        }

        // Train
        let mut classifier = DeadCodeClassifier::new();
        if classifier.train(&train_window).is_err() {
            continue;
        }

        // Evaluate
        let metrics = evaluate_classifier_full(&classifier, &test_window);

        let train_start_time = with_time[0].1;
        let train_end_time = with_time[test_start - 1].1;
        let test_start_time = with_time[test_start].1;
        let test_end_time = with_time[test_end - 1].1;

        let first_f1_val = first_f1.unwrap_or(metrics.f1);
        let degradation = first_f1_val - metrics.f1;

        results.push(TemporalWindowResult {
            window: i + 1,
            train_period: format!(
                "{} → {}",
                format_time(train_start_time),
                format_time(train_end_time)
            ),
            test_period: format!(
                "{} → {}",
                format_time(test_start_time),
                format_time(test_end_time)
            ),
            train_examples: train_window.len(),
            test_examples: test_window.len(),
            accuracy: metrics.accuracy,
            precision: metrics.precision,
            recall: metrics.recall,
            f1: metrics.f1,
            degradation_from_first: degradation,
        });
    }

    Ok(Some(results))
}

// ============ STEP 4: MODEL COMPARISON ============

fn run_model_comparison(
    train_examples: &[TrainingExample],
    _val_examples: &[TrainingExample],
    test_examples: &[TrainingExample],
) -> Result<ModelComparisonResult> {
    // 1. Static only (heuristic baseline)
    println!("\n⚖️  Static Only (Heuristic)");
    let static_metrics = evaluate_static_only(test_examples);

    // 2. ML only
    println!("\n⚖️  ML Only");
    let mut ml_classifier = DeadCodeClassifier::new();
    ml_classifier
        .train(train_examples)
        .map_err(|e| code_intelligence::error::err::model(e))?;
    let ml_metrics = evaluate_classifier_full(&ml_classifier, test_examples);

    // 3. Static + ML combined
    println!("\n⚖️  Static + ML Combined");
    let combined_metrics = evaluate_static_plus_ml(&ml_classifier, test_examples);

    Ok(ModelComparisonResult {
        static_only: static_metrics,
        ml_only: ml_metrics,
        static_plus_ml: combined_metrics,
    })
}

fn evaluate_static_only(examples: &[TrainingExample]) -> ModelMetrics {
    let mut tp = 0;
    let mut tn = 0;
    let mut fp = 0;
    let mut fn_ = 0;

    for example in examples {
        if example.label == TrainingLabel::Unknown {
            continue;
        }

        // Static heuristic: dead if fan_in == 0 and not public
        let is_dead = example.features.fan_in == 0 && !example.features.is_public;
        let pred_label = if is_dead {
            TrainingLabel::Dead
        } else {
            TrainingLabel::Alive
        };

        match (pred_label, &example.label) {
            (TrainingLabel::Dead, TrainingLabel::Dead) => tp += 1,
            (TrainingLabel::Alive, TrainingLabel::Alive) => tn += 1,
            (TrainingLabel::Alive, TrainingLabel::Dead) => fn_ += 1,
            (TrainingLabel::Dead, TrainingLabel::Alive) => fp += 1,
            _ => {}
        }
    }

    compute_metrics_from_confusion(tp, tn, fp, fn_)
}

fn evaluate_static_plus_ml(
    classifier: &DeadCodeClassifier,
    examples: &[TrainingExample],
) -> ModelMetrics {
    let mut tp = 0;
    let mut tn = 0;
    let mut fp = 0;
    let mut fn_ = 0;

    for example in examples {
        if example.label == TrainingLabel::Unknown {
            continue;
        }

        // Combined: ML probability + static heuristic
        let ml_dead_prob = classifier.predict_dead_probability(example);
        let static_dead = example.features.fan_in == 0 && !example.features.is_public;

        // Weighted combination (60% ML, 40% static)
        let combined_score = if static_dead {
            ml_dead_prob * 0.6 + 0.4
        } else {
            ml_dead_prob * 0.6
        };

        let pred_label = if combined_score >= 0.5 {
            TrainingLabel::Dead
        } else {
            TrainingLabel::Alive
        };

        match (pred_label, &example.label) {
            (TrainingLabel::Dead, TrainingLabel::Dead) => tp += 1,
            (TrainingLabel::Alive, TrainingLabel::Alive) => tn += 1,
            (TrainingLabel::Alive, TrainingLabel::Dead) => fn_ += 1,
            (TrainingLabel::Dead, TrainingLabel::Alive) => fp += 1,
            _ => {}
        }
    }

    compute_metrics_from_confusion(tp, tn, fp, fn_)
}

// ============ STEP 5: CALIBRATION METRICS ============

fn run_calibration_metrics(
    train_examples: &[TrainingExample],
    val_examples: &[TrainingExample],
) -> Result<CalibrationMetrics> {
    println!("\n🎯 Computing calibration metrics...");

    // Train model
    let mut classifier = DeadCodeClassifier::new();
    classifier
        .train(train_examples)
        .map_err(|e| code_intelligence::error::err::model(e))?;

    // Collect predictions and actual labels
    let mut predictions = Vec::new();
    for example in val_examples {
        if example.label == TrainingLabel::Unknown {
            continue;
        }

        let dead_prob = classifier.predict_dead_probability(example);
        let actual = match example.label {
            TrainingLabel::Dead => 1.0,
            TrainingLabel::Alive => 0.0,
            TrainingLabel::Unknown => continue,
        };

        predictions.push((dead_prob, actual));
    }

    if predictions.is_empty() {
        return Ok(CalibrationMetrics {
            expected_calibration_error: 0.0,
            maximum_calibration_error: 0.0,
            brier_score: 0.0,
            log_loss: 0.0,
            reliability_curve: Vec::new(),
        });
    }

    // Brier score
    let brier_score = predictions
        .iter()
        .map(|(pred, actual)| (pred - actual).powi(2))
        .sum::<f64>()
        / predictions.len() as f64;

    // Log loss
    let log_loss = predictions
        .iter()
        .map(|(pred, actual)| {
            let p = pred.clamp(1e-7, 1.0 - 1e-7);
            -actual * p.ln() - (1.0 - actual) * (1.0 - p).ln()
        })
        .sum::<f64>()
        / predictions.len() as f64;

    // Reliability curve (10 bins)
    let mut bins = vec![(0.0, 0.0, 0usize); 10];
    for (pred, actual) in &predictions {
        let bin_idx = ((pred * 10.0) as usize).min(9);
        bins[bin_idx].0 += actual;
        bins[bin_idx].1 += pred;
        bins[bin_idx].2 += 1;
    }

    let mut reliability_curve = Vec::new();
    let mut ece = 0.0;
    let mut mce: f64 = 0.0;

    for (i, (actual_sum, pred_sum, count)) in bins.iter().enumerate() {
        if *count > 0 {
            let expected_accuracy = pred_sum / *count as f64;
            let observed_accuracy = actual_sum / *count as f64;
            let gap = (expected_accuracy - observed_accuracy).abs();

            ece += gap * (*count as f64 / predictions.len() as f64);
            mce = mce.max(gap);

            reliability_curve.push(ReliabilityBin {
                confidence_bin: format!("{:.1}-{:.1}", i as f64 * 0.1, (i + 1) as f64 * 0.1),
                expected_accuracy,
                observed_accuracy,
                count: *count,
            });
        }
    }

    Ok(CalibrationMetrics {
        expected_calibration_error: ece,
        maximum_calibration_error: mce,
        brier_score,
        log_loss,
        reliability_curve,
    })
}

// ============ HELPER FUNCTIONS ============

fn load_examples(path: &PathBuf) -> Result<Vec<TrainingExample>> {
    let data = std::fs::read_to_string(path)?;
    let examples: Vec<TrainingExample> = serde_json::from_str(&data)?;
    Ok(examples)
}

fn evaluate_classifier_full(
    classifier: &DeadCodeClassifier,
    examples: &[TrainingExample],
) -> ModelMetrics {
    let mut tp = 0;
    let mut tn = 0;
    let mut fp = 0;
    let mut fn_ = 0;

    for example in examples {
        if example.label == TrainingLabel::Unknown {
            continue;
        }

        let pred = classifier.predict(example);

        match (pred, &example.label) {
            (TrainingLabel::Dead, TrainingLabel::Dead) => tp += 1,
            (TrainingLabel::Alive, TrainingLabel::Alive) => tn += 1,
            (TrainingLabel::Alive, TrainingLabel::Dead) => fn_ += 1,
            (TrainingLabel::Dead, TrainingLabel::Alive) => fp += 1,
            _ => {}
        }
    }

    compute_metrics_from_confusion(tp, tn, fp, fn_)
}

fn compute_metrics_from_confusion(tp: usize, tn: usize, fp: usize, fn_: usize) -> ModelMetrics {
    let total = tp + tn + fp + fn_;
    let accuracy = if total > 0 {
        (tp + tn) as f64 / total as f64
    } else {
        0.0
    };
    let precision = if tp + fp > 0 {
        tp as f64 / (tp + fp) as f64
    } else {
        0.0
    };
    let recall = if tp + fn_ > 0 {
        tp as f64 / (tp + fn_) as f64
    } else {
        0.0
    };
    let f1 = if precision + recall > 0.0 {
        2.0 * precision * recall / (precision + recall)
    } else {
        0.0
    };
    let fpr = if fp + tn > 0 {
        fp as f64 / (fp + tn) as f64
    } else {
        0.0
    };
    let fnr = if fn_ + tp > 0 {
        fn_ as f64 / (fn_ + tp) as f64
    } else {
        0.0
    };
    let specificity = if tn + fp > 0 {
        tn as f64 / (tn + fp) as f64
    } else {
        0.0
    };

    ModelMetrics {
        accuracy,
        precision,
        recall,
        f1,
        fpr,
        fnr,
        specificity,
        pr_auc: 0.0,
        roc_auc: 0.0,
    }
}

fn calculate_std(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    variance.sqrt()
}

fn parse_timestamp_from_hash(hash: &str) -> Option<i64> {
    use std::process::Command;
    if hash.chars().all(|c| c.is_ascii_hexdigit()) && hash.len() >= 7 {
        let output = Command::new("git")
            .args(["show", "-s", "--format=%ct", hash])
            .output()
            .ok()?;
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse::<i64>()
                .ok();
        }
    }
    None
}

fn format_time(ts: i64) -> String {
    if let Some(dt) = chrono::DateTime::from_timestamp(ts, 0) {
        dt.format("%Y-%m-%d").to_string()
    } else {
        ts.to_string()
    }
}

fn generate_summary(
    ablation: &Option<Vec<AblationResult>>,
    repo_isolated: &RepoIsolationResult,
    temporal: &Option<Vec<TemporalWindowResult>>,
    model_comparison: &ModelComparisonResult,
    calibration: &CalibrationMetrics,
) -> SummaryReport {
    let mut recommendations = Vec::new();

    // Best ablation config
    let best_ablation = ablation
        .as_ref()
        .and_then(|results| results.iter().max_by(|a, b| a.f1.total_cmp(&b.f1)));

    // Best model
    let best_model = if model_comparison.static_plus_ml.f1 > model_comparison.ml_only.f1
        && model_comparison.static_plus_ml.f1 > model_comparison.static_only.f1
    {
        "Static + ML".to_string()
    } else if model_comparison.ml_only.f1 > model_comparison.static_only.f1 {
        "ML Only".to_string()
    } else {
        "Static Only".to_string()
    };

    // Calibration quality
    let calibration_quality = if calibration.expected_calibration_error < 0.05 {
        "Excellent".to_string()
    } else if calibration.expected_calibration_error < 0.10 {
        "Good".to_string()
    } else if calibration.expected_calibration_error < 0.15 {
        "Fair".to_string()
    } else {
        "Poor".to_string()
    };

    // Recommendations
    if repo_isolated.std_f1 > 0.10 {
        recommendations.push(
            "High variance across repositories - consider repository-specific normalization"
                .to_string(),
        );
    }

    if let Some(temp_results) = temporal {
        if let Some(last) = temp_results.last() {
            if last.degradation_from_first > 0.05 {
                recommendations
                    .push("Significant temporal degradation - retrain on recent data".to_string());
            }
        }
    }

    if calibration.expected_calibration_error > 0.10 {
        recommendations
            .push("Poor calibration - consider temperature scaling or Platt scaling".to_string());
    }

    if repo_isolated.leakage_detected {
        recommendations.push(
            "⚠️  CRITICAL: Data leakage detected - same repository in train and test!".to_string(),
        );
    }

    SummaryReport {
        best_ablation_config: best_ablation.map(|b| b.feature_set.clone()),
        repository_isolation_score: Some(repo_isolated.average_f1),
        temporal_degradation: temporal
            .as_ref()
            .and_then(|t| t.last().map(|r| r.degradation_from_first)),
        best_model,
        calibration_quality,
        recommendations,
    }
}

fn generate_report(results: &Phase2Results, output_dir: &PathBuf) -> Result<()> {
    let mut markdown = String::new();

    markdown.push_str("# 📊 Phase 2 Evaluation Report\n\n");

    // Summary
    markdown.push_str("## Summary\n\n");
    markdown.push_str(&format!(
        "- **Best model**: {}\n",
        results.summary.best_model
    ));
    markdown.push_str(&format!(
        "- **Calibration quality**: {}\n",
        results.summary.calibration_quality
    ));

    if let Some(best_ablation) = &results.summary.best_ablation_config {
        markdown.push_str(&format!("- **Best ablation config**: {}\n", best_ablation));
    }

    if let Some(iso_score) = results.summary.repository_isolation_score {
        markdown.push_str(&format!(
            "- **Repository isolation F1**: {:.1}%\n",
            iso_score * 100.0
        ));
    }

    if !results.summary.recommendations.is_empty() {
        markdown.push_str("\n### Recommendations\n");
        for rec in &results.summary.recommendations {
            markdown.push_str(&format!("- {}\n", rec));
        }
    }

    // Model comparison
    markdown.push_str("\n## Model Comparison\n\n");
    markdown.push_str("| Model | Accuracy | Precision | Recall | F1 | FPR | FNR | Specificity |\n");
    markdown.push_str("|-------|----------|-----------|--------|----|-----|-----|-------------|\n");
    markdown.push_str(&format!(
        "| Static Only | {:.1}% | {:.1}% | {:.1}% | {:.1}% | {:.1}% | {:.1}% | {:.1}% |\n",
        results.model_comparison.static_only.accuracy * 100.0,
        results.model_comparison.static_only.precision * 100.0,
        results.model_comparison.static_only.recall * 100.0,
        results.model_comparison.static_only.f1 * 100.0,
        results.model_comparison.static_only.fpr * 100.0,
        results.model_comparison.static_only.fnr * 100.0,
        results.model_comparison.static_only.specificity * 100.0
    ));
    markdown.push_str(&format!(
        "| ML Only | {:.1}% | {:.1}% | {:.1}% | {:.1}% | {:.1}% | {:.1}% | {:.1}% |\n",
        results.model_comparison.ml_only.accuracy * 100.0,
        results.model_comparison.ml_only.precision * 100.0,
        results.model_comparison.ml_only.recall * 100.0,
        results.model_comparison.ml_only.f1 * 100.0,
        results.model_comparison.ml_only.fpr * 100.0,
        results.model_comparison.ml_only.fnr * 100.0,
        results.model_comparison.ml_only.specificity * 100.0
    ));
    markdown.push_str(&format!(
        "| Static + ML | {:.1}% | {:.1}% | {:.1}% | {:.1}% | {:.1}% | {:.1}% | {:.1}% |\n",
        results.model_comparison.static_plus_ml.accuracy * 100.0,
        results.model_comparison.static_plus_ml.precision * 100.0,
        results.model_comparison.static_plus_ml.recall * 100.0,
        results.model_comparison.static_plus_ml.f1 * 100.0,
        results.model_comparison.static_plus_ml.fpr * 100.0,
        results.model_comparison.static_plus_ml.fnr * 100.0,
        results.model_comparison.static_plus_ml.specificity * 100.0
    ));

    // Per-language benchmarks
    markdown.push_str("\n## Per-Language Benchmarks\n\n");
    markdown.push_str("| Language | Examples | Accuracy | Precision | Recall | F1 | FPR |\n");
    markdown.push_str("|----------|----------|----------|-----------|--------|----|-----|\n");
    for lang in &results.per_language {
        markdown.push_str(&format!(
            "| {} | {} | {:.1}% | {:.1}% | {:.1}% | {:.1}% | {:.1}% |\n",
            lang.language,
            lang.examples,
            lang.accuracy * 100.0,
            lang.precision * 100.0,
            lang.recall * 100.0,
            lang.f1 * 100.0,
            lang.fpr * 100.0
        ));
    }

    // Leakage report
    markdown.push_str("\n## Leakage Tests\n\n");
    if results.leakage.passed {
        markdown.push_str("✅ No data leakage detected.\n");
    } else {
        markdown.push_str("🔴 Data leakage detected!\n\n");
        markdown.push_str("| Test | Severity | Details |\n");
        markdown.push_str("|------|----------|---------|\n");
        for test in &results.leakage.tests {
            if test.leaked {
                markdown.push_str(&format!(
                    "| {} | {} | {} |\n",
                    test.test_name, test.severity, test.details
                ));
            }
        }
    }

    // Unseen repository benchmark
    if let Some(unseen) = &results.unseen_repo {
        markdown.push_str("\n## Unseen Repository Benchmark\n\n");
        markdown.push_str(&format!(
            "Training repos: {}\n\n",
            unseen.train_repos.join(", ")
        ));
        markdown.push_str(&format!(
            "Test repos (unseen): {}\n\n",
            unseen.test_repos.join(", ")
        ));
        markdown.push_str("| Metric | Value |\n");
        markdown.push_str("|--------|-------|\n");
        markdown.push_str(&format!("| Accuracy | {:.1}% |\n", unseen.accuracy * 100.0));
        markdown.push_str(&format!(
            "| Precision | {:.1}% |\n",
            unseen.precision * 100.0
        ));
        markdown.push_str(&format!("| Recall | {:.1}% |\n", unseen.recall * 100.0));
        markdown.push_str(&format!("| F1 | {:.1}% |\n", unseen.f1 * 100.0));
        markdown.push_str(&format!("| FPR | {:.1}% |\n", unseen.fpr * 100.0));
        markdown.push_str(&format!("| FNR | {:.1}% |\n", unseen.fnr * 100.0));
        markdown.push_str(&format!(
            "| Specificity | {:.1}% |\n",
            unseen.specificity * 100.0
        ));
    }

    // Calibration
    markdown.push_str("\n## Calibration Metrics\n\n");
    markdown.push_str(&format!(
        "- **ECE**: {:.4}\n",
        results.calibration.expected_calibration_error
    ));
    markdown.push_str(&format!(
        "- **MCE**: {:.4}\n",
        results.calibration.maximum_calibration_error
    ));
    markdown.push_str(&format!(
        "- **Brier score**: {:.4}\n",
        results.calibration.brier_score
    ));
    markdown.push_str(&format!(
        "- **Log loss**: {:.4}\n",
        results.calibration.log_loss
    ));

    // Reliability curve
    if !results.calibration.reliability_curve.is_empty() {
        markdown.push_str("\n### Reliability Curve\n\n");
        markdown.push_str("| Confidence | Expected | Observed | Count |\n");
        markdown.push_str("|------------|----------|----------|-------|\n");
        for bin in &results.calibration.reliability_curve {
            markdown.push_str(&format!(
                "| {} | {:.1}% | {:.1}% | {} |\n",
                bin.confidence_bin,
                bin.expected_accuracy * 100.0,
                bin.observed_accuracy * 100.0,
                bin.count
            ));
        }
    }

    markdown.push_str("\n---\n");
    markdown.push_str(&format!(
        "*Report generated on {}*\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));

    let report_path = output_dir.join("phase2_report.md");
    std::fs::write(&report_path, markdown)?;
    println!("📁 Report saved to: {:?}", report_path);

    Ok(())
}
