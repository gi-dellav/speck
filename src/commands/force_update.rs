use crate::config::SpeckConfig;
use crate::hashes::SpeckHashes;
use crate::helpers;
use std::path::Path;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = std::env::current_dir()?;
    let config_path = project_dir.join("Speck.toml");
    let hash_path = project_dir.join(".speck_hash.toml");

    if !config_path.exists() {
        return Err("Not a Speck project: Speck.toml not found".into());
    }

    let config = SpeckConfig::from_file(&config_path)?;
    let mut hashes = if hash_path.exists() {
        SpeckHashes::from_file(&hash_path)?
    } else {
        SpeckHashes::default()
    };

    let features_path = Path::new(config.features_path());
    let technical_path = Path::new(SpeckConfig::technical_path());
    let src_path = Path::new(&config.source_dir);
    let gitignore_patterns = helpers::load_gitignore()?;

    hashes.features_hash.clear();
    if features_path.exists() {
        helpers::collect_hashes(features_path, &mut hashes.features_hash, &gitignore_patterns)?;
    }

    hashes.technical_hash.clear();
    if technical_path.exists() {
        helpers::collect_hashes(technical_path, &mut hashes.technical_hash, &gitignore_patterns)?;
    }

    hashes.src_hash.clear();
    if src_path.exists() {
        helpers::collect_hashes(src_path, &mut hashes.src_hash, &gitignore_patterns)?;
    }

    hashes.to_file(&hash_path)?;
    println!("All hashes updated to current files.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::atomic::{AtomicU32, Ordering};
    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn setup_temp_project() -> std::path::PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("speck_fu_test_{}_{}", std::process::id(), id));
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("main.rs"), "fn main() {}").unwrap();
        let features = dir.join("specs/features");
        std::fs::create_dir_all(&features).unwrap();
        std::fs::write(features.join("hello.md"), "# Hello").unwrap();
        let tech = dir.join("specs/technical");
        std::fs::create_dir_all(&tech).unwrap();
        std::fs::write(tech.join("main.rs.md"), "# main").unwrap();
        let config_path = dir.join("Speck.toml");
        let mut f = std::fs::File::create(&config_path).unwrap();
        writeln!(f, "name = \"test\"").unwrap();
        writeln!(f, "source_dir = \"src\"").unwrap();
        dir
    }

    fn run_in_dir(dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        use std::sync::Mutex;
        static CWD_MUTEX: Mutex<()> = Mutex::new(());
        let _lock = CWD_MUTEX.lock().unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir).unwrap();
        let result = run();
        std::env::set_current_dir(&original).unwrap();
        result
    }

    #[test]
    fn test_force_update_creates_hash_file() {
        let dir = setup_temp_project();
        let hash_path = dir.join(".speck_hash.toml");
        assert!(!hash_path.exists());
        run_in_dir(&dir).unwrap();
        assert!(hash_path.exists());
        let content = std::fs::read_to_string(&hash_path).unwrap();
        assert!(content.contains("src/main.rs"));
        assert!(content.contains("specs/features/hello.md"));
        assert!(content.contains("specs/technical/main.rs.md"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_force_update_overwrites_existing_hashes() {
        let dir = setup_temp_project();
        let hash_path = dir.join(".speck_hash.toml");
        std::fs::write(&hash_path, "[src_hash]\n\"src/main.rs\" = \"oldhash\"\n").unwrap();
        run_in_dir(&dir).unwrap();
        let content = std::fs::read_to_string(&hash_path).unwrap();
        assert!(!content.contains("oldhash"));
        assert!(content.contains("src/main.rs"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_force_update_fails_without_speck_toml() {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("speck_fu_no_config_{}_{}", std::process::id(), id));
        std::fs::create_dir_all(&dir).unwrap();
        let result = run_in_dir(&dir);
        std::fs::remove_dir_all(&dir).ok();
        assert!(result.is_err());
    }

    #[test]
    fn test_force_update_ignores_underscore_files() {
        let dir = setup_temp_project();
        std::fs::write(dir.join("specs/_draft.md"), "# draft").unwrap();
        run_in_dir(&dir).unwrap();
        let content = std::fs::read_to_string(dir.join(".speck_hash.toml")).unwrap();
        assert!(!content.contains("_draft.md"));
        std::fs::remove_dir_all(&dir).ok();
    }
}
