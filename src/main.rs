mod cli;
mod commands;
mod config;
mod hashes;
mod helpers;
#[cfg(test)]
mod test_utils;
mod zerostack;

use clap::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Command::Init { name, source_path, skip_git } => {
            commands::init::run(name, source_path, skip_git)
        }
        cli::Command::Migrate { custom } => {
            commands::migrate::run(custom)
        }
        cli::Command::Review { output } => {
            commands::review::run(output)
        }
        cli::Command::Apply { custom, only_direct, only_inverse, update_features, prefer_code, prefer_specs, gen_temperature } => {
            commands::apply::run(custom, only_direct, only_inverse, update_features, prefer_code, prefer_specs, gen_temperature)
        }
        cli::Command::Fmt => {
            commands::fmt::run()
        }
        cli::Command::Chat => {
            commands::chat::run()
        }
        cli::Command::ForceUpdate => {
            commands::force_update::run()
        }
        cli::Command::Reset { hard, rebuild, full } => {
            commands::reset::run(hard, rebuild, full, cli.always_yes, cli.always_no)
        }
        cli::Command::Status => {
            commands::status::run()
        }
        cli::Command::SwitchLang { keep_all_specs } => {
            commands::switch_lang::run(keep_all_specs, cli.always_yes, cli.always_no)
        }
        cli::Command::Mv { source, dest } => {
            commands::mv::run(source, dest)
        }
        cli::Command::Rm { path } => {
            commands::rm::run(path, cli.always_yes, cli.always_no)
        }
        cli::Command::GitHooks => {
            commands::git_hooks::run()
        }
    }
}
