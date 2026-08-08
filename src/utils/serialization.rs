use std::path::Path;

pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn file_size(path: &str) -> Result<u64, String> {
    std::fs::metadata(path)
        .map(|m| m.len())
        .map_err(|e| format!("Failed to get file metadata: {}", e))
}
