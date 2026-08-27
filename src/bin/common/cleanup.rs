// src/bin/common/cleanup.rs

use std::path::PathBuf;
use std::sync::Arc;
use tempfile;
use tokio::sync::Mutex;

/// Resource manager for cleanup
#[derive(Clone)]
#[allow(dead_code)]
pub struct ResourceManager {
    temp_dirs: Arc<Mutex<Vec<PathBuf>>>,
    files: Arc<Mutex<Vec<PathBuf>>>,
    cleanup_on_exit: bool,
}

impl ResourceManager {
    pub fn new(cleanup_on_exit: bool) -> Self {
        Self {
            temp_dirs: Arc::new(Mutex::new(Vec::new())),
            files: Arc::new(Mutex::new(Vec::new())),
            cleanup_on_exit,
        }
    }

    /// Register a temporary directory for cleanup
    pub async fn register_temp_dir(&self, path: PathBuf) {
        let mut dirs = self.temp_dirs.lock().await;
        dirs.push(path);
    }

    /// Register a file for cleanup
    pub async fn register_file(&self, path: PathBuf) {
        let mut files = self.files.lock().await;
        files.push(path);
    }

    /// Clean up all registered resources
    pub async fn cleanup(&self) -> crate::error::Result<()> {
        // Clean up files
        let files = {
            let mut f = self.files.lock().await;
            std::mem::take(&mut *f)
        };

        for file in files {
            if file.exists() {
                if let Err(e) = std::fs::remove_file(&file) {
                    eprintln!("⚠️ Failed to remove {}: {}", file.display(), e);
                }
            }
        }

        // Clean up directories
        let dirs = {
            let mut d = self.temp_dirs.lock().await;
            std::mem::take(&mut *d)
        };

        for dir in dirs {
            if dir.exists() {
                if let Err(e) = std::fs::remove_dir_all(&dir) {
                    eprintln!("⚠️ Failed to remove {}: {}", dir.display(), e);
                }
            }
        }

        Ok(())
    }

    /// Register a cleanup handler with the program
    pub fn install_signal_handlers(&self) {
        let manager = self.clone();

        // Use the ctrlc crate with proper handling
        let _ = ctrlc::set_handler(move || {
            println!("\n⚠️ Cleaning up resources...");
            if let Ok(rt) = tokio::runtime::Runtime::new() {
                rt.block_on(async {
                    if let Err(e) = manager.cleanup().await {
                        eprintln!("❌ Cleanup error: {}", e);
                    }
                });
            }
            std::process::exit(130);
        });
    }

    /// Clean up on drop
    pub fn with_cleanup_on_drop(self) -> impl Drop {
        struct CleanupGuard {
            manager: Option<ResourceManager>,
        }

        impl Drop for CleanupGuard {
            fn drop(&mut self) {
                if let Some(manager) = self.manager.take() {
                    if let Ok(rt) = tokio::runtime::Runtime::new() {
                        rt.block_on(async {
                            if let Err(e) = manager.cleanup().await {
                                eprintln!("❌ Cleanup error: {}", e);
                            }
                        });
                    }
                    std::process::exit(130);
                }
            }
        }

        CleanupGuard {
            manager: Some(self),
        }
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new(true)
    }
}

/// Create a temporary directory that will be cleaned up
pub async fn create_temp_dir_with_cleanup(
    manager: &ResourceManager,
    prefix: &str,
) -> crate::error::Result<tempfile::TempDir> {
    let temp_dir = tempfile::Builder::new()
        .prefix(prefix)
        .tempdir()
        .map_err(|e| anyhow::anyhow!("Failed to create temp dir: {}", e))?;

    let path = temp_dir.path().to_path_buf();
    manager.register_temp_dir(path).await;

    Ok(temp_dir)
}

/// Create a temporary file that will be cleaned up
pub async fn create_temp_file_with_cleanup(
    manager: &ResourceManager,
    prefix: &str,
    suffix: &str,
) -> crate::error::Result<tempfile::NamedTempFile> {
    let temp_file = tempfile::Builder::new()
        .prefix(prefix)
        .suffix(suffix)
        .tempfile()
        .map_err(|e| anyhow::anyhow!("Failed to create temp file: {}", e))?;

    let path = temp_file.path().to_path_buf();
    manager.register_file(path).await;

    Ok(temp_file)
}
