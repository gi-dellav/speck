use crate::zerostack;

pub fn run(output: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = std::env::current_dir()?;
    let config_path = project_dir.join("Speck.toml");
    if !config_path.exists() {
        return Err("Not a Speck project: Speck.toml not found".into());
    }

    let prompt_path = zerostack::prompt_path("speck-review.md");
    if let Some(output_path) = output {
        let result = zerostack::run(&[
            "--load-prompt",
            &prompt_path,
            "--no-session",
            "--temperature",
            "0",
        ])?;
        std::fs::write(&output_path, &result)?;
        println!("Review saved to {}", output_path);
    } else {
        let result = zerostack::run(&[
            "--load-prompt",
            &prompt_path,
            "--pure-stdout",
            "--no-session",
            "--temperature",
            "0",
        ])?;
        println!("{}", result);
    }
    Ok(())
}
