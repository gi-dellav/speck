use crate::config::SpeckConfig;
use crate::zerostack;

pub fn run(output: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = std::env::current_dir()?;
    let config_path = project_dir.join("Speck.toml");
    if !config_path.exists() {
        return Err("Not a Speck project: Speck.toml not found".into());
    }

    let config = SpeckConfig::from_file(&config_path)?;
    let model = config.model.as_deref();

    let prompt_name = zerostack::prompt_name("speck-review.md");
    let msg = "Perform the complete code review described in your instructions and \
               output the full Markdown report.";
    eprintln!("Reviewing project... (this runs the agent and may take a while)");
    if let Some(output_path) = output {
        let result = zerostack::run_p(&[
            "--load-prompt",
            &prompt_name,
            "--no-session",
            "--temperature",
            "0",
        ], msg, model)?;
        std::fs::write(&output_path, &result)?;
        println!("Review saved to {}", output_path);
    } else {
        let result = zerostack::run_p(&[
            "--load-prompt",
            &prompt_name,
            "--pure-stdout",
            "--no-session",
            "--temperature",
            "0",
        ], msg, model)?;
        println!("{}", result);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    fn run_review_in_dir(dir: &std::path::Path, output: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        crate::test_utils::with_cwd_locked(dir, || super::run(output))
    }

    #[test]
    fn test_review_fails_without_speck_toml() {
        let dir = std::env::temp_dir()
            .join(format!("speck_review_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let result = run_review_in_dir(&dir, None);
        std::fs::remove_dir_all(&dir).ok();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Speck.toml"));
    }
}
