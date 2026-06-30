use crate::config::SpeckConfig;
use crate::hashes::SpeckHashes;
use crate::helpers;
use std::path::Path;

pub fn run(hard: bool, rebuild: bool, full: bool, always_yes: bool, always_no: bool) -> Result<(), Box<dyn std::error::Error>> {
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
            let proceed = helpers::confirm(
                always_yes,
                always_no,
                &format!("This will delete all source code in {}/. Proceed?", config.source_dir),
                false,
            )?;
            if !proceed {
                println!("Aborted.");
                return Ok(());
            }
            std::fs::remove_dir_all(src_dir)?;
            println!("Removed source directory: {}", config.source_dir);
        }
    }

    if full {
        let technical_dir = Path::new(SpeckConfig::technical_path());
        if technical_dir.exists() {
            let proceed = helpers::confirm(
                always_yes,
                always_no,
                "This will delete specs/technical/. Proceed?",
                false,
            )?;
            if !proceed {
                println!("Aborted.");
                return Ok(());
            }
            std::fs::remove_dir_all(technical_dir)?;
            println!("Removed specs/technical/");
        }
    }

    let hashes = SpeckHashes::default();
    hashes.to_file(&hash_path)?;
    println!("All stored hashes removed.");

    if rebuild {
        println!("Running speck apply to rebuild...");
        crate::commands::apply::run(None, false, false, false, false, false, None)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::atomic::{AtomicU32, Ordering};
    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn setup_temp_project() -> (std::path::PathBuf, std::path::PathBuf) {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("speck_reset_test_{}_{}", std::process::id(), id));
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("main.rs"), "fn main() {}").unwrap();
        std::fs::create_dir_all(dir.join("specs/technical")).unwrap();
        std::fs::write(dir.join("specs/technical/main.rs.md"), "# main.rs").unwrap();
        let config_path = dir.join("Speck.toml");
        let mut f = std::fs::File::create(&config_path).unwrap();
        writeln!(f, "name = \"test\"").unwrap();
        writeln!(f, "source_dir = \"src\"").unwrap();
        let hash_path = dir.join(".speck_hash.toml");
        std::fs::write(&hash_path, "[src_hash]\n\"src/main.rs\" = \"abc123\"\n").unwrap();
        (dir, config_path)
    }

    fn run_in_dir(dir: &std::path::Path, hard: bool, rebuild: bool, full: bool, always_yes: bool, always_no: bool) -> Result<(), Box<dyn std::error::Error>> {
        crate::test_utils::with_cwd_locked(dir, || run(hard, rebuild, full, always_yes, always_no))
    }

    #[test]
    fn test_reset_hard_removes_src() {
        let (dir, _) = setup_temp_project();
        assert!(dir.join("src").exists());
        run_in_dir(&dir, true, false, false, true, false).unwrap();
        assert!(!dir.join("src").exists());
        let content = std::fs::read_to_string(dir.join(".speck_hash.toml")).unwrap();
        assert!(!content.contains("abc123"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_reset_full_removes_technical() {
        let (dir, _) = setup_temp_project();
        assert!(dir.join("specs/technical").exists());
        run_in_dir(&dir, false, false, true, true, false).unwrap();
        assert!(!dir.join("specs/technical").exists());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_reset_clears_hashes() {
        let (dir, _) = setup_temp_project();
        run_in_dir(&dir, false, false, false, false, false).unwrap();
        let content = std::fs::read_to_string(dir.join(".speck_hash.toml")).unwrap();
        assert!(!content.contains("abc123"));
        assert!(dir.join("src").exists());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_reset_fails_without_speck_toml() {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("speck_reset_no_config_{}_{}", std::process::id(), id));
        std::fs::create_dir_all(&dir).unwrap();
        let result = run_in_dir(&dir, false, false, false, false, false);
        std::fs::remove_dir_all(&dir).ok();
        assert!(result.is_err());
    }

    #[test]
    fn test_reset_full_and_hard() {
        let (dir, _) = setup_temp_project();
        run_in_dir(&dir, true, false, true, true, false).unwrap();
        assert!(!dir.join("src").exists());
        assert!(!dir.join("specs/technical").exists());
        std::fs::remove_dir_all(&dir).ok();
    }
}
