// src/bin/ci/dashboard.rs

use code_intelligence::error::{err, Result};

use std::path::Path;
use std::path::PathBuf;

pub async fn run_dashboard(path: &Path, model: Option<PathBuf>) -> Result<()> {
    println!("📊 Opening dashboard for: {:?}", path);

    let mut cmd = std::process::Command::new("dead_code_dashboard");
    cmd.arg(path);
    if let Some(m) = model {
        cmd.args(["--model", &m.to_string_lossy()]);
    }
    let status = cmd.status()?;

    if !status.success() {
        return Err(err::internal("Dashboard failed"));
    }

    Ok(())
}
