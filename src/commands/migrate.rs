use crate::config::SpeckConfig;
use crate::hashes::SpeckHashes;
use crate::zerostack;
use dialoguer::{Confirm, Input};
use std::path::PathBuf;

pub fn run(custom: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = std::env::current_dir()?;
    let config_path = project_dir.join("Speck.toml");

    if config_path.exists() {
        return Err("Speck.toml already exists. This project is already initialized.".into());
    }

    // Step 1: Generate specs/technical from source code
    eprintln!("Step 1/4: Generating specs/technical from source code...");
    let src_dir = detect_source_dir(&project_dir);
    let msg1 = build_migrate_msg(
        &format!(
            "Analyze the source code in '{}/' and create detailed technical specifications in specs/technical/. \
             Create one markdown file per source file, following the same directory structure. \
             Include technical decisions: HOW it works, WHY it works that way. \
             Keep code snippets to max 5 lines.",
            src_dir
        ),
        &custom,
    );
    std::fs::create_dir_all(project_dir.join("specs/technical"))?;
    zerostack::run_p(
        &[
            "--load-prompt",
            &zerostack::prompt_path("speck-code2tech.md"),
            "--temperature",
            "0",
            "--no-session",
        ],
        &msg1,
    )?;

    // Step 2: Generate specs/features from specs/technical
    eprintln!("Step 2/4: Generating specs/features from specs/technical...");
    let msg2 = build_migrate_msg(
        "Based on the technical specifications in specs/technical/, create high-level feature \
         specifications in specs/features/. Describe only WHAT each feature does, without \
         technical implementation details.",
        &custom,
    );
    std::fs::create_dir_all(project_dir.join("specs/features"))?;
    zerostack::run_p(
        &[
            "--load-prompt",
            &zerostack::prompt_path("speck-tech2feat.md"),
            "--temperature",
            "0.7",
            "--no-session",
        ],
        &msg2,
    )?;

    // Step 3: Create Speck.toml and .speck_hash.toml
    eprintln!("Step 3/4: Initializing Speck configuration...");
    let dir_name = project_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("speck_project");
    let project_name = Input::<String>::new()
        .with_prompt("Project name")
        .default(dir_name.to_string())
        .interact_text()?;
    let source_dir = Input::<String>::new()
        .with_prompt("Source directory")
        .default(src_dir)
        .interact_text()?;

    let config = SpeckConfig {
        name: project_name,
        source_dir,
        ..Default::default()
    };
    config.to_file(&config_path)?;

    // Initialize hashes from current file state
    let mut hashes = SpeckHashes::default();
    let gitignore_patterns = load_gitignore()?;

    let features_path = PathBuf::from(config.features_path());
    let technical_path = PathBuf::from(SpeckConfig::technical_path());
    let src_path = PathBuf::from(&config.source_dir);

    if features_path.exists() {
        collect_hashes(&features_path, &mut hashes.features_hash, &gitignore_patterns)?;
    }
    if technical_path.exists() {
        collect_hashes(&technical_path, &mut hashes.technical_hash, &gitignore_patterns)?;
    }
    if src_path.exists() {
        collect_hashes(&src_path, &mut hashes.src_hash, &gitignore_patterns)?;
    }
    hashes.to_file(&project_dir.join(".speck_hash.toml"))?;

    // Write AGENTS.md snippet
    let agents_snippet = include_str!("../../data/AGENTS.md");
    std::fs::write(project_dir.join("AGENTS.md"), agents_snippet)?;

    // Write ARCHITECTURE.md
    let architecture_md = include_str!("../../data/ARCHITECTURE.md");
    std::fs::write(project_dir.join("ARCHITECTURE.md"), architecture_md)?;

    // Write TECH_STACK.md
    std::fs::write(
        project_dir.join("specs/TECH_STACK.md"),
        "## Tech Stack\n\n_Describe your tech stack here_\n",
    )?;

    // Copy prompts
    std::fs::create_dir_all(project_dir.join(".zerostack/prompts"))?;
    let prompts: &[(&str, &str)] = &[
        ("speck-feat2tech.md", include_str!("../../data/prompts/speck-feat2tech.md")),
        ("speck-tech2code.md", include_str!("../../data/prompts/speck-tech2code.md")),
        ("speck-code2tech.md", include_str!("../../data/prompts/speck-code2tech.md")),
        ("speck-tech2feat.md", include_str!("../../data/prompts/speck-tech2feat.md")),
        ("speck-review.md", include_str!("../../data/prompts/speck-review.md")),
    ];
    for (name, content) in prompts {
        std::fs::write(project_dir.join(".zerostack/prompts").join(name), content)?;
    }

    // Step 4: Ask user to review specs/features
    eprintln!("Step 4/4: Review time!");
    println!("\nSpeck has generated specs in specs/features/ and specs/technical/.");
    println!("Please review the feature specifications before proceeding.\n");
    let ready = Confirm::new()
        .with_prompt("Have you reviewed specs/features/? Ready to continue?")
        .default(false)
        .interact()?;
    if !ready {
        println!("You can review specs/features/ and run `speck apply` when ready.");
    }

    ensure_not_gitignored(&project_dir)?;
    println!("\nMigration complete. Run `speck apply` to ensure specs and code are in sync.");
    Ok(())
}

fn detect_source_dir(project_dir: &std::path::Path) -> String {
    for candidate in &["src", "lib", "app", "source"] {
        if project_dir.join(candidate).exists() {
            return candidate.to_string();
        }
    }
    "src".to_string()
}

fn build_migrate_msg(base: &str, custom: &Option<String>) -> String {
    let mut msg = base.to_string();
    if let Some(c) = custom {
        msg.push_str(&format!("\n\nAdditional instructions: {}", c));
    }
    msg
}

fn load_gitignore() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let gitignore_path = std::env::current_dir()?.join(".gitignore");
    if !gitignore_path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(gitignore_path)?;
    Ok(content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect())
}

fn collect_hashes(
    dir: &PathBuf,
    map: &mut std::collections::BTreeMap<String, String>,
    gitignore_patterns: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = std::env::current_dir()?;
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
    {
        let rel = entry.path().strip_prefix(&project_dir)?;
        let rel_str = rel.to_string_lossy().to_string();
        if is_ignored_file(&rel_str, entry.path(), gitignore_patterns) {
            continue;
        }
        let hash = crate::hashes::compute_hash(&entry.path().to_path_buf())?;
        map.insert(rel_str, hash);
    }
    Ok(())
}

fn is_ignored_file(rel_str: &str, path: &std::path::Path, gitignore_patterns: &[String]) -> bool {
    if rel_str.starts_with("specs/")
        && path.file_name().and_then(|n| n.to_str()).is_some_and(|f| f.starts_with('_') && f.ends_with(".md"))
    {
        return true;
    }
    if rel_str == "Speck.toml" || rel_str == ".speck_hash.toml" {
        return true;
    }
    for pattern in gitignore_patterns {
        if pattern.contains('*') {
            let regex_pattern = pattern.replace('.', "\\.").replace('*', ".*").replace('?', ".");
            if let Ok(re) = regex::Regex::new(&format!("^{}$", regex_pattern))
                && re.is_match(rel_str)
            {
                return true;
            }
        } else if rel_str.starts_with(pattern) {
            return true;
        }
    }
    false
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
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_source_dir_detects_src() {
        let dir = std::env::temp_dir().join(format!("speck_migrate_test_src_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("src")).unwrap();
        assert_eq!(detect_source_dir(&dir), "src");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_detect_source_dir_detects_lib() {
        let dir = std::env::temp_dir().join(format!("speck_migrate_test_lib_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        assert_eq!(detect_source_dir(&dir), "lib");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_detect_source_dir_fallback() {
        let dir = std::env::temp_dir().join(format!("speck_migrate_test_none_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(detect_source_dir(&dir), "src");
        std::fs::remove_dir_all(&dir).ok();
    }
}
