// src/bin/ci/config.rs

use crate::commands::ConfigAction;
use crate::helpers::{load_config, save_config};
use code_intelligence::error::Result;

pub fn run_config(action: ConfigAction) -> Result<()> {
    let mut config = load_config();

    match action {
        ConfigAction::Set { key, value } => match key.as_str() {
            "model" => {
                config.defaults.model = Some(value.clone());
                save_config(&config)?;
                println!("✅ Model set to: {}", value);
            }
            "duplicate_model" => {
                config.defaults.duplicate_model = Some(value.clone());
                save_config(&config)?;
                println!("✅ Duplicate model set to: {}", value);
            }
            "threshold" => {
                let threshold = value.parse::<f64>()?;
                config.defaults.threshold = Some(threshold);
                save_config(&config)?;
                println!("✅ Threshold set to: {:.2}", threshold);
            }
            "verbose" => {
                let verbose = value.parse::<bool>()?;
                config.defaults.verbose = verbose;
                save_config(&config)?;
                println!("✅ Verbose set to: {}", verbose);
            }
            "llm_provider" => {
                config.defaults.llm_provider = Some(value.clone());
                save_config(&config)?;
                println!("✅ LLM provider set to: {}", value);
            }
            "llm_model" => {
                config.defaults.llm_model = Some(value.clone());
                save_config(&config)?;
                println!("✅ LLM model set to: {}", value);
            }
            _ => {
                println!("⚠️ Unknown config key: {}", key);
                println!("   Available: model, duplicate_model, threshold, verbose, llm_provider, llm_model");
            }
        },
        ConfigAction::Get { key } => match key.as_str() {
            "model" => println!(
                "{}",
                config.defaults.model.as_deref().unwrap_or("(not set)")
            ),
            "duplicate_model" => println!(
                "{}",
                config
                    .defaults
                    .duplicate_model
                    .as_deref()
                    .unwrap_or("(not set)")
            ),
            "threshold" => println!("{:.2}", config.defaults.threshold.unwrap_or(0.92)),
            "verbose" => println!("{}", config.defaults.verbose),
            "llm_provider" => println!(
                "{}",
                config.defaults.llm_provider.as_deref().unwrap_or("ollama")
            ),
            "llm_model" => println!(
                "{}",
                config.defaults.llm_model.as_deref().unwrap_or("phi:2.7b")
            ),
            _ => println!("⚠️ Unknown config key: {}", key),
        },
        ConfigAction::List => {
            println!("📋 Current Configuration:");
            println!("");
            println!("[defaults]");
            println!("  model = {:?}", config.defaults.model);
            println!("  duplicate_model = {:?}", config.defaults.duplicate_model);
            println!("  threshold = {:?}", config.defaults.threshold);
            println!("  verbose = {}", config.defaults.verbose);
            println!("  llm_provider = {:?}", config.defaults.llm_provider);
            println!("  llm_model = {:?}", config.defaults.llm_model);
        }
    }

    Ok(())
}
