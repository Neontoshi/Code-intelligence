// src/engine/config.rs

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub enable_llm: bool,
    pub enable_git: bool,
    pub llm_temperature: f32,
    pub llm_max_tokens: usize,
    pub max_files: usize,
    pub max_file_size: u64,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            enable_llm: false,
            enable_git: false,
            llm_temperature: 0.3,
            llm_max_tokens: 1000,
            max_files: 10000,
            max_file_size: 1_000_000, // 1MB
        }
    }
}
