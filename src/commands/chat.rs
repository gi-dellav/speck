use crate::config::SpeckConfig;
use crate::zerostack;

pub fn run(cli_model: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = std::env::current_dir()?;
    let config_path = project_dir.join("Speck.toml");
    if !config_path.exists() {
        return Err("Not a Speck project: Speck.toml not found".into());
    }

    let config = SpeckConfig::from_file(&config_path)?;

    {
        let model = cli_model
            .as_deref()
            .or(config.plan_model.as_deref())
            .or(config.model.as_deref());
        println!("Launching zerostack TUI...");
        zerostack::run_tui(model)?;
    }

    let plan_model = cli_model.clone();
    let code_model = cli_model;

    println!("\nUpdating project files with speck apply...");
    crate::commands::apply::run(
        None, false, false, false, false, false, None, plan_model, code_model, false, false,
    )
}
