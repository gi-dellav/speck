use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpeckConfig {
    pub name: String,
    pub source_dir: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_model: Option<String>,
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
            plan_model: None,
            code_model: None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SpeckConfig::default();
        assert_eq!(config.name, "");
        assert_eq!(config.source_dir, "src");
        assert!(config.model.is_none());
        assert!(config.features_dir.is_none());
        assert!(config.fmt_cmd.is_none());
        assert!(config.test_cmd.is_none());
    }

    #[test]
    fn test_features_path_default() {
        let config = SpeckConfig::default();
        assert_eq!(config.features_path(), "specs/features");
    }

    #[test]
    fn test_features_path_custom() {
        let config = SpeckConfig {
            features_dir: Some("docs/features".to_string()),
            plan_model: None,
            code_model: None,
            ..Default::default()
        };
        assert_eq!(config.features_path(), "docs/features");
    }

    #[test]
    fn test_technical_path() {
        assert_eq!(SpeckConfig::technical_path(), "specs/technical");
    }

    #[test]
    fn test_roundtrip_to_file_from_file() {
        let dir = std::env::temp_dir().join(format!("speck_config_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let config_path = dir.join("Speck.toml");

        let config = SpeckConfig {
            name: "test-project".to_string(),
            source_dir: "lib".to_string(),
            model: Some("gpt-4".to_string()),
            plan_model: None,
            code_model: None,
            features_dir: Some("specs/features".to_string()),
            fmt_cmd: Some("cargo fmt".to_string()),
            test_cmd: Some("cargo test".to_string()),
        };
        config.to_file(&config_path).unwrap();
        let loaded = SpeckConfig::from_file(&config_path).unwrap();
        assert_eq!(loaded.name, "test-project");
        assert_eq!(loaded.source_dir, "lib");
        assert_eq!(loaded.model.as_deref(), Some("gpt-4"));
        assert_eq!(loaded.fmt_cmd.as_deref(), Some("cargo fmt"));
        assert_eq!(loaded.test_cmd.as_deref(), Some("cargo test"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_from_file_missing() {
        let path = std::path::PathBuf::from("/nonexistent/Speck.toml");
        let result = SpeckConfig::from_file(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_roundtrip_omits_none_fields() {
        let dir = std::env::temp_dir().join(format!("speck_config_test2_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let config_path = dir.join("Speck.toml");

        let config = SpeckConfig {
            name: "minimal".to_string(),
            source_dir: "src".to_string(),
            model: None,
            plan_model: None,
            code_model: None,
            features_dir: None,
            fmt_cmd: None,
            test_cmd: None,
        };
        config.to_file(&config_path).unwrap();
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(!content.contains("model"));
        assert!(!content.contains("features_dir"));
        assert!(!content.contains("fmt_cmd"));
        assert!(!content.contains("test_cmd"));

        let loaded = SpeckConfig::from_file(&config_path).unwrap();
        assert_eq!(loaded.name, "minimal");
        assert!(loaded.model.is_none());

        std::fs::remove_dir_all(&dir).ok();
    }
}
