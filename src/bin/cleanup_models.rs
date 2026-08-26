// src/bin/cleanup_models.rs

//! Clean up obsolete/duplicate ML models

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(about = "Clean up obsolete ML models")]
struct Args {
    /// Models directory
    #[arg(long, default_value = "models")]
    models_dir: PathBuf,

    /// Dry run - don't actually delete anything
    #[arg(long)]
    dry_run: bool,
}

fn main() {
    let args = Args::parse();

    println!("🧹 Cleaning up ML models in: {:?}", args.models_dir);
    if args.dry_run {
        println!("   DRY RUN - no files will be deleted");
    }

    // Models to remove (obsolete/duplicated)
    let obsolete_models = vec![
        "duplicate_model.bin",     // Duplicate - no version number
        "model_v2_calibrated.bin", // Old version
        "model_v2.bin",            // Old version
        "model_v3.bin",            // Old version (if exists)
        "model_v3_calibrated.bin", // Old version (if exists)
    ];

    // Keep these
    let keep_models = vec![
        "model.bin",              // Current production model
        "duplicate_model_v4.bin", // Current duplicate model (if exists)
    ];

    for model_name in &obsolete_models {
        let path = args.models_dir.join(model_name);
        if path.exists() {
            if args.dry_run {
                println!("   Would remove: {:?}", path);
            } else {
                match std::fs::remove_file(&path) {
                    Ok(_) => println!("   ✅ Removed: {:?}", path),
                    Err(e) => println!("   ❌ Failed to remove {:?}: {}", path, e),
                }
            }
        }
    }

    println!("\n   Keeping:");
    for model_name in &keep_models {
        let path = args.models_dir.join(model_name);
        if path.exists() {
            println!("   ✅ Keep: {:?}", path);
        }
    }
}
