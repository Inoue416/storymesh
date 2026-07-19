use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use storymesh::Framework;

/// Audit Storybook story coverage and report gaps.
#[derive(Debug, Parser)]
#[command(version, about)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(super) command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub(super) enum Command {
    /// Find components that do not have a corresponding story file.
    Check(CheckArgs),
    /// Calculate the ratio of components with at least one story file.
    Coverage(ScanArgs),
    /// Show covered and missing component totals.
    Report(ScanArgs),
}

#[derive(Clone, Debug, Args)]
pub(super) struct CheckArgs {
    #[command(flatten)]
    pub(super) scan: ScanArgs,

    /// Generate a minimal story file beside each component missing one.
    #[arg(long)]
    pub(super) generate: bool,
}

#[derive(Clone, Debug, Args)]
pub(super) struct ScanArgs {
    /// Directory to scan. Defaults to the current directory.
    #[arg(default_value = ".")]
    pub(super) path: PathBuf,

    /// Component library to inspect.
    #[arg(long, value_enum, default_value_t = CliFramework::React)]
    pub(super) framework: CliFramework,

    /// Ignore a path pattern. May be specified more than once.
    #[arg(long, value_name = "PATTERN")]
    pub(super) ignore: Vec<String>,

    /// Load additional Git ignore-style patterns from this file. May be specified more than once.
    #[arg(long, value_name = "PATH")]
    pub(super) ignore_file: Vec<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(super) enum CliFramework {
    React,
    Vue,
    Angular,
}

impl From<CliFramework> for Framework {
    fn from(framework: CliFramework) -> Self {
        match framework {
            CliFramework::React => Self::React,
            CliFramework::Vue => Self::Vue,
            CliFramework::Angular => Self::Angular,
        }
    }
}
