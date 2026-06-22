use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpeckConfig {
    pub name: String,
    pub source_dir: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fmt_cmd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_cmd: Option<String>,
}

impl Default for SpeckConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            source_dir: "src".to_string(),
            model: None,
            features_dir: None,
            fmt_cmd: None,
            test_cmd: None,
        }
    }
}

impl SpeckConfig {
    pub fn from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: SpeckConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn to_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn features_path(&self) -> &str {
        self.features_dir.as_deref().unwrap_or("specs/features")
    }

    pub fn technical_path() -> &'static str {
        "specs/technical"
    }
}
