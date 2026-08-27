use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub enable_llm: bool,
    pub enable_git: bool,
    pub llm_temperature: f32,
    pub llm_max_tokens: usize,
    pub max_files: usize,
    pub max_file_size: u64,
    pub max_memory_mb: Option<usize>,
    pub timeout_seconds: Option<u64>,
    pub cache_dir: Option<PathBuf>,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            enable_llm: false,
            enable_git: false,
            llm_temperature: 0.3,
            llm_max_tokens: 1000,
            max_files: 10000,
            max_file_size: 1_000_000,
            max_memory_mb: Some(4096),
            timeout_seconds: Some(300),
            cache_dir: None,
        }
    }
}
