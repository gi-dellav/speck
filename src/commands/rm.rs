use crate::hashes::SpeckHashes;
use std::path::Path;

pub fn run(path: String) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = std::env::current_dir()?;
    let hash_path = project_dir.join(".speck_hash.toml");

    if !hash_path.exists() {
        return Err("Not a Speck project: .speck_hash.toml not found".into());
    }

    let target = Path::new(&path);

    if !target.exists() {
        return Err(format!("File not found: {}", path).into());
    }

    if target.is_dir() {
        std::fs::remove_dir_all(target)?;
    } else {
        std::fs::remove_file(target)?;
    }

    let mut hashes = SpeckHashes::from_file(&hash_path)?;
    let rel = target.strip_prefix(&project_dir)?.to_string_lossy().to_string();

    hashes.features_hash.remove(&rel);
    hashes.technical_hash.remove(&rel);
    hashes.src_hash.remove(&rel);

    hashes.to_file(&hash_path)?;
    println!("Removed {}", path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("speck_test_rm_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("Failed to create temp dir");
        dir
    }

    fn cleanup_temp_dir(dir: &std::path::PathBuf) {
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_rm_nonexistent_file_errors() {
        let dir = setup_temp_dir();
        let hash_path = dir.join(".speck_hash.toml");
        let hashes = SpeckHashes::default();
        hashes.to_file(&hash_path).unwrap();
        std::fs::write(&dir.join("Speck.toml"), "name = \"test\"\nsource_dir = \"src\"\n").unwrap();
        let nonexistent = dir.join("nonexistent.txt");
        assert!(!nonexistent.exists());
        cleanup_temp_dir(&dir);
    }
}
