use clap::{builder::Str, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "jargo")]
#[command(about = "Build tool for Java projects", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new project skeleton with default Jargo.toml
    New {
        name: String,
    },
    /// Build the project (generate Gradle files and run build)
    Build {
        #[arg(value_name = "DIR")]
        directory_opt: Option<PathBuf>,
    },
}
