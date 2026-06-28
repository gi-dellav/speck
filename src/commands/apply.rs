use crate::config::SpeckConfig;
use crate::hashes::SpeckHashes;
use crate::helpers;
use crate::zerostack;
use dialoguer::Confirm;
use std::path::{Path, PathBuf};

pub fn run(
    custom: Option<String>,
    only_direct: bool,
    only_inverse: bool,
    update_features: bool,
    prefer_code: bool,
    prefer_specs: bool,
    gen_temperature: Option<f64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = std::env::current_dir()?;
    let config_path = project_dir.join("Speck.toml");
    let hash_path = project_dir.join(".speck_hash.toml");

    if !config_path.exists() {
        return Err("Not a Speck project: Speck.toml not found".into());
    }
    if !hash_path.exists() {
        return Err("Not a Speck project: .speck_hash.toml not found".into());
    }

    let config = SpeckConfig::from_file(&config_path)?;
    let stored_hashes = SpeckHashes::from_file(&hash_path)?;

    let features_path = PathBuf::from(config.features_path());
    let technical_path = PathBuf::from(SpeckConfig::technical_path());
    let src_path = PathBuf::from(&config.source_dir);

    let gitignore = helpers::load_gitignore()?;

    // Snapshot edited + unregistered files before any changes
    let (edited_src, unreg_src) =
        helpers::scan_directory(&src_path, &stored_hashes.src_hash, &gitignore)?;
    let (edited_tech, unreg_tech) =
        helpers::scan_directory(&technical_path, &stored_hashes.technical_hash, &gitignore)?;
    let (edited_feat, unreg_feat) =
        helpers::scan_directory(&features_path, &stored_hashes.features_hash, &gitignore)?;

    let all_edited_src: Vec<String> = edited_src.iter().chain(unreg_src.iter()).cloned().collect();
    let all_edited_tech: Vec<String> = edited_tech.iter().chain(unreg_tech.iter()).cloned().collect();
    let all_edited_feat: Vec<String> = edited_feat.iter().chain(unreg_feat.iter()).cloned().collect();

    if all_edited_src.is_empty() && all_edited_tech.is_empty() && all_edited_feat.is_empty() {
        println!("there's nothing to do");
        return Ok(());
    }

    // Conflict detection
    let conflicts = detect_conflicts(&all_edited_src, &all_edited_tech, &config.source_dir);
    let resolved_src = if !conflicts.is_empty() {
        resolve_conflicts(&conflicts, prefer_code, prefer_specs)?
    } else {
        all_edited_src.clone()
    };
    let resolved_tech = if !conflicts.is_empty() {
        let conflict_tech: Vec<String> = conflicts.iter().map(|(_, tech)| tech.clone()).collect();
        let keep_code: Vec<String> = resolved_src.iter()
            .filter(|s| conflicts.iter().any(|(src, _)| src == *s))
            .cloned()
            .collect();
        all_edited_tech
            .iter()
            .filter(|t| !conflict_tech.contains(t) || !keep_code.iter().any(|s| {
                let counterpart = src_to_tech_counterpart(s, &config.source_dir);
                counterpart.as_deref() == Some(t.as_str())
            }))
            .cloned()
            .collect()
    } else {
        all_edited_tech.clone()
    };

    let run_inverse = !only_direct;
    let run_direct = !only_inverse;

    // Step 1: code → specs/technical
    if run_inverse && !resolved_src.is_empty() {
        eprintln!("Step 1/4: Updating specs/technical from source code...");
        let msg = build_message(
            "Update the technical specifications to match the edited source code files.",
            &format!(
                "These source files were edited:\n{}\n\nUpdate the corresponding files in specs/technical/.",
                resolved_src.join("\n")
            ),
            &custom,
        );
        zerostack::run_p_streamed(
            &[
                "--load-prompt",
                &zerostack::prompt_name("speck-code2tech.md"),
                "--temperature",
                "0",
                "--no-session",
            ],
            &msg,
            config.model.as_deref(),
        )
        .map_err(|e| format!("Step 1/4 (code → specs/technical) failed: {}", e))?;
        save_hashes(&hash_path, &features_path, &technical_path, &src_path, &gitignore)?;
    }

    // Step 2: specs/technical → specs/features (only with --update-features)
    if run_inverse && update_features && (!resolved_src.is_empty() || !resolved_tech.is_empty()) {
        eprintln!("Step 2/4: Updating specs/features from specs/technical...");
        let msg = build_message(
            "Update the high-level feature specifications to reflect changes in the technical specifications.",
            &format!(
                "These technical spec files were updated:\n{}\n\nUpdate specs/features/ accordingly.",
                resolved_tech.join("\n")
            ),
            &custom,
        );
        zerostack::run_p_streamed(
            &[
                "--load-prompt",
                &zerostack::prompt_name("speck-tech2feat.md"),
                "--temperature",
                "0",
                "--no-session",
            ],
            &msg,
            config.model.as_deref(),
        )
        .map_err(|e| format!("Step 2/4 (specs/technical → specs/features) failed: {}", e))?;
        save_hashes(&hash_path, &features_path, &technical_path, &src_path, &gitignore)?;
    }

    // Step 3: specs/features → specs/technical
    if run_direct && !all_edited_feat.is_empty() {
        eprintln!("Step 3/4: Updating specs/technical from specs/features...");
        let msg = build_message(
            "Update the technical specifications to implement the edited feature specifications.",
            &format!(
                "These feature spec files were edited:\n{}\n\nUpdate specs/technical/ accordingly.",
                all_edited_feat.join("\n")
            ),
            &custom,
        );
        zerostack::run_p_streamed(
            &[
                "--load-prompt",
                &zerostack::prompt_name("speck-feat2tech.md"),
                "--temperature",
                "0.7",
                "--no-session",
            ],
            &msg,
            config.model.as_deref(),
        )
        .map_err(|e| format!("Step 3/4 (specs/features → specs/technical) failed: {}", e))?;
        save_hashes(&hash_path, &features_path, &technical_path, &src_path, &gitignore)?;
    }

    // Step 4: specs/technical + specs/features → source code
    let tech_or_feat_edited = !resolved_tech.is_empty() || !all_edited_feat.is_empty();
    if run_direct && tech_or_feat_edited {
        eprintln!("Step 4/4: Updating source code from specifications...");
        let mut relevant: Vec<String> = Vec::new();
        relevant.extend(resolved_tech.clone());
        relevant.extend(all_edited_feat.clone());
        let msg = build_message(
            "Update the source code to match the edited specification files.",
            &format!(
                "These spec files were edited:\n{}\n\nUpdate the source code in {}/ accordingly.",
                relevant.join("\n"),
                config.source_dir
            ),
            &custom,
        );
        let prompt_name = zerostack::prompt_name("speck-tech2code.md");
        let mut args: Vec<&str> = vec![
            "--load-prompt",
            &prompt_name,
            "--no-session",
        ];
        let temp_str;
        if let Some(t) = gen_temperature {
            temp_str = t.to_string();
            args.push("--temperature");
            args.push(&temp_str);
        }
        zerostack::run_p_streamed(&args, &msg, config.model.as_deref())
            .map_err(|e| format!("Step 4/4 (specifications → source code) failed: {}", e))?;
    }

    // Final hash save
    save_hashes(&hash_path, &features_path, &technical_path, &src_path, &gitignore)?;

    println!("Apply complete.");
    Ok(())
}

fn save_hashes(
    hash_path: &PathBuf,
    features_path: &Path,
    technical_path: &Path,
    src_path: &Path,
    gitignore: &ignore::gitignore::Gitignore,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut hashes = SpeckHashes::default();
    if features_path.exists() {
        helpers::collect_hashes(features_path, &mut hashes.features_hash, gitignore)?;
    }
    if technical_path.exists() {
        helpers::collect_hashes(technical_path, &mut hashes.technical_hash, gitignore)?;
    }
    if src_path.exists() {
        helpers::collect_hashes(src_path, &mut hashes.src_hash, gitignore)?;
    }
    hashes.to_file(hash_path)?;
    Ok(())
}

fn build_message(intro: &str, detail: &str, custom: &Option<String>) -> String {
    let mut msg = format!("{}\n\n{}", intro, detail);
    if let Some(c) = custom {
        msg.push_str(&format!("\n\nAdditional instructions: {}", c));
    }
    msg
}

fn detect_conflicts(src_files: &[String], tech_files: &[String], source_dir: &str) -> Vec<(String, String)> {
    let mut conflicts = Vec::new();
    for sf in src_files {
        if let Some(tf) = src_to_tech_counterpart(sf, source_dir)
            && tech_files.contains(&tf)
        {
            conflicts.push((sf.clone(), tf));
        }
    }
    conflicts
}

fn src_to_tech_counterpart(src_file: &str, source_dir: &str) -> Option<String> {
    let prefix = format!("{}/", source_dir);
    if src_file.starts_with(&prefix) {
        let remainder = src_file.strip_prefix(&prefix)?;
        let tech_file = format!("specs/technical/{}", remainder);
        Some(tech_file)
    } else {
        None
    }
}

fn resolve_conflicts(
    conflicts: &[(String, String)],
    prefer_code: bool,
    prefer_specs: bool,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    if prefer_code && prefer_specs {
        return Err("Cannot use both --prefer-code and --prefer-specs".into());
    }
    if prefer_code {
        return Ok(conflicts.iter().map(|(s, _)| s.clone()).collect());
    }
    if prefer_specs {
        return Ok(Vec::new());
    }

    let mut keep_src = Vec::new();
    for (src, tech) in conflicts {
        let prompt = format!(
            "Conflict: both '{}' and '{}' were edited. Keep Code version?",
            src, tech
        );
        let keep = Confirm::new()
            .with_prompt(&prompt)
            .default(true)
            .interact()?;
        if keep {
            keep_src.push(src.clone());
        }
    }
    Ok(keep_src)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_src_to_tech_counterpart() {
        assert_eq!(
            src_to_tech_counterpart("src/main.rs", "src"),
            Some("specs/technical/main.rs".to_string())
        );
        assert_eq!(
            src_to_tech_counterpart("src/lib/models/user.py", "src"),
            Some("specs/technical/lib/models/user.py".to_string())
        );
        assert_eq!(
            src_to_tech_counterpart("specs/features/auth.md", "src"),
            None
        );
    }

    #[test]
    fn test_detect_conflicts() {
        let src = vec!["src/main.rs".to_string()];
        let tech = vec!["specs/technical/main.rs".to_string()];
        let conflicts = detect_conflicts(&src, &tech, "src");
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].0, "src/main.rs");
        assert_eq!(conflicts[0].1, "specs/technical/main.rs");
    }

    #[test]
    fn test_detect_no_conflicts() {
        let src = vec!["src/main.rs".to_string()];
        let tech = vec!["specs/technical/lib.rs".to_string()];
        let conflicts = detect_conflicts(&src, &tech, "src");
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_build_message_with_custom() {
        let msg = build_message(
            "Intro",
            "Details here",
            &Some("Custom note".to_string()),
        );
        assert!(msg.contains("Intro"));
        assert!(msg.contains("Details here"));
        assert!(msg.contains("Custom note"));
    }

    #[test]
    fn test_build_message_without_custom() {
        let msg = build_message(
            "Intro",
            "Details",
            &None,
        );
        assert!(msg.contains("Intro"));
        assert!(msg.contains("Details"));
        assert!(!msg.contains("Additional instructions"));
    }

    #[test]
    fn test_resolve_conflicts_prefer_code() {
        let conflicts = vec![
            ("src/main.rs".to_string(), "specs/technical/main.rs".to_string()),
        ];
        let result = resolve_conflicts(&conflicts, true, false).unwrap();
        assert_eq!(result, vec!["src/main.rs"]);
    }

    #[test]
    fn test_resolve_conflicts_prefer_specs() {
        let conflicts = vec![
            ("src/main.rs".to_string(), "specs/technical/main.rs".to_string()),
        ];
        let result = resolve_conflicts(&conflicts, false, true).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_resolve_conflicts_both_flags_error() {
        let conflicts = vec![
            ("src/main.rs".to_string(), "specs/technical/main.rs".to_string()),
        ];
        let result = resolve_conflicts(&conflicts, true, true);
        assert!(result.is_err());
    }
}
