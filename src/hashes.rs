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

    pub fn all_files(&self) -> BTreeMap<String, String> {
        let mut all = BTreeMap::new();
        for (k, v) in &self.features_hash {
            all.insert(k.clone(), v.clone());
        }
        for (k, v) in &self.technical_hash {
            all.insert(k.clone(), v.clone());
        }
        for (k, v) in &self.src_hash {
            all.insert(k.clone(), v.clone());
        }
        all
    }
}

pub fn compute_hash(path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let content = std::fs::read(path)?;
    Ok(blake3::hash(&content).to_hex().to_string())
}
