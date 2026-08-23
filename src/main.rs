// src/main.rs

use clap::Parser;
use code_intelligence::graph::traits::GraphMetrics;
use code_intelligence::{
    engine::pipeline::Pipeline,
    llm::{create_provider_from_string, ProviderConfig, ProviderType},
};

use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Semantic Code analysis Engine with LLM Support"
)]
struct Args {
    /// Project directory to analyze
    project_dir: PathBuf,

    /// Output format: json, markdown, graphviz, graphviz-focused, graphviz-summary, full, training, pairs
    #[arg(short, long, default_value = "markdown")]
    format: String,

    /// Output file
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Include git history analysis
    #[arg(long)]
    git: bool,

    /// Importance threshold (0.0 - 1.0)
    #[arg(long, default_value = "0.0")]
    threshold: f64,

    /// Max nodes to render for graphviz output (bounds file size)
    #[arg(long, default_value = "60")]
    graph_max_nodes: usize,

    /// Entry point full_path for graphviz-focused output
    #[arg(long)]
    graph_entry: Option<String>,

    /// Depth (call-hops) for graphviz-focused output
    #[arg(long, default_value = "2")]
    graph_depth: usize,

    /// Enable LLM analysis
    #[arg(long)]
    llm: bool,

    /// LLM provider: ollama, openai, anthropic, mock
    #[arg(long, default_value = "ollama")]
    llm_provider: String,

    /// LLM model name (default: phi:2.7b for ollama)
    #[arg(long)]
    llm_model: Option<String>,

    /// LLM API key (for cloud providers)
    #[arg(long)]
    llm_api_key: Option<String>,

    /// LLM base URL (for custom endpoints)
    #[arg(long)]
    llm_base_url: Option<String>,

    /// LLM temperature (0.0 - 1.0)
    #[arg(long, default_value = "0.3")]
    llm_temperature: f32,

    /// LLM max tokens
    #[arg(long, default_value = "1000")]
    llm_max_tokens: usize,

    /// Skip LLM cache
    #[arg(long)]
    llm_no_cache: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Max files to analyze
    #[arg(long, default_value = "10000")]
    max_files: usize,

    /// Max file size in bytes
    #[arg(long, default_value = "1000000")]
    max_file_size: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    if !args.project_dir.is_dir() {
        return Err(anyhow::anyhow!("{:?} is not a directory", args.project_dir).into());
    }

    let root = args.project_dir.canonicalize()?;

    println!("🔍 Analyzing project: {:?}", root);
    println!("📊 Output format: {}", args.format);

    // Build pipeline with configuration
    let config = code_intelligence::PipelineConfig {
        enable_llm: args.llm,
        enable_git: args.git,
        llm_temperature: args.llm_temperature,
        llm_max_tokens: args.llm_max_tokens,
        max_files: args.max_files,
        max_file_size: args.max_file_size,
        max_memory_mb: None,
        timeout_seconds: None,
    };

    let mut pipeline = Pipeline::new().with_config(config.clone());

    // Initialize LLM if enabled
    if args.llm {
        match setup_llm(&args).await {
            Ok(provider) => {
                println!(
                    "🤖 LLM initialized: {} (model: {})",
                    args.llm_provider,
                    provider.model_name()
                );
                pipeline = pipeline.with_llm(provider);
            }
            Err(e) => {
                eprintln!("⚠️ Failed to initialize LLM: {}", e);
                eprintln!("   Continuing without LLM support.");
            }
        }
    }

    // Add git if enabled
    if args.git {
        pipeline = pipeline.enable_git();
    }

    // Process project
    let analysis = if args.git {
        pipeline.process_project_with_git(&root).await?
    } else {
        pipeline.process_project(&root).await?
    };

    // Print LLM analysis summary if available
    if let Some(llm_analysis) = &analysis.llm_analysis {
        println!("\n🤖 LLM Analysis Results:");
        if llm_analysis.has_documentation {
            println!("   ✅ Documentation generated");
        }
        if llm_analysis.summarized_count > 0 {
            println!(
                "   ✅ {} functions summarized",
                llm_analysis.summarized_count
            );
        }
        if llm_analysis.issues_count > 0 {
            println!("   ⚠️ {} issues found", llm_analysis.issues_count);
        }
    }

    // Generate output
    let output = match args.format.as_str() {
        "json" => analysis.to_json(),
        "training" => analysis.to_training_json(),
        "pairs" => {
            use code_intelligence::output::JsonOutput;
            JsonOutput::generate_training_pairs(&analysis.call_graph, &analysis.files)
        }
        "rag" => {
            use code_intelligence::output::rag::RAGGenerator;
            RAGGenerator::generate_rag_markdown(&analysis.call_graph, &analysis.files)
        }
        "full" => analysis.to_full_report(),
        _ => analysis.to_markdown(),
    };

    // Write output
    let output_path = args.output.unwrap_or_else(|| {
        let ext = match args.format.as_str() {
            "json" => "json",
            "training" => "json",
            "pairs" => "jsonl",
            _ => "md",
        };
        PathBuf::from(format!("code_analysis.{}", ext))
    });

    std::fs::write(&output_path, output)?;
    println!("✅ Output written to: {:?}", output_path);

    // Print stats
    println!("\n📊 Project Stats:");
    println!(
        "   Functions analyzed: {}",
        analysis.call_graph.node_count()
    );
    println!(
        "   Relationships found: {}",
        analysis.call_graph.edge_count()
    );
    println!("   Files analyzed: {}", analysis.files.len());

    // Print important functions if threshold is set
    if args.threshold > 0.0 {
        println!("\n🔥 Important Functions (score > {:.2}):", args.threshold);
        let mut count = 0;
        for idx in analysis.call_graph.node_indices() {
            let func = &analysis.call_graph[idx];
            if func.importance_score >= args.threshold {
                println!("   - {} (score: {:.2})", func.name, func.importance_score);
                count += 1;
                if count >= 20 {
                    println!(
                        "   ... and {} more",
                        analysis.call_graph.node_count() - count
                    );
                    break;
                }
            }
        }
    }

    // Print LLM issues if any
    if let Some(llm_analysis) = &analysis.llm_analysis {
        if !llm_analysis.issues.is_empty() {
            println!("\n🐛 Issues Found by LLM:");
            for (func_name, issue) in llm_analysis.issues.iter().take(10) {
                println!(
                    "   - [{}] {}: {}",
                    issue.severity.to_uppercase(),
                    func_name,
                    issue.description
                );
            }
            if llm_analysis.issues.len() > 10 {
                println!("   ... and {} more", llm_analysis.issues.len() - 10);
            }
        }
    }

    Ok(())
}

// LLM Setup

async fn setup_llm(args: &Args) -> Result<Arc<dyn code_intelligence::llm::LLMProvider>, String> {
    // Check if we should use the connection string format
    if args.llm_provider.contains(":") {
        // Connection string format: provider:model@url
        let provider = create_provider_from_string(&args.llm_provider)
            .await
            .map_err(|e| format!("Failed to parse LLM connection string: {}", e))?;
        return Ok(provider);
    }

    // Build provider config
    let mut config = ProviderConfig {
        api_key: args.llm_api_key.clone(),
        base_url: args.llm_base_url.clone(),
        model: args
            .llm_model
            .clone()
            .unwrap_or_else(|| match args.llm_provider.as_str() {
                "ollama" => "phi:2.7b".to_string(),
                "openai" => "gpt-3.5-turbo".to_string(),
                "anthropic" => "claude-3-haiku-20240307".to_string(),
                "mock" => "mock".to_string(),
                _ => "phi:2.7b".to_string(),
            }),
        timeout_seconds: 60,
        max_retries: 3,
        extra_headers: Vec::new(),
    };

    // Set Ollama base URL if not provided
    if args.llm_provider == "ollama" && config.base_url.is_none() {
        config.base_url = Some("http://localhost:11434".to_string());
    }

    // Create provider
    let provider_type = match args.llm_provider.as_str() {
        "ollama" => ProviderType::Ollama,
        "openai" => ProviderType::OpenAI,
        "anthropic" => ProviderType::Anthropic,
        "mock" => ProviderType::Mock,
        _ => ProviderType::Ollama,
    };

    let provider = code_intelligence::llm::providers::create_provider(provider_type, &config)
        .await
        .map_err(|e| format!("Failed to create provider: {}", e))?;

    // Test availability
    if !provider.is_available().await {
        let mut warning = format!("Provider '{}' is not available.", args.llm_provider);
        if args.llm_provider == "ollama" {
            warning.push_str("\n   Please make sure Ollama is running:");
            warning.push_str("\n   $ ollama serve");
            warning.push_str(&format!("\n   $ ollama pull {}", config.model));
        } else if args.llm_provider == "openai" || args.llm_provider == "anthropic" {
            warning.push_str("\n   Please check your API key and internet connection.");
        }
        return Err(warning);
    }

    Ok(provider)
}

// Help Text Extension

/// Print extended help for LLM options
pub fn print_llm_help() {
    println!("\n🤖 LLM Options:");
    println!("  --llm                    Enable LLM analysis");
    println!("  --llm-provider <name>    Provider: ollama, openai, anthropic, mock");
    println!("  --llm-model <name>       Model name (default depends on provider)");
    println!("  --llm-api-key <key>      API key for cloud providers");
    println!("  --llm-base-url <url>     Custom API endpoint");
    println!("  --llm-temperature <n>    Temperature (0.0-1.0, default: 0.3)");
    println!("  --llm-max-tokens <n>     Max tokens (default: 1000)");
    println!("  --llm-no-cache           Disable LLM response cache");
    println!("\nExamples:");
    println!("  # Ollama with phi-2 (default)");
    println!("  code-analysis ./project --llm");
    println!("\n  # Ollama with custom model");
    println!("  code-analysis ./project --llm --llm-model llama2");
    println!("\n  # OpenAI with GPT-4");
    println!("  code-analysis ./project --llm --llm-provider openai --llm-model gpt-4 --llm-api-key $OPENAI_API_KEY");
    println!("\n  # Connection string format");
    println!(
        "  code-analysis ./project --llm --llm-provider ollama:phi:2.7b@http://localhost:11434"
    );
    println!();
}
