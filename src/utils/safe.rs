// src/utils/safe.rs

//! Safe utilities for replacing dangerous unwrap() calls

use std::path::Path;
use std::sync::MutexGuard;

/// Safe file read with proper error handling
pub fn safe_read_file(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))
}

/// Safe parse for integers
pub fn safe_parse_int(s: &str) -> Option<i64> {
    s.parse::<i64>().ok()
}

/// Safe parse for floats
pub fn safe_parse_float(s: &str) -> Option<f64> {
    s.parse::<f64>().ok()
}

/// Safe slice access
pub fn safe_slice<T>(slice: &[T], index: usize) -> Option<&T> {
    if index < slice.len() {
        Some(&slice[index])
    } else {
        None
    }
}

/// Safe mut slice access
pub fn safe_slice_mut<T>(slice: &mut [T], index: usize) -> Option<&mut T> {
    if index < slice.len() {
        Some(&mut slice[index])
    } else {
        None
    }
}

/// Safe vector pop
pub fn safe_pop<T>(vec: &mut Vec<T>) -> Option<T> {
    vec.pop()
}

pub fn safe_get<'a, K, V>(map: &'a std::collections::HashMap<K, V>, key: &K) -> Option<&'a V>
where
    K: std::hash::Hash + Eq,
{
    map.get(key)
}

pub fn safe_get_mut<'a, K, V>(
    map: &'a mut std::collections::HashMap<K, V>,
    key: &K,
) -> Option<&'a mut V>
where
    K: std::hash::Hash + Eq,
{
    map.get_mut(key)
}

/// Safe unwrap_or_default
pub fn safe_unwrap_or_default<T: Default>(opt: Option<T>) -> T {
    opt.unwrap_or_default()
}

/// Safe unwrap_or_else
pub fn safe_unwrap_or_else<T, F>(opt: Option<T>, f: F) -> T
where
    F: FnOnce() -> T,
{
    opt.unwrap_or_else(f)
}

/// Safe expect with custom message
pub fn safe_expect<T>(opt: Option<T>, msg: &str) -> Result<T, String> {
    opt.ok_or_else(|| msg.to_string())
}

/// Safe result unwrap
pub fn safe_unwrap_result<T, E: std::fmt::Display>(res: Result<T, E>) -> Result<T, String> {
    res.map_err(|e| e.to_string())
}

/// Safe mutex lock
pub fn safe_lock<'a, T>(
    guard: Result<MutexGuard<'a, T>, std::sync::PoisonError<MutexGuard<'a, T>>>,
) -> Result<MutexGuard<'a, T>, String> {
    guard.map_err(|e| format!("Mutex lock poisoned: {}", e))
}

/// Safe scaler access for ML
pub fn safe_scaler_access<T>(scaler: Option<&T>) -> Result<&T, String> {
    scaler.ok_or_else(|| "Scaler not initialized".to_string())
}

/// Safe thread join
pub fn safe_thread_join<T>(handle: std::thread::JoinHandle<T>) -> Result<T, String> {
    handle
        .join()
        .map_err(|e| format!("Thread panicked: {:?}", e))
}

/// Safe JSON serialization
pub fn safe_to_json<T: serde::Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string(value).map_err(|e| format!("JSON serialization failed: {}", e))
}

/// Safe JSON pretty serialization
pub fn safe_to_json_pretty<T: serde::Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|e| format!("JSON serialization failed: {}", e))
}
