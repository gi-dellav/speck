use crate::config::SpeckConfig;
use crate::helpers;
use crate::zerostack;
use dialoguer::Input;

pub fn run(
    keep_all_specs: bool,
    always_yes: bool,
    always_no: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = std::env::current_dir()?;
    let config_path = project_dir.join("Speck.toml");
    if !config_path.exists() {
        return Err("Not a Speck project: Speck.toml not found".into());
    }

    let config = SpeckConfig::from_file(&config_path)?;

    let tech_stack_path = project_dir.join("specs/TECH_STACK.md");
    if !tech_stack_path.exists() {
        return Err("specs/TECH_STACK.md not found. Run `speck init` first.".into());
    }

    if keep_all_specs {
        eprintln!(
            "WARNING: speck switch-lang will DELETE all source code in {}/.",
            config.source_dir
        );
        eprintln!(
            "It will then regenerate everything from specs/ and TECH_STACK.md using the AI agent."
        );
        eprintln!("Consider committing your work first.\n");
    } else {
        eprintln!(
            "WARNING: speck switch-lang will DELETE all source code in {}/ and all of specs/technical/.",
            config.source_dir
        );
        eprintln!(
            "It will then regenerate everything from specs/features/ and TECH_STACK.md using the AI agent."
        );
        eprintln!("Consider committing your work first.\n");
    }
    let proceed = helpers::confirm(always_yes, always_no, "Proceed with switch-lang?", false)?;
    if !proceed {
        println!("Aborted.");
        return Ok(());
    }

    let current = std::fs::read_to_string(&tech_stack_path)?;
    println!("Current TECH_STACK.md:\n{}", current);

    let new_stack = Input::<String>::new()
        .with_prompt("Describe the new tech stack")
        .interact_text()?;

    eprintln!("Updating specs/TECH_STACK.md...");
    let msg = format!(
        "Current TECH_STACK.md:\n{}\n\nNew tech stack requirements:\n{}\n\n\
         Update specs/TECH_STACK.md to reflect the new tech stack.",
        current, new_stack
    );
    zerostack::run_p(&["--no-session"], &msg, config.model.as_deref())?;

    if keep_all_specs {
        eprintln!("Resetting and rebuilding project (keeping all specs)...");
        crate::commands::reset::run(true, true, false, always_yes, always_no)
    } else {
        eprintln!("Resetting and rebuilding project...");
        crate::commands::reset::run(true, true, true, always_yes, always_no)
    }
}

#[cfg(test)]
mod tests {
    fn run_switch_lang_in_dir(
        dir: &std::path::Path,
        keep_all_specs: bool,
        always_yes: bool,
        always_no: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        crate::test_utils::with_cwd_locked(dir, || {
            super::run(keep_all_specs, always_yes, always_no)
        })
    }

    #[test]
    fn test_switch_lang_fails_without_speck_toml() {
        let dir =
            std::env::temp_dir().join(format!("speck_switch_lang_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let result = run_switch_lang_in_dir(&dir, false, false, false);
        std::fs::remove_dir_all(&dir).ok();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Speck.toml"));
    }

    #[test]
    fn test_switch_lang_fails_without_tech_stack() {
        let dir =
            std::env::temp_dir().join(format!("speck_switch_lang_test2_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("Speck.toml"),
            "name = \"test\"\nsource_dir = \"src\"\n",
        )
        .unwrap();
        let result = run_switch_lang_in_dir(&dir, false, false, false);
        std::fs::remove_dir_all(&dir).ok();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("TECH_STACK.md"));
    }
}
