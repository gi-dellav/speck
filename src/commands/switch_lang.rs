use crate::zerostack;
use dialoguer::Input;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = std::env::current_dir()?;
    let config_path = project_dir.join("Speck.toml");
    if !config_path.exists() {
        return Err("Not a Speck project: Speck.toml not found".into());
    }

    let tech_stack_path = project_dir.join("specs/TECH_STACK.md");
    if !tech_stack_path.exists() {
        return Err("specs/TECH_STACK.md not found. Run `speck init` first.".into());
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
    zerostack::run_p(
        &["--no-session"],
        &msg,
    )?;

    eprintln!("Resetting and rebuilding project...");
    crate::commands::reset::run(false, true, true)
}
