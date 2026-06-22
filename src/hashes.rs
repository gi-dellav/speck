use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SpeckHashes {
    #[serde(default)]
    pub features_hash: BTreeMap<String, String>,
    #[serde(default)]
    pub technical_hash: BTreeMap<String, String>,
    #[serde(default)]
    pub src_hash: BTreeMap<String, String>,
}

impl SpeckHashes {
    pub fn from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let hashes: SpeckHashes = toml::from_str(&content)?;
        Ok(hashes)
    }

    pub fn to_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }


}

pub fn compute_hash(path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let content = std::fs::read(path)?;
    Ok(blake3::hash(&content).to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash_same_content_same_hash() {
        let dir = std::env::temp_dir()
            .join(format!("speck_hash_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let f1 = dir.join("a.txt");
        let f2 = dir.join("b.txt");
        std::fs::write(&f1, "hello world").unwrap();
        std::fs::write(&f2, "hello world").unwrap();
        let h1 = compute_hash(&f1).unwrap();
        let h2 = compute_hash(&f2).unwrap();
        assert_eq!(h1, h2);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_compute_hash_different_content_different_hash() {
        let dir = std::env::temp_dir()
            .join(format!("speck_hash_test2_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let f1 = dir.join("a.txt");
        let f2 = dir.join("b.txt");
        std::fs::write(&f1, "hello").unwrap();
        std::fs::write(&f2, "world").unwrap();
        let h1 = compute_hash(&f1).unwrap();
        let h2 = compute_hash(&f2).unwrap();
        assert_ne!(h1, h2);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_compute_hash_missing_file() {
        let path = std::path::PathBuf::from("/nonexistent/file.txt");
        let result = compute_hash(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_speck_hashes_default() {
        let hashes = SpeckHashes::default();
        assert!(hashes.features_hash.is_empty());
        assert!(hashes.technical_hash.is_empty());
        assert!(hashes.src_hash.is_empty());
    }

    #[test]
    fn test_speck_hashes_roundtrip() {
        let dir = std::env::temp_dir()
            .join(format!("speck_hash_rt_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let hash_path = dir.join(".speck_hash.toml");

        let mut hashes = SpeckHashes::default();
        hashes.features_hash.insert("specs/features/auth.md".to_string(), "abc123".to_string());
        hashes.technical_hash.insert("specs/technical/auth.md".to_string(), "def456".to_string());
        hashes.src_hash.insert("src/main.rs".to_string(), "789xyz".to_string());
        hashes.to_file(&hash_path).unwrap();

        let loaded = SpeckHashes::from_file(&hash_path).unwrap();
        assert_eq!(loaded.features_hash.get("specs/features/auth.md").unwrap(), "abc123");
        assert_eq!(loaded.technical_hash.get("specs/technical/auth.md").unwrap(), "def456");
        assert_eq!(loaded.src_hash.get("src/main.rs").unwrap(), "789xyz");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_speck_hashes_from_missing_file() {
        let path = std::path::PathBuf::from("/nonexistent/.speck_hash.toml");
        let result = SpeckHashes::from_file(&path);
        assert!(result.is_err());
    }
}
