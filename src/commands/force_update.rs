use crate::config::SpeckConfig;
use crate::hashes::{self, SpeckHashes};
use std::path::Path;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = std::env::current_dir()?;
    let config_path = project_dir.join("Speck.toml");
    let hash_path = project_dir.join(".speck_hash.toml");

    if !config_path.exists() {
        return Err("Not a Speck project: Speck.toml not found".into());
    }

    let config = SpeckConfig::from_file(&config_path)?;
    let mut hashes = if hash_path.exists() {
        SpeckHashes::from_file(&hash_path)?
    } else {
        SpeckHashes::default()
    };

    let features_path = Path::new(config.features_path());
    let technical_path = Path::new(SpeckConfig::technical_path());
    let src_path = Path::new(&config.source_dir);

    hashes.features_hash.clear();
    if features_path.exists() {
        collect_hashes(features_path, &mut hashes.features_hash)?;
    }

    hashes.technical_hash.clear();
    if technical_path.exists() {
        collect_hashes(technical_path, &mut hashes.technical_hash)?;
    }

    hashes.src_hash.clear();
    if src_path.exists() {
        collect_hashes(src_path, &mut hashes.src_hash)?;
    }

    hashes.to_file(&hash_path)?;
    println!("All hashes updated to current files.");
    Ok(())
}

fn collect_hashes(dir: &Path, map: &mut std::collections::BTreeMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let gitignore_patterns = load_gitignore()?;

    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
    {
        let rel = entry.path().strip_prefix(&current_dir)?;
        let rel_str = rel.to_string_lossy().to_string();

        if is_ignored_file(&rel_str, entry.path(), &gitignore_patterns) {
            continue;
        }

        let hash = hashes::compute_hash(&entry.path().to_path_buf())?;
        map.insert(rel_str, hash);
    }
    Ok(())
}

fn load_gitignore() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let gitignore_path = std::env::current_dir()?.join(".gitignore");
    if !gitignore_path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(gitignore_path)?;
    Ok(content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect())
}

fn is_ignored_file(rel_str: &str, path: &Path, gitignore_patterns: &[String]) -> bool {
    if rel_str.starts_with("specs/")
        && let Some(filename) = path.file_name().and_then(|n| n.to_str())
        && filename.starts_with('_')
        && filename.ends_with(".md")
    {
        return true;
    }

    if rel_str == "Speck.toml" || rel_str == ".speck_hash.toml" {
        return true;
    }

    for pattern in gitignore_patterns {
        if pattern.contains('*') {
            let regex_pattern = pattern
                .replace('.', "\\.")
                .replace('*', ".*")
                .replace('?', ".");
            if let Ok(re) = regex::Regex::new(&format!("^{}$", regex_pattern))
                && re.is_match(rel_str)
            {
                return true;
            }
        } else if rel_str.starts_with(pattern) {
            return true;
        }
    }
    false
}
