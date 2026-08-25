// src/bin/ci/main.rs

use clap::Parser;
use code_intelligence::error::Result;

pub mod analyze;
pub mod check;
pub mod commands;
pub mod config;
pub mod dashboard;
pub mod deadcode;
pub mod dedup;
pub mod graph;
pub mod helpers;
pub mod keep;
pub mod list;
pub mod remove;
pub mod report;
pub mod stats;
pub mod types;

#[cfg(feature = "advanced")]
pub mod advanced;

pub use commands::{Args, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Analyze {
            path,
            threshold,
            verbose,
            model,
            llm,
            git,
            cache,
            cache_dir,
        } => {
            let project_path = helpers::resolve_path(&path)?;
            analyze::run_analyze(
                project_path,
                threshold,
                verbose,
                llm,
                git,
                cache,
                cache_dir,
                model,
            )
            .await?;
        }

        Commands::List { path, all } => {
            let project_path = helpers::resolve_path(&path)?;
            list::run_list(&project_path, all).await?;
        }

        Commands::DeadCode {
            path,
            threshold,
            output,
            model,
            verbose,
        } => {
            let project_path = helpers::resolve_path(&path)?;
            deadcode::run_deadcode(project_path, threshold, output, model, verbose).await?;
        }

        Commands::Dedup {
            path,
            threshold,
            output,
            ml,
            duplicate_model,
            verbose,
        } => {
            let project_path = helpers::resolve_path(&path)?;
            dedup::run_dedup_report(
                project_path,
                threshold,
                output,
                ml,
                duplicate_model,
                verbose,
            )
            .await?;
        }

        Commands::Remove { name, commit, path } => {
            let project_path = helpers::resolve_path(&path)?;
            remove::run_remove(&project_path, &name, commit)?;
        }

        Commands::Keep { name, reason, path } => {
            let project_path = helpers::resolve_path(&path)?;
            keep::run_keep(&project_path, &name, &reason)?;
        }

        Commands::Stats { path, detailed } => {
            let project_path = helpers::resolve_path(&path)?;
            stats::run_stats(&project_path, detailed)?;
        }

        Commands::Report {
            path,
            format,
            output,
            llm,
        } => {
            let project_path = helpers::resolve_path(&path)?;
            report::run_report(project_path, &format, output, llm).await?;
        }

        Commands::Graph { path, output, mode } => {
            let project_path = helpers::resolve_path(&path)?;
            graph::run_graph(&project_path, output, &mode).await?;
        }

        Commands::Dashboard { path, model } => {
            let project_path = helpers::resolve_path(&path)?;
            dashboard::run_dashboard(&project_path, model).await?;
        }

        Commands::Check {
            path,
            max_dead,
            max_ratio,
            threshold,
            format,
            output,
            fail_on_dead,
            conservative,
        } => {
            let project_path = helpers::resolve_path(&path)?;
            check::run_check(
                project_path,
                max_dead,
                max_ratio,
                &format,
                output,
                fail_on_dead,
                threshold,
                conservative,
            )
            .await?;
        }

        Commands::Config { action } => {
            config::run_config(action)?;
        }

        // Advanced Commands (Hidden)
        #[cfg(feature = "advanced")]
        Commands::Train {
            data,
            val_data,
            output,
            precision,
        } => {
            advanced::run_train(&data, val_data.as_deref(), &output, precision)?;
        }

        #[cfg(feature = "advanced")]
        Commands::TrainDuplicate { input, output } => {
            advanced::run_train_duplicate(&input, &output)?;
        }

        #[cfg(feature = "advanced")]
        Commands::Calibrate {
            model,
            data,
            output,
            method,
        } => {
            advanced::run_calibrate(&model, &data, &output, &method)?;
        }

        #[cfg(feature = "advanced")]
        Commands::Tune {
            model,
            data,
            precision,
        } => {
            advanced::run_tune(&model, &data, precision)?;
        }

        #[cfg(feature = "advanced")]
        Commands::Export { path, output } => {
            let project_path = helpers::resolve_path(&path)?;
            advanced::run_export(&project_path, &output).await?;
        }

        #[cfg(feature = "advanced")]
        Commands::Merge {
            input,
            output,
            dedup,
        } => {
            advanced::run_merge(&input, &output, dedup)?;
        }

        #[cfg(feature = "advanced")]
        Commands::Collect {
            repos,
            output,
            max_repos,
        } => {
            advanced::run_collect(&repos, &output, max_repos).await?;
        }

        #[cfg(feature = "advanced")]
        Commands::ExportFeedback { path, output } => {
            let project_path = helpers::resolve_path(&path)?;
            advanced::run_export_feedback(&project_path, &output)?;
        }

        #[cfg(feature = "advanced")]
        Commands::Update { path, id, action } => {
            let project_path = helpers::resolve_path(&path)?;
            advanced::run_update(&project_path, &id, action)?;
        }

        #[cfg(feature = "advanced")]
        Commands::SelfAnalyze { format, output } => {
            advanced::run_self_analyze(&format, output).await?;
        }
    }

    Ok(())
}

#[test]
fn test_dart_factory_ast() {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_dart::LANGUAGE.into())
        .expect("Failed to set Dart language");

    let src = "class Todo { factory Todo.fromJson(Map<String, dynamic> j) => _$TodoFromJson(j); }";
    let tree = parser
        .parse(src, None)
        .expect("Failed to parse Dart source");

    println!("\n=== DART FACTORY S-EXPRESSION ===");
    println!("{}", tree.root_node().to_sexp());
    println!("=================================\n");
}
