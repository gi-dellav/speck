use crate::config::SpeckConfig;
use crate::hashes::SpeckHashes;
use crate::helpers;
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
        let gitignore = helpers::load_gitignore()?;
        let (edited, _) = helpers::scan_directory(&src_path, &stored_hashes.src_hash, &gitignore)?;
        if !edited.is_empty() {
            return Err(format!(
                "Cannot format: source file '{}' has uncommitted edits. Run `speck apply` first.",
                edited[0]
            )
            .into());
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
        let gitignore = helpers::load_gitignore()?;
        helpers::collect_hashes(&src_path, &mut hashes.src_hash, &gitignore)?;
    }
    hashes.to_file(&hash_path)?;

    println!("Formatted and updated source hashes.");
    Ok(())
}

#[cfg(test)]
mod tests {
    fn run_fmt_in_dir(dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        crate::test_utils::with_cwd_locked(dir, super::run)
    }

    #[test]
    fn test_fmt_fails_without_speck_toml() {
        let dir = std::env::temp_dir().join(format!("speck_fmt_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let result = run_fmt_in_dir(&dir);
        std::fs::remove_dir_all(&dir).ok();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Speck.toml"));
    }

    #[test]
    fn test_fmt_fails_without_hash_file() {
        let dir = std::env::temp_dir().join(format!("speck_fmt_test2_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("Speck.toml"),
            "name = \"test\"\nsource_dir = \"src\"\n",
        )
        .unwrap();
        let result = run_fmt_in_dir(&dir);
        std::fs::remove_dir_all(&dir).ok();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(".speck_hash.toml"));
    }

    #[test]
    fn test_fmt_fails_without_fmt_cmd() {
        let dir = std::env::temp_dir().join(format!("speck_fmt_test3_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("Speck.toml"),
            "name = \"test\"\nsource_dir = \"src\"\n",
        )
        .unwrap();
        let hashes = crate::hashes::SpeckHashes::default();
        hashes.to_file(&dir.join(".speck_hash.toml")).unwrap();
        let result = run_fmt_in_dir(&dir);
        std::fs::remove_dir_all(&dir).ok();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("fmt_cmd"));
    }
}
