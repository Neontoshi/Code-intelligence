use code_intelligence::analysis::training_data::TrainingExample;
use code_intelligence::analysis::verdict_source::label_source::LabelSource;
use code_intelligence::error::{err, Result};
use std::collections::HashMap;
use std::path::PathBuf;

fn main() -> Result<()> {
    let training_dir = PathBuf::from("data/raw/jsonl");

    // Create output directory if it doesn't exist
    std::fs::create_dir_all("data")?;

    // Load all examples grouped by repository
    let mut by_repo: HashMap<String, Vec<TrainingExample>> = HashMap::new();

    println!("📊 Loading training data from: {:?}", training_dir);

    for entry in std::fs::read_dir(&training_dir)? {
        let entry = entry?;
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext == "json" || ext == "jsonl" {
            let repo_name = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            println!("   Loading: {}", repo_name);
            let data = std::fs::read_to_string(&path)?;

            let mut examples: Vec<TrainingExample> = if ext == "jsonl" {
                data.lines()
                    .filter(|l| !l.trim().is_empty())
                    .filter_map(|l| {
                        match serde_json::from_str::<TrainingExample>(l) {
                            Ok(ex) => Some(ex),
                            Err(e) => {
                                // ⭐ NEW: Try legacy parsing if modern fails
                                match parse_legacy_example(l) {
                                    Ok(ex) => {
                                        println!("   ✅ Converted legacy example");
                                        Some(ex)
                                    }
                                    Err(_) => {
                                        eprintln!(
                                            "   ⚠️ Skipping malformed line in {}: {}",
                                            repo_name, e
                                        );
                                        None
                                    }
                                }
                            }
                        }
                    })
                    .collect()
            } else {
                match serde_json::from_str::<Vec<TrainingExample>>(&data) {
                    Ok(examples) => examples,
                    Err(e) => {
                        eprintln!("   ⚠️ Failed to parse {} as modern JSON: {}", repo_name, e);
                        // ⭐ NEW: Try legacy parsing
                        parse_legacy_json_file(&data, &repo_name)?
                    }
                }
            };

            // Add repository_id to each example
            for example in &mut examples {
                if example.repository_id.is_none() {
                    example.repository_id = Some(repo_name.clone());
                }
                if example.commit_hash.is_none() {
                    example.commit_hash = Some("unknown".to_string());
                }
                // ⭐ Ensure label_source is set
                if example.label_source == LabelSource::StaticHeuristic {
                    // Already set
                }
            }

            println!("      {} examples", examples.len());
            by_repo.insert(repo_name, examples);
        }
    }

    println!("\n📊 Found {} repositories", by_repo.len());
    let total_examples: usize = by_repo.values().map(|v| v.len()).sum();
    println!("   Total examples: {}", total_examples);

    // ⭐ Deduplicate examples
    println!("\n🔍 Deduplicating examples...");
    let deduped = deduplicate_examples(&by_repo);
    println!("   After dedup: {} examples", deduped.len());

    // Rebuild by_repo from deduped
    let mut by_repo_deduped: HashMap<String, Vec<TrainingExample>> = HashMap::new();
    for example in deduped {
        if let Some(repo) = &example.repository_id {
            by_repo_deduped
                .entry(repo.clone())
                .or_default()
                .push(example);
        }
    }
    by_repo = by_repo_deduped;

    // Split repos into train/validation/test sets
    let repo_names: Vec<String> = by_repo.keys().cloned().collect();

    // Use deterministic shuffle with seed for reproducibility
    use rand::seq::SliceRandom;
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let mut shuffled = repo_names.clone();
    shuffled.shuffle(&mut rng);

    let total = shuffled.len();

    let (train_repos, val_repos, test_repos) = if total == 0 {
        println!("❌ No repositories found!");
        return Ok(());
    } else if total == 1 {
        // Only one repo - use for training
        (&shuffled[0..1], &[][..], &[][..])
    } else if total == 2 {
        // Two repos - one for train, one for val
        (&shuffled[0..1], &shuffled[1..2], &[][..])
    } else {
        // 3+ repos: guarantee at least 1 for val and 1 for test,
        // everything else goes to train. Scales cleanly as repo
        // count grows (unlike the old ceil(0.7*n) formula, which
        // could zero out val or test entirely for small n like 4).
        let test_count = 1;
        let val_count = 1;
        let train_count = total - val_count - test_count;

        (
            &shuffled[0..train_count],
            &shuffled[train_count..train_count + val_count],
            &shuffled[train_count + val_count..],
        )
    };

    println!("\n📊 Splitting {} repositories:", total);
    println!("   Train: {} repos", train_repos.len());
    println!("   Validation: {} repos", val_repos.len());
    println!("   Test: {} repos", test_repos.len());

    // Build datasets with split labels
    let mut train_examples = Vec::new();
    let mut val_examples = Vec::new();
    let mut test_examples = Vec::new();

    for repo in train_repos {
        if let Some(examples) = by_repo.get(repo) {
            let mut cloned = examples.clone();
            for example in &mut cloned {
                example.dataset_split = Some("train".to_string());
                example.label_reason = Some("auto".to_string());
                example.label_version = Some(1);
            }
            train_examples.extend(cloned);
        }
    }
    for repo in val_repos {
        if let Some(examples) = by_repo.get(repo) {
            let mut cloned = examples.clone();
            for example in &mut cloned {
                example.dataset_split = Some("val".to_string());
                example.label_reason = Some("auto".to_string());
                example.label_version = Some(1);
            }
            val_examples.extend(cloned);
        }
    }
    for repo in test_repos {
        if let Some(examples) = by_repo.get(repo) {
            let mut cloned = examples.clone();
            for example in &mut cloned {
                example.dataset_split = Some("test".to_string());
                example.label_reason = Some("auto".to_string());
                example.label_version = Some(1);
            }
            test_examples.extend(cloned);
        }
    }
    // Save datasets
    println!("\n📊 Saving datasets to ./data/");
    std::fs::create_dir_all("data")?;

    std::fs::write(
        "data/train.json",
        serde_json::to_string_pretty(&train_examples)?,
    )?;
    std::fs::write(
        "data/val.json",
        serde_json::to_string_pretty(&val_examples)?,
    )?;
    std::fs::write(
        "data/test.json",
        serde_json::to_string_pretty(&test_examples)?,
    )?;

    // Also save as JSONL for streaming
    let train_jsonl: String = train_examples
        .iter()
        .filter_map(|e| serde_json::to_string(e).ok())
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write("data/train.jsonl", train_jsonl)?;

    println!("\n📊 Dataset split complete:");
    println!(
        "   Train: {} repos, {} examples",
        train_repos.len(),
        train_examples.len()
    );
    println!(
        "   Validation: {} repos, {} examples",
        val_repos.len(),
        val_examples.len()
    );
    println!(
        "   Test: {} repos, {} examples",
        test_repos.len(),
        test_examples.len()
    );

    println!("\n   Repositories:");
    println!("      Train: {:?}", train_repos);
    println!("      Val: {:?}", val_repos);
    println!("      Test: {:?}", test_repos);

    // Show label distribution
    use code_intelligence::analysis::training_data::TrainingLabel;

    let train_alive = train_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Alive)
        .count();
    let train_dead = train_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Dead)
        .count();
    let val_alive = val_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Alive)
        .count();
    let val_dead = val_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Dead)
        .count();
    let test_alive = test_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Alive)
        .count();
    let test_dead = test_examples
        .iter()
        .filter(|e| e.label == TrainingLabel::Dead)
        .count();

    println!("\n   Label Distribution:");
    println!("      Train: Alive={}, Dead={}", train_alive, train_dead);
    println!("      Val:   Alive={}, Dead={}", val_alive, val_dead);
    println!("      Test:  Alive={}, Dead={}", test_alive, test_dead);

    Ok(())
}

fn deduplicate_examples(by_repo: &HashMap<String, Vec<TrainingExample>>) -> Vec<TrainingExample> {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    let mut deduped = Vec::new();

    for examples in by_repo.values() {
        for example in examples {
            // ⭐ Use a LESS aggressive key - only signature hash, not body hash
            let key = format!("{}", example.features.signature_hash);

            // ⭐ Allow some duplicates if they have different function names
            // This preserves more training examples
            if !seen.contains(&key) {
                seen.insert(key);
                deduped.push(example.clone());
            } else {
                // If signature matches, still add if the function name is different
                // This gives us more variety
                let existing = deduped
                    .iter()
                    .find(|e| e.features.signature_hash == example.features.signature_hash);
                if let Some(existing) = existing {
                    if existing.function_name != example.function_name {
                        // Different name, same signature - keep both for variety
                        deduped.push(example.clone());
                    }
                }
            }
        }
    }

    deduped
}

fn parse_legacy_json_file(data: &str, repo_name: &str) -> Result<Vec<TrainingExample>> {
    use serde_json::Value;

    let json: Vec<Value> = serde_json::from_str(data)?;
    let mut examples = Vec::new();

    for item in json {
        if let Ok(ex) = convert_legacy_to_training_example(item, repo_name) {
            examples.push(ex);
        }
    }

    Ok(examples)
}

// ⭐ NEW: Convert legacy JSON to TrainingExample
fn convert_legacy_to_training_example(
    item: serde_json::Value,
    repo_name: &str,
) -> Result<TrainingExample> {
    use code_intelligence::analysis::training_data::{FunctionFeatures, TrainingLabel};

    // Extract fields with defaults
    let function_name = item
        .get("function_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let full_path = item
        .get("full_path")
        .and_then(|v| v.as_str())
        .unwrap_or(&function_name)
        .to_string();
    let file = item
        .get("file")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown.rs")
        .to_string();
    let language = item
        .get("language")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    // Extract label
    let label_str = item
        .get("label")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");
    let label = match label_str {
        "Alive" => TrainingLabel::Alive,
        "Dead" => TrainingLabel::Dead,
        _ => TrainingLabel::Unknown,
    };

    let confidence = item
        .get("confidence")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.5);
    let source = item
        .get("source")
        .and_then(|v| v.as_str())
        .unwrap_or("legacy")
        .to_string();
    let label_reason = item
        .get("label_reason")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let label_version = item
        .get("label_version")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    // Create default features (simplified)
    let features = FunctionFeatures {
        param_count: item
            .get("features")
            .and_then(|f| f.get("param_count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize,
        return_count: item
            .get("features")
            .and_then(|f| f.get("return_count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize,
        is_public: item
            .get("features")
            .and_then(|f| f.get("is_public"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        is_async: item
            .get("features")
            .and_then(|f| f.get("is_async"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        name_length: function_name.len(),
        starts_with_use: function_name.starts_with("use"),
        starts_with_test: function_name.starts_with("test_") || function_name.starts_with("Test"),
        starts_with_bench: function_name.starts_with("bench_")
            || function_name.starts_with("Benchmark"),
        ends_with_test: function_name.ends_with("_test"),
        contains_trait_impl: false,
        signature_hash: String::new(),
        body_hash: String::new(),
        fan_in: 0,
        fan_out: 0,
        complexity: 1.0,
        call_depth: 0,
        is_cycle: false,
        file_extension: file.split('.').last().unwrap_or("").to_string(),
        is_in_test_file: file.contains("/tests/")
            || file.contains("/test/")
            || file.ends_with("_test.rs"),
        is_in_benches: file.contains("/benches/"),
        is_in_meta: file.contains("/.meta/"),
        is_in_examples: file.contains("/examples/"),
        is_generated: file.contains(".gen.") || file.contains("_gen."),
        name_contains_use: function_name.to_lowercase().contains("use"),
        name_contains_test: function_name.to_lowercase().contains("test"),
        name_contains_init: function_name.to_lowercase().contains("init"),
        name_contains_get: function_name.to_lowercase().contains("get"),
        name_contains_set: function_name.to_lowercase().contains("set"),
        name_contains_new: function_name.to_lowercase().contains("new"),
        name_contains_create: function_name.to_lowercase().contains("create"),
        name_contains_build: function_name.to_lowercase().contains("build"),
        name_contains_parse: function_name.to_lowercase().contains("parse"),
        name_contains_validate: function_name.to_lowercase().contains("validate"),
        name_contains_handle: function_name.to_lowercase().contains("handle"),
        name_contains_process: function_name.to_lowercase().contains("process"),
        name_contains_convert: function_name.to_lowercase().contains("convert"),
        name_contains_commit: function_name.to_lowercase().contains("commit"),
        name_contains_reveal: function_name.to_lowercase().contains("reveal"),
        name_contains_submit: function_name.to_lowercase().contains("submit"),
        name_contains_upload: function_name.to_lowercase().contains("upload"),
        name_contains_download: function_name.to_lowercase().contains("download"),
        name_contains_fetch: function_name.to_lowercase().contains("fetch"),
        name_contains_verify: function_name.to_lowercase().contains("verify"),
        name_contains_audit: function_name.to_lowercase().contains("audit"),
        type_name: None,
        type_path: None,
        is_method: false,
        is_trait_impl: false,
        trait_name: None,
        is_associated: false,
    };

    Ok(TrainingExample {
        function_name,
        full_path,
        file,
        language,
        features,
        label,
        confidence,
        source,
        repository_id: Some(repo_name.to_string()),
        commit_hash: Some("unknown".to_string()),
        dataset_split: None,
        label_reason,
        label_version,
        label_source: LabelSource::StaticHeuristic,
        generated_by_model: None,
        verified_by: None,
        created_at: Some(chrono::Utc::now().timestamp()),
    })
}

// ⭐ NEW: Parse legacy single line
fn parse_legacy_example(line: &str) -> Result<TrainingExample> {
    let value: serde_json::Value = serde_json::from_str(line)
        .map_err(|e| err::internal(format!("Failed to parse JSON: {}", e)))?;
    convert_legacy_to_training_example(value, "legacy")
}
