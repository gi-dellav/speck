use crate::zerostack;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = std::env::current_dir()?;
    let config_path = project_dir.join("Speck.toml");
    if !config_path.exists() {
        return Err("Not a Speck project: Speck.toml not found".into());
    }

    println!("Launching zerostack TUI...");
    zerostack::run_tui()?;

    println!("\nUpdating project files with speck apply...");
    crate::commands::apply::run(None, false, false, false, false, false, None, false, false)
}
