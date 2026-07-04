use crate::config::SpeckConfig;
use crate::hashes::SpeckHashes;
use crate::helpers;
use dialoguer::{Confirm, Input};

pub fn run(
    name: Option<String>,
    source_path: Option<String>,
    skip_git: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let use_pickers = name.is_none() || source_path.is_none();

    let project_name = if let Some(n) = name {
        n
    } else {
        let dir_name = std::env::current_dir()?
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("speck_project")
            .to_string();
        Input::<String>::new()
            .with_prompt("Project name")
            .default(dir_name)
            .interact_text()?
    };

    let source_dir = if let Some(s) = source_path {
        s
    } else {
        Input::<String>::new()
            .with_prompt("Source directory")
            .default("src".to_string())
            .interact_text()?
    };

    let should_init_git = if skip_git {
        false
    } else if use_pickers {
        let git_exists = std::path::Path::new(".git").exists();
        if !git_exists {
            Confirm::new()
                .with_prompt("No .git found. Initialize git repo?")
                .default(true)
                .interact()?
        } else {
            false
        }
    } else {
        false
    };

    let config = SpeckConfig {
        name: project_name,
        source_dir,
        ..Default::default()
    };

    let project_dir = std::env::current_dir()?;

    // Create directories
    std::fs::create_dir_all(project_dir.join("specs/features"))?;
    std::fs::create_dir_all(project_dir.join("specs/technical"))?;
    std::fs::create_dir_all(project_dir.join(".zerostack/prompts"))?;

    // Write Speck.toml
    config.to_file(&project_dir.join("Speck.toml"))?;

    // Write empty .speck_hash.toml
    let hashes = SpeckHashes::default();
    hashes.to_file(&project_dir.join(".speck_hash.toml"))?;

    // Write default AGENTS.md
    let agents_md = include_str!("../../data/AGENTS.md");
    std::fs::write(project_dir.join("AGENTS.md"), agents_md)?;

    // Write default ARCHITECTURE.md
    let architecture_md = include_str!("../../data/ARCHITECTURE.md");
    std::fs::write(project_dir.join("ARCHITECTURE.md"), architecture_md)?;

    // Write default TECH_STACK.md
    let tech_stack_md = "## Tech Stack\n\n_Describe your tech stack here_\n";
    std::fs::write(project_dir.join("specs/TECH_STACK.md"), tech_stack_md)?;

    // Copy prompts
    let prompts: &[(&str, &str)] = &[
        (
            "speck-feat2tech.md",
            include_str!("../../data/prompts/speck-feat2tech.md"),
        ),
        (
            "speck-tech2code.md",
            include_str!("../../data/prompts/speck-tech2code.md"),
        ),
        (
            "speck-code2tech.md",
            include_str!("../../data/prompts/speck-code2tech.md"),
        ),
        (
            "speck-tech2feat.md",
            include_str!("../../data/prompts/speck-tech2feat.md"),
        ),
        (
            "speck-review.md",
            include_str!("../../data/prompts/speck-review.md"),
        ),
    ];
    for (name, content) in prompts {
        let dest = project_dir.join(".zerostack/prompts").join(name);
        std::fs::write(&dest, content)?;
    }

    if should_init_git {
        let status = std::process::Command::new("git")
            .arg("init")
            .current_dir(&project_dir)
            .status()?;
        if !status.success() {
            eprintln!("Warning: git init failed");
        }
    }

    // Ensure Speck.toml and .speck_hash.toml are not in .gitignore
    helpers::ensure_not_gitignored(&project_dir)?;

    println!("Speck project '{}' initialized successfully.", config.name);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use std::sync::atomic::{AtomicU32, Ordering};
    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn setup_temp_dir() -> std::path::PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("speck_test_{}_{}", std::process::id(), id));
        std::fs::create_dir_all(&dir).expect("Failed to create temp dir");
        dir
    }

    fn cleanup_temp_dir(dir: &std::path::PathBuf) {
        let _ = std::fs::remove_dir_all(dir);
    }

    fn run_init_in_dir(
        dir: &std::path::Path,
        name: &str,
        source_path: &str,
        skip_git: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        crate::test_utils::with_cwd_locked(dir, || {
            super::run(
                Some(name.to_string()),
                Some(source_path.to_string()),
                skip_git,
            )
        })
    }

    #[test]
    fn test_init_creates_directories() {
        let dir = setup_temp_dir();
        run_init_in_dir(&dir, "test", "src", true).unwrap();
        assert!(dir.join("specs/features").exists());
        assert!(dir.join("specs/technical").exists());
        assert!(dir.join(".zerostack/prompts").exists());
        assert!(dir.join("Speck.toml").exists());
        assert!(dir.join(".speck_hash.toml").exists());
        cleanup_temp_dir(&dir);
    }

    #[test]
    fn test_ensure_not_gitignored_removes_entries() {
        let dir = setup_temp_dir();
        let gitignore = dir.join(".gitignore");
        fs::write(&gitignore, "node_modules\nSpeck.toml\n.speck_hash.toml\n").unwrap();
        helpers::ensure_not_gitignored(&dir).unwrap();
        let content = fs::read_to_string(&gitignore).unwrap();
        assert!(!content.contains("Speck.toml"));
        assert!(!content.contains(".speck_hash.toml"));
        assert!(content.contains("node_modules"));
        cleanup_temp_dir(&dir);
    }

    #[test]
    fn test_ensure_not_gitignored_noop_when_missing() {
        let dir = setup_temp_dir();
        let result = helpers::ensure_not_gitignored(&dir);
        assert!(result.is_ok());
        cleanup_temp_dir(&dir);
    }
}
