use dialoguer::MultiSelect;

const HOOKS: &[(&str, &str, &str)] = &[
    ("check-before-commit", "pre-commit", "speck status && speck apply"),
    ("format-before-commit", "pre-commit", "speck fmt"),
    ("apply-before-push", "pre-push", "speck apply"),
    ("apply-after-merge", "post-merge", "speck apply"),
    ("apply-after-checkout", "post-checkout", "speck apply"),
];

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = std::env::current_dir()?;
    let git_dir = project_dir.join(".git");
    if !git_dir.exists() {
        return Err("Not a git repository. Run `git init` first.".into());
    }

    let hook_names: Vec<&str> = HOOKS.iter().map(|(name, _, _)| *name).collect();
    let selection = MultiSelect::new()
        .with_prompt("Select git hooks to install (space to select, enter to confirm)")
        .items(&hook_names)
        .interact()?;

    if selection.is_empty() {
        println!("No hooks selected.");
        return Ok(());
    }

    let hooks_dir = git_dir.join("hooks");
    std::fs::create_dir_all(&hooks_dir)?;

    for idx in selection {
        let (name, hook_type, command) = HOOKS[idx];
        let hook_path = hooks_dir.join(hook_type);
        let existing = if hook_path.exists() {
            std::fs::read_to_string(&hook_path).unwrap_or_default()
        } else {
            String::new()
        };

        let entry = format!(
            "# speck {}\n{}\n",
            name, command
        );

        if existing.contains(&format!("# speck {}", name)) {
            println!("Hook '{}' already installed for {}.", name, hook_type);
            continue;
        }

        let mut new_content = if existing.is_empty() {
            format!("#!/bin/sh\n\n{}", entry)
        } else if existing.ends_with('\n') {
            format!("{}{}", existing, entry)
        } else {
            format!("{}\n{}", existing, entry)
        };

        if !existing.contains("#!/bin/sh") && !existing.contains("#!/usr/bin/env") {
            new_content = format!("#!/bin/sh\n\n{}", new_content.trim_start_matches("#!/bin/sh\n"));
            if !existing.is_empty() && !existing.starts_with("#!") {
                new_content = format!("#!/bin/sh\n{}", new_content.trim_start_matches("#!/bin/sh\n"));
            }
        }

        std::fs::write(&hook_path, &new_content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&hook_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&hook_path, perms)?;
        }

        println!("Installed {} hook: speck {}", hook_type, name);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hooks_constants() {
        assert_eq!(HOOKS.len(), 5);
        assert_eq!(HOOKS[0].0, "check-before-commit");
        assert_eq!(HOOKS[0].1, "pre-commit");
        assert_eq!(HOOKS[1].0, "format-before-commit");
        assert_eq!(HOOKS[1].1, "pre-commit");
        assert_eq!(HOOKS[2].1, "pre-push");
        assert_eq!(HOOKS[3].1, "post-merge");
        assert_eq!(HOOKS[4].1, "post-checkout");
    }

    #[test]
    fn test_run_fails_outside_git_repo() {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("speck_hooks_test_{}_{}", std::process::id(), id));
        std::fs::create_dir_all(&dir).unwrap();
        let result = crate::test_utils::with_cwd_locked(&dir, run);
        std::fs::remove_dir_all(&dir).ok();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not a git repository"));
    }
}
