// src/config.rs
use crate::error::{err, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub projects: HashMap<String, ProjectConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defaults {
    pub model: Option<String>,
    pub duplicate_model: Option<String>,
    pub threshold: Option<f64>,
    pub verbose: bool,
    pub llm_provider: Option<String>,
    pub llm_model: Option<String>,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            model: None,
            duplicate_model: None,
            threshold: None,
            verbose: false,
            llm_provider: Some("ollama".to_string()),
            llm_model: Some("phi:2.7b".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub path: String,
    pub project_type: Option<String>,
    pub threshold: Option<f64>,
    pub last_analyzed: Option<String>,
    pub dead_count: Option<usize>,
}

pub fn get_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config/code-intelligence/config.toml")
}

pub fn load_config() -> GlobalConfig {
    let path = get_config_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        toml::from_str(&content).unwrap_or_else(|_| GlobalConfig {
            defaults: Defaults::default(),
            projects: HashMap::new(),
        })
    } else {
        GlobalConfig {
            defaults: Defaults::default(),
            projects: HashMap::new(),
        }
    }
}

pub fn save_config(config: &GlobalConfig) -> Result<()> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| err::config(format!("Failed to create config dir: {}", e)))?;
    }
    let content = toml::to_string_pretty(config)
        .map_err(|e| err::config(format!("Failed to serialize: {}", e)))?;
    std::fs::write(&path, content)
        .map_err(|e| err::config(format!("Failed to write config: {}", e)))?;
    Ok(())
}

pub fn save_project_config(path: &Path, project_config: ProjectConfig) -> Result<()> {
    let mut config = load_config();
    let key = path
        .canonicalize()
        .unwrap_or(path.to_path_buf())
        .to_string_lossy()
        .to_string();
    config.projects.insert(key, project_config);
    save_config(&config)
}

pub fn get_default_model() -> Option<PathBuf> {
    load_config().defaults.model.map(PathBuf::from)
}

pub fn get_default_duplicate_model() -> Option<PathBuf> {
    load_config().defaults.duplicate_model.map(PathBuf::from)
}

pub fn get_default_threshold() -> Option<f64> {
    load_config().defaults.threshold
}

pub fn detect_project_type(path: &Path) -> Option<String> {
    let path = path.canonicalize().unwrap_or(path.to_path_buf());
    let has_rust = path.join("Cargo.toml").exists();
    let has_typescript = path.join("package.json").exists() && path.join("tsconfig.json").exists();
    let has_javascript = path.join("package.json").exists();
    let has_go = path.join("go.mod").exists();
    let has_java = path.join("pom.xml").exists() || path.join("build.gradle").exists();
    let has_python = path.join("requirements.txt").exists() || path.join("pyproject.toml").exists();
    let has_dart = path.join("pubspec.yaml").exists();
    let has_php = path.join("composer.json").exists();
    let has_cpp = path.join("CMakeLists.txt").exists() || path.join("Makefile").exists();

    let lang_count = [
        has_rust,
        has_typescript,
        has_go,
        has_java,
        has_python,
        has_dart,
        has_php,
        has_cpp,
    ]
    .iter()
    .filter(|&&x| x)
    .count();

    if lang_count > 1 {
        Some("mixed".to_string())
    } else if has_rust {
        Some("rust".to_string())
    } else if has_typescript {
        Some("typescript".to_string())
    } else if has_javascript {
        Some("javascript".to_string())
    } else if has_go {
        Some("go".to_string())
    } else if has_java {
        Some("java".to_string())
    } else if has_python {
        Some("python".to_string())
    } else if has_dart {
        Some("dart".to_string())
    } else if has_php {
        Some("php".to_string())
    } else if has_cpp {
        Some("cpp".to_string())
    } else {
        None
    }
}

pub fn resolve_path(path: &Path) -> Result<PathBuf> {
    let resolved = if path.to_string_lossy() == "." {
        std::env::current_dir().map_err(|e| err::io(path.to_path_buf(), e))?
    } else if path.is_relative() {
        std::env::current_dir()
            .map_err(|e| err::io(path.to_path_buf(), e))?
            .join(path)
    } else {
        path.to_path_buf()
    };
    if !resolved.exists() {
        return Err(err::analysis(format!(
            "Path does not exist: {:?}",
            resolved
        )));
    }
    Ok(resolved)
}
