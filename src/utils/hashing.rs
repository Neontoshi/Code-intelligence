use blake3::Hasher as Blake3Hasher;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub struct HashUtils;

impl HashUtils {
    /// Compute Blake3 hash of a string
    pub fn hash_string(content: &str) -> String {
        let mut hasher = Blake3Hasher::new();
        hasher.update(content.as_bytes());
        hasher.finalize().to_hex().to_string()
    }

    /// Compute SHA256 hash of a string
    pub fn sha256_string(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Compute Blake3 hash of a file
    pub fn hash_file(path: &Path) -> Result<String, std::io::Error> {
        let mut file = File::open(path)?;
        let mut hasher = Blake3Hasher::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Compute SHA256 hash of a file
    pub fn sha256_file(path: &Path) -> Result<String, std::io::Error> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Combine multiple hashes into one
    pub fn combine_hashes(hashes: &[String]) -> String {
        let mut hasher = Blake3Hasher::new();
        for hash in hashes {
            hasher.update(hash.as_bytes());
        }
        hasher.finalize().to_hex().to_string()
    }

    /// Generate a deterministic ID from content
    pub fn generate_id(content: &str) -> String {
        let hash = Self::hash_string(content);
        format!("id_{}", &hash[..16])
    }

    /// Check if two strings are identical (fast equality using hashes)
    pub fn fast_equal(a: &str, b: &str) -> bool {
        if a.len() != b.len() {
            return false;
        }
        if a == b {
            return true;
        }

        let hash_a = Self::hash_string(a);
        let hash_b = Self::hash_string(b);
        hash_a == hash_b
    }
}

/// A hash cache for memoization
pub struct HashCache {
    cache: std::collections::HashMap<String, String>,
}

impl HashCache {
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }

    pub fn get_or_compute<F>(&mut self, key: &str, compute: F) -> String
    where
        F: FnOnce() -> String,
    {
        if let Some(hash) = self.cache.get(key) {
            return hash.clone();
        }

        let hash = compute();
        self.cache.insert(key.to_string(), hash.clone());
        hash
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }
}
