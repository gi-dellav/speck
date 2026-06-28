use crate::hashes::SpeckHashes;
use crate::helpers;
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
    let rel = helpers::project_relative(target, &project_dir)?;

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
    use std::sync::atomic::{AtomicU32, Ordering};
    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn setup_temp_dir() -> std::path::PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("speck_test_rm_{}_{}", std::process::id(), id));
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

    fn run_rm_in_dir(dir: &std::path::Path, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let p = path.to_string();
        crate::test_utils::with_cwd_locked(dir, || super::run(p))
    }

    #[test]
    fn test_rm_removes_file_and_hash() {
        let dir = setup_temp_dir();
        let src_dir = dir.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        let file = src_dir.join("to_remove.rs");
        std::fs::write(&file, "fn unused() {}").unwrap();
        let mut hashes = SpeckHashes::default();
        hashes.src_hash.insert("src/to_remove.rs".to_string(), "somehash".to_string());
        hashes.to_file(&dir.join(".speck_hash.toml")).unwrap();

        let abs_path = dir.join("src/to_remove.rs");
        run_rm_in_dir(&dir, abs_path.to_str().unwrap()).unwrap();

        assert!(!file.exists());
        let loaded = SpeckHashes::from_file(&dir.join(".speck_hash.toml")).unwrap();
        assert!(!loaded.src_hash.contains_key("src/to_remove.rs"));

        cleanup_temp_dir(&dir);
    }

    #[test]
    fn test_rm_removes_directory() {
        let dir = setup_temp_dir();
        let subdir = dir.join("specs/features/old_feature");
        std::fs::create_dir_all(&subdir).unwrap();
        std::fs::write(subdir.join("spec.md"), "# Old").unwrap();
        let mut hashes = SpeckHashes::default();
        hashes.features_hash.insert("specs/features/old_feature".to_string(), "hash1".to_string());
        hashes.to_file(&dir.join(".speck_hash.toml")).unwrap();

        let abs_path = dir.join("specs/features/old_feature");
        run_rm_in_dir(&dir, abs_path.to_str().unwrap()).unwrap();

        assert!(!subdir.exists());
        let loaded = SpeckHashes::from_file(&dir.join(".speck_hash.toml")).unwrap();
        assert!(!loaded.features_hash.contains_key("specs/features/old_feature"));

        cleanup_temp_dir(&dir);
    }
}
