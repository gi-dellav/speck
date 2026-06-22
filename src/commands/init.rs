use crate::config::SpeckConfig;
use crate::hashes::SpeckHashes;
use dialoguer::{Confirm, Input};

pub fn run(name: Option<String>, source_path: Option<String>, skip_git: bool) -> Result<(), Box<dyn std::error::Error>> {
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
        ("speck-feat2tech.md", include_str!("../../data/prompts/speck-feat2tech.md")),
        ("speck-tech2code.md", include_str!("../../data/prompts/speck-tech2code.md")),
        ("speck-code2tech.md", include_str!("../../data/prompts/speck-code2tech.md")),
        ("speck-tech2feat.md", include_str!("../../data/prompts/speck-tech2feat.md")),
        ("speck-review.md", include_str!("../../data/prompts/speck-review.md")),
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
    ensure_not_gitignored(&project_dir)?;

    println!("Speck project '{}' initialized successfully.", config.name);
    Ok(())
}

fn ensure_not_gitignored(project_dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let gitignore_path = project_dir.join(".gitignore");
    if !gitignore_path.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(&gitignore_path)?;
    let mut modified = false;
    let mut new_lines: Vec<&str> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "Speck.toml" || trimmed == ".speck_hash.toml" {
            modified = true;
            continue;
        }
        new_lines.push(line);
    }

    if modified {
        std::fs::write(&gitignore_path, new_lines.join("\n") + "\n")?;
        eprintln!("Removed Speck.toml and .speck_hash.toml from .gitignore");
    }
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

    #[test]
    fn test_init_creates_directories() {
        let dir = setup_temp_dir();
        std::fs::create_dir_all(dir.join("specs/features")).unwrap();
        std::fs::create_dir_all(dir.join("specs/technical")).unwrap();
        assert!(dir.join("specs/features").exists());
        assert!(dir.join("specs/technical").exists());
        cleanup_temp_dir(&dir);
    }

    #[test]
    fn test_ensure_not_gitignored_removes_entries() {
        let dir = setup_temp_dir();
        let gitignore = dir.join(".gitignore");
        fs::write(&gitignore, "node_modules\nSpeck.toml\n.speck_hash.toml\n").unwrap();
        ensure_not_gitignored(&dir).unwrap();
        let content = fs::read_to_string(&gitignore).unwrap();
        assert!(!content.contains("Speck.toml"));
        assert!(!content.contains(".speck_hash.toml"));
        assert!(content.contains("node_modules"));
        cleanup_temp_dir(&dir);
    }

    #[test]
    fn test_ensure_not_gitignored_noop_when_missing() {
        let dir = setup_temp_dir();
        let result = ensure_not_gitignored(&dir);
        assert!(result.is_ok());
        cleanup_temp_dir(&dir);
    }
}
