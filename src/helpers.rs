use crate::hashes;
use std::collections::BTreeMap;
use std::path::Path;

pub fn load_gitignore() -> Result<Vec<String>, Box<dyn std::error::Error>> {
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

pub fn is_ignored_file(rel_str: &str, path: &Path, gitignore_patterns: &[String]) -> bool {
    if rel_str.starts_with("specs/")
        && path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|f| f.starts_with('_') && f.ends_with(".md"))
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

pub fn collect_hashes(
    dir: &Path,
    map: &mut BTreeMap<String, String>,
    gitignore_patterns: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = std::env::current_dir()?;
    let absolute_dir = if dir.is_relative() {
        project_dir.join(dir)
    } else {
        dir.to_path_buf()
    };
    for entry in walkdir::WalkDir::new(&absolute_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
    {
        let rel = entry.path().strip_prefix(&project_dir)?;
        let rel_str = rel.to_string_lossy().to_string();
        if is_ignored_file(&rel_str, entry.path(), gitignore_patterns) {
            continue;
        }
        let hash = hashes::compute_hash(&entry.path().to_path_buf())?;
        map.insert(rel_str, hash);
    }
    Ok(())
}

pub fn ensure_not_gitignored(project_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let gitignore_path = project_dir.join(".gitignore");
    if !gitignore_path.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(&gitignore_path)?;
    let mut modified = false;
    let mut new_lines: Vec<&str> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "Speck.toml" || trimmed == ".speck_hash.toml" {
            modified = true;
            continue;
        }
        new_lines.push(line);
    }

    if modified {
        std::fs::write(&gitignore_path, new_lines.join("\n") + "\n")?;
        eprintln!("Removed Speck.toml and .speck_hash.toml from .gitignore");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ignored_file_speck_metadata() {
        let patterns: Vec<String> = vec![];
        assert!(is_ignored_file(
            "Speck.toml",
            Path::new("Speck.toml"),
            &patterns
        ));
        assert!(is_ignored_file(
            ".speck_hash.toml",
            Path::new(".speck_hash.toml"),
            &patterns
        ));
    }

    #[test]
    fn test_is_ignored_file_underscore_md() {
        let patterns: Vec<String> = vec![];
        assert!(is_ignored_file(
            "specs/_draft.md",
            Path::new("specs/_draft.md"),
            &patterns
        ));
        assert!(!is_ignored_file(
            "specs/features/auth.md",
            Path::new("specs/features/auth.md"),
            &patterns
        ));
    }

    #[test]
    fn test_is_ignored_file_gitignore_pattern() {
        let patterns = vec!["target".to_string(), "*.log".to_string()];
        assert!(is_ignored_file("target/debug/foo", Path::new("target/debug/foo"), &patterns));
        assert!(is_ignored_file("debug.log", Path::new("debug.log"), &patterns));
        assert!(!is_ignored_file("src/main.rs", Path::new("src/main.rs"), &patterns));
    }

    #[test]
    fn test_ensure_not_gitignored_removes_entries() {
        let dir = std::env::temp_dir()
            .join(format!("speck_helper_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let gitignore = dir.join(".gitignore");
        std::fs::write(&gitignore, "node_modules\nSpeck.toml\n.speck_hash.toml\n").unwrap();
        ensure_not_gitignored(&dir).unwrap();
        let content = std::fs::read_to_string(&gitignore).unwrap();
        assert!(!content.contains("Speck.toml"));
        assert!(!content.contains(".speck_hash.toml"));
        assert!(content.contains("node_modules"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_ensure_not_gitignored_noop_when_missing() {
        let dir = std::env::temp_dir()
            .join(format!("speck_helper_test2_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let result = ensure_not_gitignored(&dir);
        assert!(result.is_ok());
        std::fs::remove_dir_all(&dir).ok();
    }
}
