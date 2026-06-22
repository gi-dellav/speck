use crate::config::SpeckConfig;
use crate::hashes::{self, SpeckHashes};
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
        let gitignore_patterns = helpers::load_gitignore()?;
        for entry in walkdir::WalkDir::new(&src_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
        {
            let rel = entry.path().strip_prefix(&project_dir)?;
            let rel_str = rel.to_string_lossy().to_string();

            if helpers::is_ignored_file(&rel_str, entry.path(), &gitignore_patterns) {
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
        let gitignore_patterns = helpers::load_gitignore()?;
        helpers::collect_hashes(&src_path, &mut hashes.src_hash, &gitignore_patterns)?;
    }
    hashes.to_file(&hash_path)?;

    println!("Formatted and updated source hashes.");
    Ok(())
}
