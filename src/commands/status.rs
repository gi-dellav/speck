use crate::config::SpeckConfig;
use crate::hashes::{self, SpeckHashes};
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

    let mut edited_features: Vec<String> = Vec::new();
    let mut edited_technical: Vec<String> = Vec::new();
    let mut edited_src: Vec<String> = Vec::new();
    let mut unregistered_features: Vec<String> = Vec::new();
    let mut unregistered_technical: Vec<String> = Vec::new();
    let mut unregistered_src: Vec<String> = Vec::new();

    let gitignore_patterns = helpers::load_gitignore()?;

    // Check features
    check_directory(
        &features_path,
        &stored_hashes.features_hash,
        &mut edited_features,
        &mut unregistered_features,
        &gitignore_patterns,
    )?;

    // Check technical
    check_directory(
        &technical_path,
        &stored_hashes.technical_hash,
        &mut edited_technical,
        &mut unregistered_technical,
        &gitignore_patterns,
    )?;

    // Check source
    check_directory(
        &src_path,
        &stored_hashes.src_hash,
        &mut edited_src,
        &mut unregistered_src,
        &gitignore_patterns,
    )?;

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

    print_section("Features (High-level Specifications)", &edited_features, &unregistered_features);
    print_section("Technicals (Low-level Specifications)", &edited_technical, &unregistered_technical);
    print_section("Code (Source Code)", &edited_src, &unregistered_src);

    Ok(())
}

fn check_directory(
    dir: &PathBuf,
    stored: &std::collections::BTreeMap<String, String>,
    edited: &mut Vec<String>,
    unregistered: &mut Vec<String>,
    gitignore_patterns: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
    {
        let rel = entry.path().strip_prefix(std::env::current_dir()?)?;
        let rel_str = rel.to_string_lossy().to_string();

        if helpers::is_ignored_file(&rel_str, entry.path(), gitignore_patterns) {
            continue;
        }

        if let Some(stored_hash) = stored.get(&rel_str) {
            let current_hash = hashes::compute_hash(&entry.path().to_path_buf())?;
            if &current_hash != stored_hash {
                edited.push(rel_str);
            }
        } else {
            unregistered.push(rel_str);
        }
    }
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
