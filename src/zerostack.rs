use std::process::{Command, Stdio};

pub fn run(args: &[&str], model: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    let mut cmd = Command::new("zerostack");
    if let Some(m) = model {
        cmd.arg("--quick-model").arg(m);
    }
    // These calls run zerostack non-interactively in print mode:
    // - stdin is null so any interactive prompt (e.g. the one-time
    //   "Create ARCHITECTURE.md? [y/N]" question) reads EOF and proceeds
    //   instead of blocking forever.
    // - stdout is captured and returned to the caller.
    // - stderr is inherited so the user sees live agent progress; because of
    //   that it is not captured, so errors surface on the terminal directly.
    let output = cmd
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()?;
    if !output.status.success() {
        return Err(format!("zerostack {} (see output above)", output.status).into());
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
