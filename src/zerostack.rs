use std::process::{Command, Stdio};

pub fn run(args: &[&str], model: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    let mut cmd = Command::new("zerostack");
    if let Some(m) = model {
        cmd.arg("--quick-model").arg(m);
    }
    let output = cmd
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("zerostack failed: {}", stderr).into());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn run_p(args: &[&str], msg: &str, model: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    let mut all_args: Vec<&str> = args.to_vec();
    all_args.push("-p");
    all_args.push(msg);
    run(&all_args, model)
}

pub fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("zerostack")
        .status()?;
    if !status.success() {
        return Err("zerostack TUI failed".into());
    }
    Ok(())
}

/// Returns the prompt name expected by zerostack's `--load-prompt`.
///
/// zerostack loads prompts from `.zerostack/prompts/` (among other dirs) and
/// keys them by file stem, so `--load-prompt` wants the bare name without the
/// directory prefix or the `.md` extension (e.g. `speck-code2tech`).
pub fn prompt_name(name: &str) -> String {
    name.strip_suffix(".md").unwrap_or(name).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_name() {
        assert_eq!(prompt_name("speck-code2tech.md"), "speck-code2tech");
        assert_eq!(prompt_name("speck-review"), "speck-review");
    }
}
