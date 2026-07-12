use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};

/// Audit Storybook story coverage and report gaps.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Find components that do not have a corresponding story file.
    Check {
        /// Directory to scan. Defaults to the current directory.
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Calculate the ratio of components with at least one story file.
    Coverage {
        /// Directory to scan. Defaults to the current directory.
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Produce a coverage report (planned).
    Report {
        /// Directory to scan. Defaults to the current directory.
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Check { path }) => not_implemented("check", path),
        Some(Command::Coverage { path }) => not_implemented("coverage", path),
        Some(Command::Report { path }) => not_implemented("report", path),
        None => {
            println!("Run `storymesh --help` to see available commands.");
            ExitCode::SUCCESS
        }
    }
}

fn not_implemented(command: &str, path: PathBuf) -> ExitCode {
    eprintln!(
        "`storymesh {command}` is not implemented yet (target: {}).",
        path.display()
    );
    ExitCode::from(2)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use clap::Parser;

    use super::{Cli, Command};

    #[test]
    fn parses_check_target() {
        let cli = Cli::try_parse_from(["storymesh", "check", "components"])
            .expect("the check command should parse");

        assert!(matches!(
            cli.command,
            Some(Command::Check { path }) if path == Path::new("components")
        ));
    }
}
