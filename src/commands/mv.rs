use crate::config::SpeckConfig;
use crate::hashes::{self, SpeckHashes};
use std::path::Path;

pub fn run(source: String, dest: String) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = std::env::current_dir()?;
    let hash_path = project_dir.join(".speck_hash.toml");

    if !hash_path.exists() {
        return Err("Not a Speck project: .speck_hash.toml not found".into());
    }

    let src_path = Path::new(&source);
    let dst_path = Path::new(&dest);

    if !src_path.exists() {
        return Err(format!("Source file not found: {}", source).into());
    }

    if dst_path.exists() {
        return Err(format!("Destination already exists: {}", dest).into());
    }
    if let Some(parent) = dst_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::rename(src_path, dst_path)?;

    let mut hashes = SpeckHashes::from_file(&hash_path)?;
    let rel_src = src_path.strip_prefix(&project_dir)?.to_string_lossy().to_string();
    let rel_dst = dst_path.strip_prefix(&project_dir)?.to_string_lossy().to_string();

    hashes.features_hash.remove(&rel_src);
    hashes.technical_hash.remove(&rel_src);
    hashes.src_hash.remove(&rel_src);

    let new_hash = hashes::compute_hash(&dst_path.to_path_buf())?;
    let rel_dst_str = rel_dst;
    if rel_dst_str.starts_with("specs/features") || rel_dst_str.starts_with(&config_features_dir(&project_dir)?) {
        hashes.features_hash.insert(rel_dst_str, new_hash);
    } else if rel_dst_str.starts_with("specs/technical") {
        hashes.technical_hash.insert(rel_dst_str, new_hash);
    } else {
        hashes.src_hash.insert(rel_dst_str, new_hash);
    }

    hashes.to_file(&hash_path)?;
    println!("Moved {} -> {}", source, dest);
    Ok(())
}

fn config_features_dir(project_dir: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let config_path = project_dir.join("Speck.toml");
    if config_path.exists() {
        let config = SpeckConfig::from_file(&config_path)?;
        Ok(config.features_path().to_string())
    } else {
        Ok("specs/features".to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    fn setup_temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("speck_test_mv_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("Failed to create temp dir");
        dir
    }

    fn cleanup_temp_dir(dir: &std::path::PathBuf) {
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_mv_destination_exists_error() {
        let dir = setup_temp_dir();
        fs::write(&dir.join(".speck_hash.toml"), "").unwrap();
        fs::write(&dir.join("Speck.toml"), "name = \"test\"\nsource_dir = \"src\"\n").unwrap();
        fs::write(&dir.join("a.txt"), "a").unwrap();
        fs::write(&dir.join("b.txt"), "b").unwrap();
        let dst = dir.join("b.txt");
        assert!(dst.exists());
        cleanup_temp_dir(&dir);
    }
}
