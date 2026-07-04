use crate::config::SpeckConfig;
use crate::hashes::SpeckHashes;
use crate::helpers;
use std::path::PathBuf;

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

    let features_path = PathBuf::from(config.features_path());
    let technical_path = PathBuf::from(SpeckConfig::technical_path());
    let src_path = PathBuf::from(&config.source_dir);

    let gitignore = helpers::load_gitignore()?;

    let (edited_features, unregistered_features) =
        helpers::scan_directory(&features_path, &stored_hashes.features_hash, &gitignore)?;
    let (edited_technical, unregistered_technical) =
        helpers::scan_directory(&technical_path, &stored_hashes.technical_hash, &gitignore)?;
    let (edited_src, unregistered_src) =
        helpers::scan_directory(&src_path, &stored_hashes.src_hash, &gitignore)?;

    if edited_features.is_empty()
        && edited_technical.is_empty()
        && edited_src.is_empty()
        && unregistered_features.is_empty()
        && unregistered_technical.is_empty()
        && unregistered_src.is_empty()
    {
        println!("there's nothing to do");
        return Ok(());
    }

    print_section(
        "Features (High-level Specifications)",
        &edited_features,
        &unregistered_features,
    );
    print_section(
        "Technicals (Low-level Specifications)",
        &edited_technical,
        &unregistered_technical,
    );
    print_section("Code (Source Code)", &edited_src, &unregistered_src);

    Ok(())
}

fn print_section(title: &str, edited: &[String], unregistered: &[String]) {
    if edited.is_empty() && unregistered.is_empty() {
        return;
    }
    println!("\n{}", title);
    for f in edited {
        println!("  M {}", f);
    }
    for f in unregistered {
        println!("  + {}", f);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hashes;
    use ignore::gitignore::Gitignore;
    use std::collections::BTreeMap;

    #[test]
    fn test_scan_directory_e2e_empty_dir() {
        let dir = std::env::temp_dir().join(format!("speck_status_empty_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let stored: BTreeMap<String, String> = BTreeMap::new();
        let gitignore = Gitignore::empty();
        let (edited, unreg) = crate::test_utils::with_cwd_locked(&dir, || {
            helpers::scan_directory(std::path::Path::new("."), &stored, &gitignore).unwrap()
        });
        assert!(edited.is_empty());
        assert!(unreg.is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_scan_directory_e2e_detects_edited() {
        let dir = std::env::temp_dir().join(format!("speck_status_edit_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("test.md");
        std::fs::write(&file, "initial content").unwrap();

        let mut stored: BTreeMap<String, String> = BTreeMap::new();
        stored.insert("test.md".to_string(), "different_old_hash".to_string());

        let gitignore = Gitignore::empty();
        let (edited, unreg) = crate::test_utils::with_cwd_locked(&dir, || {
            helpers::scan_directory(std::path::Path::new("."), &stored, &gitignore).unwrap()
        });
        assert!(edited.contains(&"test.md".to_string()));
        assert!(unreg.is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_scan_directory_e2e_detects_unregistered() {
        let dir = std::env::temp_dir().join(format!("speck_status_unreg_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("new.md");
        std::fs::write(&file, "new file").unwrap();

        let stored: BTreeMap<String, String> = BTreeMap::new();
        let gitignore = Gitignore::empty();
        let (edited, unreg) = crate::test_utils::with_cwd_locked(&dir, || {
            helpers::scan_directory(std::path::Path::new("."), &stored, &gitignore).unwrap()
        });
        assert!(edited.is_empty());
        assert!(unreg.contains(&"new.md".to_string()));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_scan_directory_e2e_unchanged() {
        let dir =
            std::env::temp_dir().join(format!("speck_status_unchanged_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("stable.md");
        std::fs::write(&file, "unchanged").unwrap();
        let hash = hashes::compute_hash(&file).unwrap();

        let mut stored: BTreeMap<String, String> = BTreeMap::new();
        stored.insert("stable.md".to_string(), hash);
        let gitignore = Gitignore::empty();
        let (edited, unreg) = crate::test_utils::with_cwd_locked(&dir, || {
            helpers::scan_directory(std::path::Path::new("."), &stored, &gitignore).unwrap()
        });
        assert!(edited.is_empty());
        assert!(unreg.is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }
}
