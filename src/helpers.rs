use crate::hashes;
use ignore::gitignore::Gitignore;
use std::collections::BTreeMap;
use std::path::Path;

pub fn load_gitignore() -> Result<Gitignore, Box<dyn std::error::Error>> {
    let project_dir = std::env::current_dir()?;
    let gitignore_path = project_dir.join(".gitignore");
    if !gitignore_path.exists() {
        return Ok(Gitignore::empty());
    }
    let (gitignore, err) = Gitignore::new(&gitignore_path);
    if let Some(e) = err {
        eprintln!("Warning: error reading .gitignore: {}", e);
    }
    Ok(gitignore)
}

pub fn is_ignored_file(rel_str: &str, path: &Path, gitignore: &Gitignore) -> bool {
    if rel_str == "Speck.toml" || rel_str == ".speck_hash.toml" {
        return true;
    }
    if rel_str.starts_with("specs/")
        && path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|f| f.starts_with('_') && f.ends_with(".md"))
    {
        return true;
    }
    if gitignore.matched(path, false).is_ignore() {
        return true;
    }
    let mut ancestor = path.parent();
    while let Some(dir) = ancestor {
        if gitignore.matched(dir, true).is_ignore() {
            return true;
        }
        ancestor = dir.parent();
    }
    false
}

pub fn collect_hashes(
    dir: &Path,
    map: &mut BTreeMap<String, String>,
    gitignore: &Gitignore,
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
        if is_ignored_file(&rel_str, entry.path(), gitignore) {
            continue;
        }
        let hash = hashes::compute_hash(&entry.path().to_path_buf())?;
        map.insert(rel_str, hash);
    }
    Ok(())
}

pub fn scan_directory(
    dir: &Path,
    stored: &BTreeMap<String, String>,
    gitignore: &Gitignore,
) -> Result<(Vec<String>, Vec<String>), Box<dyn std::error::Error>> {
    let mut edited = Vec::new();
    let mut unregistered = Vec::new();
    let project_dir = std::env::current_dir()?;
    let absolute_dir = if dir.is_relative() {
        project_dir.join(dir)
    } else {
        dir.to_path_buf()
    };
    if !absolute_dir.exists() {
        return Ok((edited, unregistered));
    }
    for entry in walkdir::WalkDir::new(&absolute_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
    {
        let rel = entry.path().strip_prefix(&project_dir)?;
        let rel_str = rel.to_string_lossy().to_string();
        if is_ignored_file(&rel_str, entry.path(), gitignore) {
            continue;
        }
        if let Some(stored_hash) = stored.get(&rel_str) {
            let current = hashes::compute_hash(&entry.path().to_path_buf())?;
            if current != *stored_hash {
                edited.push(rel_str);
            }
        } else {
            unregistered.push(rel_str);
        }
    }
    Ok((edited, unregistered))
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

/// Resolves `path` (relative or absolute, possibly reached through symlinks)
/// into a project-relative key suitable for the hash maps.
///
/// A naive `path.strip_prefix(current_dir())` is fragile: on some platforms
/// `current_dir()` is canonicalized (e.g. macOS resolves `/tmp` ->
/// `/private/tmp`), so it fails to match a path the caller passed in
/// un-canonicalized form, or a relative path. This canonicalizes both sides
/// first. It canonicalizes the parent directory and re-attaches the file name
/// rather than the path itself, so it still works when the target has just
/// been deleted (`speck rm`) or moved (`speck mv`).
pub fn project_relative(
    path: &Path,
    project_dir: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_dir.join(path)
    };

    let canonical = match (abs.parent(), abs.file_name()) {
        (Some(parent), Some(name)) => parent.canonicalize()?.join(name),
        _ => abs.canonicalize()?,
    };

    let project_canonical = project_dir.canonicalize()?;
    let rel = canonical
        .strip_prefix(&project_canonical)
        .map_err(|_| format!("path {} is outside the project directory", path.display()))?;
    Ok(rel.to_string_lossy().to_string())
}

pub fn confirm(
    always_yes: bool,
    always_no: bool,
    prompt: &str,
    default: bool,
) -> Result<bool, Box<dyn std::error::Error>> {
    if always_yes {
        return Ok(true);
    }
    if always_no {
        return Ok(false);
    }
    dialoguer::Confirm::new()
        .with_prompt(prompt)
        .default(default)
        .interact()
        .map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_cwd_locked<T>(dir: &Path, f: impl FnOnce() -> T) -> T {
        crate::test_utils::with_cwd_locked(dir, f)
    }

    #[test]
    fn test_project_relative_absolute_and_relative() {
        let dir = std::env::temp_dir()
            .join(format!("speck_relctx_test_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("src/foo.rs"), "fn foo() {}").unwrap();
        // current_dir() may canonicalize symlinks (e.g. /tmp -> /private/tmp);
        // project_relative must still produce the same key for an absolute path
        // passed in un-canonicalized form and for a relative path.
        let project = std::fs::canonicalize(&dir).unwrap();
        let abs = dir.join("src/foo.rs");
        assert_eq!(project_relative(&abs, &project).unwrap(), "src/foo.rs");
        assert_eq!(
            project_relative(Path::new("src/foo.rs"), &project).unwrap(),
            "src/foo.rs"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_project_relative_works_after_deletion() {
        let dir = std::env::temp_dir()
            .join(format!("speck_reldel_test_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("src")).unwrap();
        let project = std::fs::canonicalize(&dir).unwrap();
        let file = dir.join("src/gone.rs");
        std::fs::write(&file, "x").unwrap();
        std::fs::remove_file(&file).unwrap();
        // Parent still exists, so the key resolves even though the file is gone.
        assert_eq!(project_relative(&file, &project).unwrap(), "src/gone.rs");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_project_relative_outside_project_errors() {
        let dir = std::env::temp_dir()
            .join(format!("speck_relout_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let project = std::fs::canonicalize(&dir).unwrap();
        let result = project_relative(Path::new("/etc/hosts"), &project);
        assert!(result.is_err());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_is_ignored_file_speck_metadata() {
        let gitignore = Gitignore::empty();
        assert!(is_ignored_file(
            "Speck.toml",
            Path::new("Speck.toml"),
            &gitignore
        ));
        assert!(is_ignored_file(
            ".speck_hash.toml",
            Path::new(".speck_hash.toml"),
            &gitignore
        ));
    }

    #[test]
    fn test_is_ignored_file_underscore_md() {
        let gitignore = Gitignore::empty();
        assert!(is_ignored_file(
            "specs/_draft.md",
            Path::new("specs/_draft.md"),
            &gitignore
        ));
        assert!(!is_ignored_file(
            "specs/features/auth.md",
            Path::new("specs/features/auth.md"),
            &gitignore
        ));
    }

    #[test]
    fn test_is_ignored_file_gitignore_pattern() {
        let dir = std::env::temp_dir()
            .join(format!("speck_gi_test_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("target/debug")).unwrap();
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join(".gitignore"), "target\n*.log\n").unwrap();
        let (gi, err) = Gitignore::new(&dir.join(".gitignore"));
        if let Some(e) = err {
            panic!("Gitignore error: {}", e);
        }
        std::fs::write(dir.join("target/debug/foo"), "build artifact").unwrap();
        std::fs::write(dir.join("debug.log"), "log content").unwrap();
        std::fs::write(dir.join("src/main.rs"), "fn main() {}").unwrap();

        let (_edited, unreg) = with_cwd_locked(&dir, || {
            let stored = std::collections::BTreeMap::new();
            scan_directory(Path::new("."), &stored, &gi).unwrap()
        });
        assert!(unreg.contains(&"src/main.rs".to_string()));
        assert!(!unreg.contains(&"target/debug/foo".to_string()));
        assert!(!unreg.contains(&"debug.log".to_string()));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_scan_directory_detects_edited_and_unregistered() {
        let dir = std::env::temp_dir()
            .join(format!("speck_scan_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file_edited = dir.join("edited.txt");
        let file_new = dir.join("new.txt");
        std::fs::write(&file_edited, "modified content").unwrap();
        std::fs::write(&file_new, "new file").unwrap();

        let mut stored = BTreeMap::new();
        stored.insert("edited.txt".to_string(), "old_hash".to_string());
        let gitignore = Gitignore::empty();

        let (edited, unreg) = with_cwd_locked(&dir, || scan_directory(Path::new("."), &stored, &gitignore).unwrap());
        assert!(edited.contains(&"edited.txt".to_string()));
        assert!(unreg.contains(&"new.txt".to_string()));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_scan_directory_unchanged() {
        let dir = std::env::temp_dir()
            .join(format!("speck_scan_uc_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("stable.txt");
        std::fs::write(&file, "unchanged").unwrap();
        let hash = hashes::compute_hash(&file).unwrap();

        let mut stored = BTreeMap::new();
        stored.insert("stable.txt".to_string(), hash);
        let gitignore = Gitignore::empty();

        let (edited, unreg) = with_cwd_locked(&dir, || scan_directory(Path::new("."), &stored, &gitignore).unwrap());
        assert!(edited.is_empty());
        assert!(unreg.is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_scan_directory_skips_ignored() {
        let dir = std::env::temp_dir()
            .join(format!("speck_scan_ign_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("specs")).unwrap();
        let file = dir.join("specs/_draft.md");
        std::fs::write(&file, "# draft").unwrap();
        let stored = BTreeMap::new();
        let gitignore = Gitignore::empty();

        let (edited, unreg) = with_cwd_locked(&dir, || scan_directory(Path::new("."), &stored, &gitignore).unwrap());
        assert!(edited.is_empty());
        assert!(unreg.is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_scan_directory_empty_dir() {
        let dir = std::env::temp_dir()
            .join(format!("speck_scan_empty_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let stored = BTreeMap::new();
        let gitignore = Gitignore::empty();

        let (edited, unreg) = with_cwd_locked(&dir, || scan_directory(Path::new("."), &stored, &gitignore).unwrap());
        assert!(edited.is_empty());
        assert!(unreg.is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_scan_directory_missing_dir() {
        let stored = BTreeMap::new();
        let gitignore = Gitignore::empty();
        let (edited, unreg) = scan_directory(
            Path::new("/nonexistent_dir"),
            &stored,
            &gitignore,
        )
        .unwrap();
        assert!(edited.is_empty());
        assert!(unreg.is_empty());
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
