use crate::config::SpeckConfig;
use crate::hashes::{self, SpeckHashes};
use std::process::Command;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = std::env::current_dir()?;
    let config_path = project_dir.join("Speck.toml");
    let hash_path = project_dir.join(".speck_hash.toml");

    if !config_path.exists() {
        return Err("Not a Speck project: Speck.toml not found".into());
    }
    if !hash_path.exists() {
        return Err("Not a Speck project: .speck_hash.toml not found".into());
    }

    let config = SpeckConfig::from_file(&config_path)?;
    let stored_hashes = SpeckHashes::from_file(&hash_path)?;

    let src_path = std::path::PathBuf::from(&config.source_dir);
    if src_path.exists() {
        let gitignore_patterns = load_gitignore()?;
        for entry in walkdir::WalkDir::new(&src_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
        {
            let rel = entry.path().strip_prefix(&project_dir)?;
            let rel_str = rel.to_string_lossy().to_string();

            if is_ignored_file(&rel_str, entry.path(), &gitignore_patterns) {
                continue;
            }

            let current_hash = hashes::compute_hash(&entry.path().to_path_buf())?;
            if let Some(stored_hash) = stored_hashes.src_hash.get(&rel_str)
                && &current_hash != stored_hash
            {
                return Err(format!(
                    "Cannot format: source file '{}' has uncommitted edits. Run `speck apply` first.",
                    rel_str
                )
                .into());
            }
        }
    }

    let fmt_cmd = match &config.fmt_cmd {
        Some(cmd) => cmd.clone(),
        None => return Err("No fmt_cmd defined in Speck.toml".into()),
    };

    let status = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", &fmt_cmd]).status()?
    } else {
        Command::new("sh").args(["-c", &fmt_cmd]).status()?
    };

    if !status.success() {
        return Err("Formatting command failed".into());
    }

    let mut hashes = SpeckHashes::from_file(&hash_path)?;
    hashes.src_hash.clear();
    if src_path.exists() {
        collect_section_hashes(&src_path, &mut hashes.src_hash)?;
    }
    hashes.to_file(&hash_path)?;

    println!("Formatted and updated source hashes.");
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

fn is_ignored_file(rel_str: &str, path: &std::path::Path, gitignore_patterns: &[String]) -> bool {
    if rel_str.starts_with("specs/")
        && path.file_name().and_then(|n| n.to_str()).is_some_and(|f| f.starts_with('_') && f.ends_with(".md"))
    {
        return true;
    }
    if rel_str == "Speck.toml" || rel_str == ".speck_hash.toml" {
        return true;
    }
    for pattern in gitignore_patterns {
        if pattern.contains('*') {
            let regex_pattern = pattern.replace('.', "\\.").replace('*', ".*").replace('?', ".");
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

fn collect_section_hashes(
    dir: &std::path::Path,
    map: &mut std::collections::BTreeMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_is_ignored_file_speck_metadata() {
        let patterns: Vec<String> = vec![];
        assert!(is_ignored_file("Speck.toml", std::path::Path::new("Speck.toml"), &patterns));
        assert!(is_ignored_file(".speck_hash.toml", std::path::Path::new(".speck_hash.toml"), &patterns));
    }

    #[test]
    fn test_fmt_is_ignored_file_underscore_md() {
        let patterns: Vec<String> = vec![];
        assert!(is_ignored_file("specs/_draft.md", std::path::Path::new("specs/_draft.md"), &patterns));
        assert!(!is_ignored_file("specs/features/auth.md", std::path::Path::new("specs/features/auth.md"), &patterns));
    }
}
