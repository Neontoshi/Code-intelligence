// src/engine/cache.rs

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

// ============================================================================
// Cache Entry Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub hash: String,
    pub data: T,
    pub timestamp: i64,
    pub version: u32,
}

impl<T> CacheEntry<T> {
    fn now() -> i64 {
        chrono::Utc::now().timestamp()
    }
}

#[derive(Debug, Clone)]
pub struct FileCache {
    cache: DashMap<String, CacheEntry<String>>,
    persistent_dir: Option<PathBuf>,
    version: u32,
}

impl FileCache {
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
            persistent_dir: None,
            version: 1,
        }
    }

    pub fn with_persistent_dir(mut self, dir: PathBuf) -> Self {
        self.persistent_dir = Some(dir.clone());
        let _ = std::fs::create_dir_all(&dir);
        self
    }

    pub fn get_or_compute<F>(&self, path: &Path, compute: F) -> Option<String>
    where
        F: FnOnce() -> String,
    {
        let key = path.to_string_lossy().to_string();
        let hash = self.hash_file(path)?;

        // Check memory cache first
        if let Some(entry) = self.cache.get(&key) {
            if entry.hash == hash && entry.version == self.version {
                return Some(entry.data.clone());
            }
        }

        // Check persistent cache
        if let Some(dir) = &self.persistent_dir {
            let cache_path = dir.join(format!("{}.cache", hash));
            if let Ok(data) = std::fs::read_to_string(&cache_path) {
                if let Ok(entry) = serde_json::from_str::<CacheEntry<String>>(&data) {
                    if entry.hash == hash && entry.version == self.version {
                        self.cache.insert(
                            key,
                            CacheEntry {
                                hash: entry.hash.clone(),
                                data: entry.data.clone(),
                                timestamp: CacheEntry::<String>::now(),
                                version: self.version,
                            },
                        );
                        return Some(entry.data);
                    }
                }
            }
        }

        // Compute fresh
        let data = compute();
        let entry = CacheEntry {
            hash: hash.clone(),
            data: data.clone(),
            timestamp: CacheEntry::<String>::now(),
            version: self.version,
        };

        self.cache.insert(key, entry.clone());

        // Store persistent
        if let Some(dir) = &self.persistent_dir {
            let cache_path = dir.join(format!("{}.cache", hash));
            if let Ok(json) = serde_json::to_string(&entry) {
                let _ = std::fs::write(cache_path, json);
            }
        }

        Some(data)
    }

    /// Returns None for unreadable files (no more sentinel empty string)
    pub fn hash_file(&self, path: &Path) -> Option<String> {
        let contents = std::fs::read(path).ok()?;
        let mut hasher = Sha256::new();
        hasher.update(&contents);
        Some(format!("{:x}", hasher.finalize()))
    }

    pub fn hash_content(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn clear(&self) {
        self.cache.clear();
        if let Some(dir) = &self.persistent_dir {
            let _ = std::fs::remove_dir_all(dir);
        }
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }
}

impl Default for FileCache {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Analysis Cache - Stores full ProjectIntelligence with content hashes
// ============================================================================

/// File entry with content hash for validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CachedFileEntry {
    pub path: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisCache {
    pub project_hash: String,
    pub files: Vec<CachedFileEntry>,
    pub function_count: usize,
    pub edge_count: usize,
    pub timestamp: i64,
}

pub struct AnalysisCacheManager {
    cache_dir: PathBuf,
    cache: DashMap<String, AnalysisCache>,
}

impl AnalysisCacheManager {
    pub fn new(root: &Path) -> Self {
        let cache_dir = root.join(".code-intelligence-cache");
        let _ = std::fs::create_dir_all(&cache_dir);

        let cache = DashMap::new();

        // Load existing cache
        if let Ok(entries) = std::fs::read_dir(&cache_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "json" {
                        if let Ok(data) = std::fs::read_to_string(&path) {
                            if let Ok(cache_entry) = serde_json::from_str::<AnalysisCache>(&data) {
                                let key = path.file_stem().unwrap_or_default().to_string_lossy();
                                cache.insert(key.to_string(), cache_entry);
                            }
                        }
                    }
                }
            }
        }

        Self { cache_dir, cache }
    }

    pub fn get(&self, project_hash: &str) -> Option<AnalysisCache> {
        self.cache.get(project_hash).map(|e| e.clone())
    }

    pub fn put(&self, project_hash: &str, entry: &AnalysisCache) {
        self.cache.insert(project_hash.to_string(), entry.clone());

        let cache_path = self.cache_dir.join(format!("{}.json", project_hash));
        if let Ok(json) = serde_json::to_string(entry) {
            let _ = std::fs::write(cache_path, json);
        }
    }

    /// Validate cache by comparing file paths AND content hashes
    pub fn is_valid(&self, project_hash: &str, files: &[(PathBuf, String)]) -> bool {
        if let Some(cached) = self.get(project_hash) {
            // Build current entries: (path_string, content_hash)
            let current: Vec<CachedFileEntry> = files
                .iter()
                .filter_map(|(path, hash)| {
                    path.to_str().map(|s| CachedFileEntry {
                        path: s.to_string(),
                        content_hash: hash.clone(),
                    })
                })
                .collect();

            cached.files == current
        } else {
            false
        }
    }

    /// Legacy version for compatibility (deprecated - use is_valid with hashes)
    #[deprecated(note = "Use is_valid with content hashes instead")]
    pub fn is_valid_legacy(&self, project_hash: &str, files: &[PathBuf]) -> bool {
        if let Some(cached) = self.get(project_hash) {
            let current_paths: Vec<String> = files
                .iter()
                .filter_map(|p| p.to_str().map(|s| s.to_string()))
                .collect();
            let cached_paths: Vec<String> = cached.files.iter().map(|f| f.path.clone()).collect();
            cached_paths == current_paths
        } else {
            false
        }
    }
}
