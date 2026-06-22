use crate::config::SpeckConfig;
use crate::hashes::SpeckHashes;
use std::path::Path;

pub fn run(hard: bool, rebuild: bool, full: bool) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = std::env::current_dir()?;
    let config_path = project_dir.join("Speck.toml");
    let hash_path = project_dir.join(".speck_hash.toml");

    if !config_path.exists() {
        return Err("Not a Speck project: Speck.toml not found".into());
    }

    let config = SpeckConfig::from_file(&config_path)?;

    if hard {
        let src_dir = Path::new(&config.source_dir);
        if src_dir.exists() {
            std::fs::remove_dir_all(src_dir)?;
            println!("Removed source directory: {}", config.source_dir);
        }
    }

    if full {
        let technical_dir = Path::new(SpeckConfig::technical_path());
        if technical_dir.exists() {
            std::fs::remove_dir_all(technical_dir)?;
            println!("Removed specs/technical/");
        }
    }

    let hashes = SpeckHashes::default();
    hashes.to_file(&hash_path)?;
    println!("All stored hashes removed.");

    if rebuild {
        println!("Running speck apply to rebuild...");
        // ponytail: call apply logic directly when apply is implemented
        println!("rebuild: apply not yet implemented");
    }

    Ok(())
}
