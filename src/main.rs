mod cli;
mod commands;
mod config;
mod hashes;

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
            commands::reset::run(hard, rebuild, full)
        }
        cli::Command::Status => {
            commands::status::run()
        }
        cli::Command::SwitchLang => {
            commands::switch_lang::run()
        }
        cli::Command::Mv { source, dest } => {
            commands::mv::run(source, dest)
        }
        cli::Command::Rm { path } => {
            commands::rm::run(path)
        }
        cli::Command::GitHooks => {
            commands::git_hooks::run()
        }
    }
}
