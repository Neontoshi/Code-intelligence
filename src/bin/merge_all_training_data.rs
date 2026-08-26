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
                                // Try legacy parsing if modern fails
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
                        // Try legacy parsing
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
                // Ensure label_source is set
                if example.label_source == LabelSource::StaticHeuristic {}
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
    let repo_names: Vec<String> = by_repo.keys().cloned().collect();

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
            let key = format!("{}", example.features.signature_hash);
            if !seen.contains(&key) {
                seen.insert(key);
                deduped.push(example.clone());
            } else {
                let existing = deduped
                    .iter()
                    .find(|e| e.features.signature_hash == example.features.signature_hash);
                if let Some(existing) = existing {
                    if existing.function_name != example.function_name {
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

// src/bin/merge_all_training_data.rs

/// Convert legacy JSON to TrainingExample
fn convert_legacy_to_training_example(
    item: serde_json::Value,
    repo_name: &str,
) -> Result<TrainingExample> {
    use code_intelligence::analysis::training_data::{FunctionFeatures, TrainingLabel};

    // Extract basic fields
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

    // ⭐ START WITH DEFAULT (all 160+ fields initialized)
    let mut features = FunctionFeatures::default();

    // EXISTING FIELDS - Set from legacy data
    features.param_count = item
        .get("features")
        .and_then(|f| f.get("param_count"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    features.return_count = item
        .get("features")
        .and_then(|f| f.get("return_count"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    features.is_public = item
        .get("features")
        .and_then(|f| f.get("is_public"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    features.is_async = item
        .get("features")
        .and_then(|f| f.get("is_async"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    features.name_length = function_name.len();

    features.starts_with_use = function_name.starts_with("use");
    features.starts_with_test =
        function_name.starts_with("test_") || function_name.starts_with("Test");
    features.starts_with_bench =
        function_name.starts_with("bench_") || function_name.starts_with("Benchmark");
    features.ends_with_test = function_name.ends_with("_test");

    features.contains_trait_impl = item
        .get("features")
        .and_then(|f| f.get("contains_trait_impl"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Hash fields
    features.signature_hash = item
        .get("features")
        .and_then(|f| f.get("signature_hash"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    features.body_hash = item
        .get("features")
        .and_then(|f| f.get("body_hash"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Graph features
    features.fan_in = item
        .get("features")
        .and_then(|f| f.get("fan_in"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    features.fan_out = item
        .get("features")
        .and_then(|f| f.get("fan_out"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    features.complexity = item
        .get("features")
        .and_then(|f| f.get("complexity"))
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);

    features.call_depth = item
        .get("features")
        .and_then(|f| f.get("call_depth"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    features.is_cycle = item
        .get("features")
        .and_then(|f| f.get("is_cycle"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // File context
    features.file_extension = file.split('.').last().unwrap_or("").to_string();

    features.is_in_test_file = file.contains("/tests/")
        || file.contains("/test/")
        || file.ends_with("_test.rs")
        || file.ends_with("_test.go")
        || file.ends_with("_test.py")
        || file.ends_with(".test.ts")
        || file.ends_with(".test.js");

    features.is_in_benches =
        file.contains("/benches/") || file.ends_with("_bench.rs") || file.ends_with("_bench.go");

    features.is_in_meta = file.contains("/.meta/");
    features.is_in_examples = file.contains("/examples/") || file.contains("/example/");
    features.is_generated = file.contains(".gen.")
        || file.contains("_gen.")
        || file.contains(".generated.")
        || file.contains(".pb.go")
        || file.contains("_pb2.py");

    // Name contains patterns - original 21
    let name_lower = function_name.to_lowercase();
    features.name_contains_use = name_lower.contains("use");
    features.name_contains_test = name_lower.contains("test");
    features.name_contains_init = name_lower.contains("init");
    features.name_contains_get = name_lower.contains("get");
    features.name_contains_set = name_lower.contains("set");
    features.name_contains_new = name_lower.contains("new");
    features.name_contains_create = name_lower.contains("create");
    features.name_contains_build = name_lower.contains("build");
    features.name_contains_parse = name_lower.contains("parse");
    features.name_contains_validate = name_lower.contains("validate");
    features.name_contains_handle = name_lower.contains("handle");
    features.name_contains_process = name_lower.contains("process");
    features.name_contains_convert = name_lower.contains("convert");
    features.name_contains_commit = name_lower.contains("commit");
    features.name_contains_reveal = name_lower.contains("reveal");
    features.name_contains_submit = name_lower.contains("submit");
    features.name_contains_upload = name_lower.contains("upload");
    features.name_contains_download = name_lower.contains("download");
    features.name_contains_fetch = name_lower.contains("fetch");
    features.name_contains_verify = name_lower.contains("verify");
    features.name_contains_audit = name_lower.contains("audit");

    // Name contains patterns - new 20+
    features.name_contains_main = name_lower.contains("main");
    features.name_contains_start = name_lower.contains("start");
    features.name_contains_run = name_lower.contains("run");
    features.name_contains_load = name_lower.contains("load");
    features.name_contains_save = name_lower.contains("save");
    features.name_contains_read = name_lower.contains("read");
    features.name_contains_write = name_lower.contains("write");
    features.name_contains_open = name_lower.contains("open");
    features.name_contains_close = name_lower.contains("close");
    features.name_contains_connect = name_lower.contains("connect");
    features.name_contains_send = name_lower.contains("send");
    features.name_contains_receive = name_lower.contains("receive");
    features.name_contains_delete = name_lower.contains("delete");
    features.name_contains_update = name_lower.contains("update");
    features.name_contains_patch = name_lower.contains("patch");
    features.name_contains_put = name_lower.contains("put");
    features.name_contains_post = name_lower.contains("post");
    features.name_contains_list = name_lower.contains("list");
    features.name_contains_find = name_lower.contains("find");
    features.name_contains_search = name_lower.contains("search");
    features.name_contains_filter = name_lower.contains("filter");
    features.name_contains_map = name_lower.contains("map");
    features.name_contains_reduce = name_lower.contains("reduce");
    features.name_contains_clone = name_lower.contains("clone");
    features.name_contains_copy = name_lower.contains("copy");
    features.name_contains_move = name_lower.contains("move");
    features.name_contains_swap = name_lower.contains("swap");
    features.name_contains_sort = name_lower.contains("sort");
    features.name_contains_is = name_lower.contains("is");
    features.name_contains_has = name_lower.contains("has");
    features.name_contains_can = name_lower.contains("can");
    features.name_contains_should = name_lower.contains("should");
    features.name_contains_will = name_lower.contains("will");
    features.name_contains_do = name_lower.contains("do");
    features.name_contains_make = name_lower.contains("make");
    features.name_contains_take = name_lower.contains("take");
    features.name_contains_give = name_lower.contains("give");
    features.name_contains_call = name_lower.contains("call");
    features.name_contains_apply = name_lower.contains("apply");
    features.name_contains_register = name_lower.contains("register");
    features.name_contains_unregister = name_lower.contains("unregister");
    features.name_contains_subscribe = name_lower.contains("subscribe");
    features.name_contains_unsubscribe = name_lower.contains("unsubscribe");

    // Starts with patterns
    features.starts_with_get = function_name.starts_with("get");
    features.starts_with_set = function_name.starts_with("set");
    features.starts_with_is = function_name.starts_with("is");
    features.starts_with_has = function_name.starts_with("has");
    features.starts_with_can = function_name.starts_with("can");
    features.starts_with_should = function_name.starts_with("should");
    features.starts_with_will = function_name.starts_with("will");
    features.starts_with_on = function_name.starts_with("on");
    features.starts_with_handle = function_name.starts_with("handle");
    features.starts_with_process = function_name.starts_with("process");
    features.starts_with_parse = function_name.starts_with("parse");
    features.starts_with_create = function_name.starts_with("create");
    features.starts_with_build = function_name.starts_with("build");
    features.starts_with_make = function_name.starts_with("make");
    features.starts_with_do = function_name.starts_with("do");
    features.starts_with_apply = function_name.starts_with("apply");

    // Ends with patterns
    features.ends_with_handler = function_name.ends_with("handler");
    features.ends_with_processor = function_name.ends_with("processor");
    features.ends_with_service = function_name.ends_with("service");
    features.ends_with_repository = function_name.ends_with("repository");
    features.ends_with_controller = function_name.ends_with("controller");
    features.ends_with_manager = function_name.ends_with("manager");
    features.ends_with_factory = function_name.ends_with("factory");
    features.ends_with_builder = function_name.ends_with("builder");
    features.ends_with_validator = function_name.ends_with("validator");
    features.ends_with_converter = function_name.ends_with("converter");
    features.ends_with_mapper = function_name.ends_with("mapper");
    features.ends_with_filter = function_name.ends_with("filter");
    features.ends_with_loader = function_name.ends_with("loader");
    features.ends_with_saver = function_name.ends_with("saver");
    features.ends_with_creator = function_name.ends_with("creator");
    features.ends_with_updater = function_name.ends_with("updater");
    features.ends_with_deleter = function_name.ends_with("deleter");
    features.ends_with_finder = function_name.ends_with("finder");
    features.ends_with_parser = function_name.ends_with("parser");
    features.ends_with_renderer = function_name.ends_with("renderer");
    features.ends_with_serializer = function_name.ends_with("serializer");

    // Language
    features.language = language.clone();

    // Type info
    features.type_name = item
        .get("features")
        .and_then(|f| f.get("type_name"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    features.type_path = item
        .get("features")
        .and_then(|f| f.get("type_path"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    features.is_method = item
        .get("features")
        .and_then(|f| f.get("is_method"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    features.is_trait_impl = item
        .get("features")
        .and_then(|f| f.get("is_trait_impl"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    features.trait_name = item
        .get("features")
        .and_then(|f| f.get("trait_name"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    features.is_associated = item
        .get("features")
        .and_then(|f| f.get("is_associated"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Set defaults (legacy data won't have these)
    // Signature features
    features.is_generator = false;
    features.is_static = false;
    features.is_abstract = false;
    features.is_override = false;

    // Complexity features
    features.cognitive_complexity = 0;
    features.line_count = 0;
    features.token_count = 0;

    // Framework features
    features.is_flask_route = false;
    features.is_fastapi_route = false;
    features.is_express_route = false;
    features.is_nextjs_route = false;
    features.is_spring_controller = false;
    features.is_aspnet_controller = false;
    features.is_laravel_controller = false;
    features.is_django_view = false;
    features.is_rails_action = false;
    features.is_react_component = false;
    features.is_react_hook = false;
    features.is_vue_component = false;
    features.is_svelte_component = false;
    features.is_flutter_widget = false;
    features.is_flutter_state = false;
    features.is_go_init = false;
    features.is_go_interface = false;
    features.is_go_goroutine = false;
    features.is_rust_trait_impl = false;
    features.is_rust_ffi = false;

    // Type features
    features.has_receiver = false;
    features.has_self = false;
    features.has_generics = false;
    features.generic_count = 0;
    features.has_type_annotation = false;
    features.has_lifetime = false;

    // File context features
    features.is_in_lib = false;
    features.is_in_bin = false;
    features.is_in_proto = false;
    features.is_in_migrations = false;
    features.is_in_fixtures = false;

    // Decorator features
    features.has_decorator_route = false;
    features.has_decorator_get = false;
    features.has_decorator_post = false;
    features.has_decorator_put = false;
    features.has_decorator_delete = false;
    features.has_decorator_patch = false;
    features.has_decorator_override = false;
    features.has_decorator_staticmethod = false;
    features.has_decorator_classmethod = false;
    features.has_decorator_property = false;
    features.has_decorator_cached_property = false;
    features.has_decorator_pytest = false;
    features.has_decorator_fixture = false;
    features.has_decorator_parametrize = false;
    features.has_decorator_test = false;

    // Dynamic behavior features
    features.has_dynamic_call = false;
    features.has_ffi = false;
    features.has_macro = false;
    features.has_closure = false;
    features.has_yield = false;
    features.has_await = false;
    features.has_thread = false;

    // Error handling features
    features.has_try_catch = false;
    features.has_result_type = false;
    features.has_throw = false;
    features.has_panic = false;
    features.has_question_mark = false;
    features.has_error_propagation = false;

    // Documentation features
    features.has_doc_comment = false;
    features.doc_comment_length = 0;
    features.has_attr_doc = false;

    // Visibility features
    features.vis_pub_crate = false;
    features.vis_pub_super = false;
    features.vis_pub_self = false;
    features.vis_private = false;
    features.vis_protected = false;

    // Ownership features
    features.has_borrow = false;
    features.has_mut_ref = false;
    features.has_move = false;
    features.has_clone = false;

    // Pattern features
    features.pattern_singleton = false;
    features.pattern_factory = false;
    features.pattern_builder = false;
    features.pattern_observer = false;
    features.pattern_strategy = false;
    features.pattern_decorator = false;

    // Concurrency features
    features.has_channel = false;
    features.has_mutex = false;
    features.has_atomic = false;
    features.has_parallel = false;

    // Create and return the TrainingExample
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

// Parse legacy single line
fn parse_legacy_example(line: &str) -> Result<TrainingExample> {
    let value: serde_json::Value = serde_json::from_str(line)
        .map_err(|e| err::internal(format!("Failed to parse JSON: {}", e)))?;
    convert_legacy_to_training_example(value, "legacy")
}
