// src/ml/serialization.rs

use serde::{Deserialize, Serialize};

pub fn save_model<T: Serialize>(data: &T, path: &str) -> Result<(), String> {
    let json =
        serde_json::to_string_pretty(data).map_err(|e| format!("Failed to serialize: {}", e))?;
    std::fs::write(path, json).map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(())
}

pub fn load_model<T: for<'de> Deserialize<'de>>(path: &str) -> Result<T, String> {
    let data = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
    serde_json::from_str(&data).map_err(|e| format!("Failed to parse JSON: {}", e))
}
