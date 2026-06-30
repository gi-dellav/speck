use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "speck", version, about = "A fully spec-based AI agentic compiler")]
pub struct Cli {
    /// Answer yes to all confirmation prompts
    #[arg(long = "always-yes", short = 'y', global = true, conflicts_with = "always_no")]
    pub always_yes: bool,
    /// Answer no to all confirmation prompts
    #[arg(long = "always-no", short = 'n', global = true, conflicts_with = "always_yes")]
    pub always_no: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a new Speck project
    Init {
        /// Project name
        #[arg(long)]
        name: Option<String>,
        /// Source directory path
        #[arg(long)]
        source_path: Option<String>,
        /// Skip git initialization
        #[arg(long)]
        skip_git: bool,
    },
    /// Migrate a pre-existing project to Speck
    Migrate {
        /// Custom instruction to append to the coding agent
        #[arg(long)]
        custom: Option<String>,
    },
    /// Run a complete review of the project
    Review {
        /// Path to save the review Markdown file
        #[arg(long)]
        output: Option<String>,
    },
    /// Fully update the project referencing edited specifications and source code
    Apply {
        /// Custom instruction to append to the coding agent
        #[arg(long)]
        custom: Option<String>,
        /// Only run the specs-to-code pipeline
        #[arg(long = "only-direct", short = 'd')]
        only_direct: bool,
        /// Only run the code-to-specs pipeline
        #[arg(long = "only-inverse", short = 'i')]
        only_inverse: bool,
        /// Update specs/features/ (Step 2)
        #[arg(long)]
        update_features: bool,
        /// Solve conflicts giving priority to Code
        #[arg(long = "prefer-code", short = 'C')]
        prefer_code: bool,
        /// Solve conflicts giving priority to Specs
        #[arg(long = "prefer-specs", short = 'S')]
        prefer_specs: bool,
        /// Set the temperature used in Step 4 (code generation)
        #[arg(long)]
        gen_temperature: Option<f64>,
    },
    /// Run the formatting command defined in Speck.toml
    Fmt,
    /// Launch zerostack TUI and then update the project's files
    Chat,
    /// Re-set all stored hashes to current project files' hashes
    ForceUpdate,
    /// Remove all stored hashes
    Reset {
        /// Also remove the source code directory
        #[arg(long = "hard", short = 'h')]
        hard: bool,
        /// Also run speck apply after reset
        #[arg(long = "rebuild", short = 'r')]
        rebuild: bool,
        /// Also remove the specs/technical/ directory
        #[arg(long = "full", short = 'f')]
        full: bool,
    },
    /// List all unedited or unregistered files
    Status,
    /// Change the tech stack of the current project
    SwitchLang {
        /// Keep all specs (skip full reset — only hard + rebuild)
        #[arg(long = "keep-all-specs")]
        keep_all_specs: bool,
    },
    /// Move a file and update .speck_hash.toml
    Mv {
        /// Source path
        source: String,
        /// Destination path
        dest: String,
    },
    /// Remove a file and update .speck_hash.toml
    Rm {
        /// Path to remove
        path: String,
    },
    /// Set up git hooks for the current Git/Speck project
    GitHooks,
}
