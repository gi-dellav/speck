use crate::config::SpeckConfig;
use crate::hashes::{self, SpeckHashes};
use crate::helpers;
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
    let rel_src = helpers::project_relative(src_path, &project_dir)?;
    let rel_dst = helpers::project_relative(dst_path, &project_dir)?;

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
    use std::sync::atomic::{AtomicU32, Ordering};
    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn setup_temp_dir() -> std::path::PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("speck_test_mv_{}_{}", std::process::id(), id));
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
        let result = run_mv_in_dir(
            &dir,
            dir.join("a.txt").to_str().unwrap(),
            dir.join("b.txt").to_str().unwrap(),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Destination already exists"));
        cleanup_temp_dir(&dir);
    }

    fn run_mv_in_dir(dir: &std::path::Path, src: &str, dst: &str) -> Result<(), Box<dyn std::error::Error>> {
        let s = src.to_string();
        let d = dst.to_string();
        crate::test_utils::with_cwd_locked(dir, || super::run(s, d))
    }

    #[test]
    fn test_mv_updates_hashes() {
        let dir = setup_temp_dir();
        let src_dir = dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(&dir.join("Speck.toml"), "name = \"test\"\nsource_dir = \"src\"\n").unwrap();
        let mut hashes = crate::hashes::SpeckHashes::default();
        let file_a = src_dir.join("old_name.rs");
        fs::write(&file_a, "fn main() {}").unwrap();
        let hash_a = crate::hashes::compute_hash(&file_a).unwrap();
        hashes.src_hash.insert("src/old_name.rs".to_string(), hash_a);
        hashes.to_file(&dir.join(".speck_hash.toml")).unwrap();

        let src_abs = dir.join("src/old_name.rs");
        let dst_abs = dir.join("src/new_name.rs");
        run_mv_in_dir(&dir, src_abs.to_str().unwrap(), dst_abs.to_str().unwrap()).unwrap();

        assert!(!file_a.exists());
        let file_b = src_dir.join("new_name.rs");
        assert!(file_b.exists());

        let loaded = crate::hashes::SpeckHashes::from_file(&dir.join(".speck_hash.toml")).unwrap();
        assert!(!loaded.src_hash.contains_key("src/old_name.rs"));
        assert!(loaded.src_hash.contains_key("src/new_name.rs"));

        cleanup_temp_dir(&dir);
    }

    #[test]
    fn test_mv_nonexistent_source_errors() {
        let dir = setup_temp_dir();
        fs::write(&dir.join(".speck_hash.toml"), "").unwrap();
        fs::write(&dir.join("Speck.toml"), "name = \"test\"\nsource_dir = \"src\"\n").unwrap();
        let result = run_mv_in_dir(&dir, "nonexistent.txt", "dest.txt");
        assert!(result.is_err());
        cleanup_temp_dir(&dir);
    }
}
