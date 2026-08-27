// src/engine/cache.rs

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

// Cache Entry Types
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

    pub fn validate(&self, path: &Path) -> bool {
        if let Some(dir) = &self.persistent_dir {
            let hash = self.hash_file(path);
            if let Some(hash) = hash {
                let cache_path = dir.join(format!("{}.cache", hash));
                if cache_path.exists() {
                    // Verify the cache file is readable and valid
                    if let Ok(data) = std::fs::read_to_string(&cache_path) {
                        if let Ok(_entry) = serde_json::from_str::<CacheEntry<String>>(&data) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Clear invalid cache entries
    pub fn clear_invalid(&self) -> usize {
        let mut cleared = 0;
        if let Some(dir) = &self.persistent_dir {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "cache" {
                            // Check if the cache entry is valid
                            if let Ok(data) = std::fs::read_to_string(&path) {
                                if let Ok(entry) = serde_json::from_str::<CacheEntry<String>>(&data)
                                {
                                    // Validate the entry
                                    let key = entry.hash;
                                    if key.is_empty() {
                                        let _ = std::fs::remove_file(&path);
                                        cleared += 1;
                                    }
                                } else {
                                    let _ = std::fs::remove_file(&path);
                                    cleared += 1;
                                }
                            } else {
                                let _ = std::fs::remove_file(&path);
                                cleared += 1;
                            }
                        }
                    }
                }
            }
        }
        cleared
    }
}

impl Default for FileCache {
    fn default() -> Self {
        Self::new()
    }
}

/// File entry with path and content hash for validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAnalysis {
    pub project_hash: String,
    pub root: String,
    pub function_count: usize,
    pub edge_count: usize,
    pub file_count: usize,
    pub timestamp: i64,
    pub files: Vec<CachedFileEntry>,
}

pub struct AnalysisCacheManager {
    cache_dir: PathBuf,
    cache: DashMap<String, AnalysisCache>,
    #[allow(dead_code)]
    file_cache: FileCache,
}

impl AnalysisCacheManager {
    pub fn new(root: &Path) -> Self {
        let cache_dir = root.join(".code-intelligence-cache");
        let _ = std::fs::create_dir_all(&cache_dir);

        let cache = DashMap::new();
        let file_cache = FileCache::new().with_persistent_dir(cache_dir.clone());

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

        Self {
            cache_dir,
            cache,
            file_cache,
        }
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
    pub fn is_valid(&self, project_hash: &str, files: &[CachedFileEntry]) -> bool {
        if let Some(cached) = self.get(project_hash) {
            cached.files == files
        } else {
            false
        }
    }

    /// Save full analysis result to cache
    pub fn save_analysis(
        &self,
        project_hash: &str,
        root: &Path,
        function_count: usize,
        edge_count: usize,
        files: &[CachedFileEntry],
    ) -> Result<(), String> {
        let cache_path = self
            .cache_dir
            .join(format!("{}.analysis.json", project_hash));

        let cached = CachedAnalysis {
            project_hash: project_hash.to_string(),
            root: root.to_string_lossy().to_string(),
            function_count,
            edge_count,
            file_count: files.len(),
            timestamp: chrono::Utc::now().timestamp(),
            files: files.to_vec(),
        };

        let data = serde_json::to_string_pretty(&cached)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        std::fs::write(&cache_path, data).map_err(|e| format!("Failed to write cache: {}", e))?;

        Ok(())
    }

    /// Load cached analysis metadata (full reconstruction would need file contents)
    pub fn load_analysis_metadata(&self, project_hash: &str) -> Option<CachedAnalysis> {
        let cache_path = self
            .cache_dir
            .join(format!("{}.analysis.json", project_hash));
        let data = std::fs::read_to_string(&cache_path).ok()?;
        serde_json::from_str(&data).ok()
    }

    /// Check if cached analysis exists and is valid
    pub fn has_valid_analysis(&self, project_hash: &str, files: &[CachedFileEntry]) -> bool {
        if let Some(cached) = self.load_analysis_metadata(project_hash) {
            // Validate files haven't changed
            cached.files == files
        } else {
            false
        }
    }

    pub fn clear(&self) {
        self.cache.clear();
        let _ = std::fs::remove_dir_all(&self.cache_dir);
    }
}

#[derive(Debug, Clone)]
pub struct BinaryCache {
    persistent_dir: PathBuf,
    version: u32,
}

impl BinaryCache {
    pub fn new(dir: PathBuf, version: u32) -> Self {
        let _ = std::fs::create_dir_all(&dir);
        Self {
            persistent_dir: dir,
            version,
        }
    }

    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        let cache_path = self.persistent_dir.join(format!("{}.bin", key));
        let data = std::fs::read(&cache_path).ok()?;
        bincode::deserialize(&data).ok()
    }

    pub fn put<T: serde::Serialize>(&self, key: &str, value: &T) -> Result<(), String> {
        let cache_path = self.persistent_dir.join(format!("{}.bin", key));
        let data =
            bincode::serialize(value).map_err(|e| format!("Failed to serialize cache: {}", e))?;
        std::fs::write(&cache_path, data).map_err(|e| format!("Failed to write cache: {}", e))?;
        Ok(())
    }

    pub fn exists(&self, key: &str) -> bool {
        self.persistent_dir.join(format!("{}.bin", key)).exists()
    }

    pub fn version(&self) -> u32 {
        self.version
    }
}

pub struct ContentCache {
    pub file_cache: FileCache,
    pub ast_cache: BinaryCache,
    pub feature_cache: BinaryCache,
    pub graph_cache: BinaryCache,
    pub dynamic_refs_cache: BinaryCache,
    pub framework_cache: BinaryCache,
}

impl ContentCache {
    pub fn new(root: &Path) -> Self {
        let cache_dir = root.join(".code-intelligence-cache");
        let _ = std::fs::create_dir_all(&cache_dir);

        Self {
            file_cache: FileCache::new().with_persistent_dir(cache_dir.join("files")),
            ast_cache: BinaryCache::new(cache_dir.join("ast"), 1),
            feature_cache: BinaryCache::new(cache_dir.join("features"), 1),
            graph_cache: BinaryCache::new(cache_dir.join("graph"), 1),
            dynamic_refs_cache: BinaryCache::new(cache_dir.join("dynamic_refs"), 1),
            framework_cache: BinaryCache::new(cache_dir.join("framework"), 1),
        }
    }

    pub fn clear_all(&self) {
        self.file_cache.clear();
        let _ = std::fs::remove_dir_all(&self.ast_cache.persistent_dir);
        let _ = std::fs::remove_dir_all(&self.feature_cache.persistent_dir);
        let _ = std::fs::remove_dir_all(&self.graph_cache.persistent_dir);
        let _ = std::fs::remove_dir_all(&self.dynamic_refs_cache.persistent_dir);
        let _ = std::fs::remove_dir_all(&self.framework_cache.persistent_dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_cache_hash_file() {
        let cache = FileCache::new();
        let temp_file = temp_dir().join("test.txt");

        // Test code - unwrap is acceptable here
        std::fs::write(&temp_file, "hello world").unwrap();
        let hash = cache.hash_file(&temp_file);
        assert!(hash.is_some());

        let hash_str = hash.unwrap();
        assert_eq!(hash_str.len(), 64);
    }

    #[test]
    fn test_cache_hash_content() {
        let cache = FileCache::new();
        let hash1 = cache.hash_content("hello world");
        let hash2 = cache.hash_content("hello world");
        assert_eq!(hash1, hash2);

        let hash3 = cache.hash_content("hello world!");
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_analysis_cache_manager() {
        let temp_dir = temp_dir().join("test_cache");
        let manager = AnalysisCacheManager::new(&temp_dir);

        let project_hash = "test_project";
        let files = vec![CachedFileEntry {
            path: "src/main.rs".to_string(),
            content_hash: "abc123".to_string(),
        }];

        let cache_entry = AnalysisCache {
            project_hash: project_hash.to_string(),
            files: files.clone(),
            function_count: 10,
            edge_count: 5,
            timestamp: chrono::Utc::now().timestamp(),
        };

        manager.put(project_hash, &cache_entry);

        let retrieved = manager.get(project_hash);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().function_count, 10);

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
