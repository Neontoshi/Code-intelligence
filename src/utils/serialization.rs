// src/utils/serialization.rs

//! Shared serialization utilities for saving/loading models and data

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Save a serializable object to a file with pretty JSON formatting
pub fn save_to_file<T: Serialize>(data: &T, path: &str) -> Result<(), String> {
    let json =
        serde_json::to_string_pretty(data).map_err(|e| format!("Failed to serialize: {}", e))?;

    std::fs::write(path, json).map_err(|e| format!("Failed to write file '{}': {}", path, e))?;

    Ok(())
}

/// Load a deserializable object from a JSON file
pub fn load_from_file<T: for<'de> Deserialize<'de>>(path: &str) -> Result<T, String> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file '{}': {}", path, e))?;

    let obj: T = serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse JSON from '{}': {}", path, e))?;

    Ok(obj)
}

/// Check if a file exists and is readable
pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Get file size in bytes
pub fn file_size(path: &str) -> Result<u64, String> {
    std::fs::metadata(path)
        .map(|m| m.len())
        .map_err(|e| format!("Failed to get file metadata: {}", e))
}
