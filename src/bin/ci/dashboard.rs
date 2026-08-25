// src/bin/ci/dashboard.rs

use crate::helpers::get_default_model;
use code_intelligence::error::{err, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub async fn run_dashboard(path: &Path, model: Option<PathBuf>) -> Result<()> {
    println!("📊 Opening dashboard for: {:?}", path);

    let model_path = model.or_else(get_default_model);

    let current_exe = std::env::current_exe().ok();
    let sibling_exe = current_exe
        .as_ref()
        .and_then(|p| p.parent())
        .map(|dir| dir.join("dead_code_dashboard"));

    let mut cmd = if let Some(ref exe) = sibling_exe {
        if exe.exists() {
            Command::new(exe)
        } else {
            Command::new("dead_code_dashboard")
        }
    } else {
        Command::new("dead_code_dashboard")
    };

    cmd.arg(path);
    if let Some(m) = model_path {
        cmd.args(["--model", &m.to_string_lossy()]);
    }

    let status = cmd.status().map_err(|e| {
        err::internal(format!(
            "Failed to launch dead_code_dashboard: {}. Make sure 'dead_code_dashboard' is built or in PATH.",
            e
        ))
    })?;

    if !status.success() {
        return Err(err::internal("Dashboard exited with error status"));
    }

    Ok(())
}
