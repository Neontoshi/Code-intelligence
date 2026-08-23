// src/bin/collect_training_data.rs
use code_intelligence::error::Result;
use std::path::PathBuf;
use std::process::Command;

#[tokio::main]
async fn main() -> Result<()> {
    let repos = vec![
        // Rust repositories
        "https://github.com/rust-lang/rust.git",
        "https://github.com/rust-lang/cargo.git",
        "https://github.com/rust-lang/rust-clippy.git",
        "https://github.com/rust-lang/rustfmt.git",
        "https://github.com/serde-rs/serde.git",
        "https://github.com/tokio-rs/tokio.git",
        "https://github.com/actix/actix-web.git",
        "https://github.com/diesel-rs/diesel.git",
        "https://github.com/rayon-rs/rayon.git",
        "https://github.com/rustwasm/wasm-bindgen.git",
        "https://github.com/rust-lang/rust-analyzer.git",
        "https://github.com/bevyengine/bevy.git",
        "https://github.com/paritytech/substrate.git",
        "https://github.com/hyperium/hyper.git",
        "https://github.com/async-rs/async-std.git",
        // TypeScript/JavaScript repositories (for frontend support)
        "https://github.com/facebook/react.git",
        "https://github.com/vuejs/core.git",
        "https://github.com/sveltejs/svelte.git",
        "https://github.com/angular/angular.git",
        "https://github.com/vercel/next.js.git",
        // Go repositories
        "https://github.com/golang/go.git",
        "https://github.com/kubernetes/kubernetes.git",
        "https://github.com/gin-gonic/gin.git",
        // Python repositories
        "https://github.com/python/cpython.git",
        "https://github.com/django/django.git",
        "https://github.com/pallets/flask.git",
    ];

    let training_dir = PathBuf::from("training_repos");
    std::fs::create_dir_all(&training_dir)?;

    let output_dir = PathBuf::from("training_data");
    std::fs::create_dir_all(&output_dir)?;

    for (i, repo_url) in repos.iter().enumerate() {
        let repo_name = repo_url
            .split('/')
            .last()
            .unwrap_or("unknown")
            .trim_end_matches(".git");

        println!("\n[{}/{}] Processing: {}", i + 1, repos.len(), repo_name);

        let repo_dir = training_dir.join(repo_name);

        // Clone the repository
        if !repo_dir.exists() {
            println!("   Cloning {}...", repo_url);
            let status = Command::new("git")
                .args([
                    "clone",
                    "--depth",
                    "1",
                    repo_url,
                    &repo_dir.to_string_lossy(),
                ])
                .status()?;

            if !status.success() {
                eprintln!("   ⚠️ Failed to clone {}, skipping", repo_name);
                continue;
            }
        }

        // Generate training data
        println!("   Generating training data...");
        let output_file = output_dir.join(format!("{}.json", repo_name));

        let result = std::process::Command::new("cargo")
            .args([
                "run",
                "--release",
                "--bin",
                "training_data_exporter",
                &repo_dir.to_string_lossy(),
                &output_file.to_string_lossy(),
            ])
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    println!("   ✅ Training data saved to: {:?}", output_file);
                } else {
                    eprintln!("   ⚠️ Failed to generate training data for {}", repo_name);
                    if let Ok(stderr) = String::from_utf8(output.stderr) {
                        eprintln!("      {}", stderr.lines().next().unwrap_or(""));
                    }
                }
            }
            Err(e) => {
                eprintln!("   ⚠️ Error running training_data_exporter: {}", e);
            }
        }
    }

    println!("\n✅ All repositories processed!");
    println!("📁 Training data saved in: {:?}", output_dir);

    Ok(())
}
