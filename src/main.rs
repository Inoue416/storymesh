use std::{
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

use clap::{Args, Parser, Subcommand, ValueEnum};
use storymesh::{CoverageReport, Framework, scan};

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
    Check(ScanArgs),
    /// Calculate the ratio of components with at least one story file.
    Coverage(ScanArgs),
    /// Show covered and missing component totals.
    Report(ScanArgs),
}

#[derive(Clone, Debug, Args)]
struct ScanArgs {
    /// Directory to scan. Defaults to the current directory.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Component library to inspect.
    #[arg(long, value_enum, default_value_t = CliFramework::React)]
    framework: CliFramework,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliFramework {
    React,
}

impl From<CliFramework> for Framework {
    fn from(framework: CliFramework) -> Self {
        match framework {
            CliFramework::React => Self::React,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    ExitCode::from(run(cli, &mut io::stdout(), &mut io::stderr()))
}

fn run(cli: Cli, stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    match cli.command {
        Some(Command::Check(args)) => execute(args, stdout, stderr, print_check),
        Some(Command::Coverage(args)) => execute(args, stdout, stderr, print_coverage),
        Some(Command::Report(args)) => execute(args, stdout, stderr, print_report),
        None => {
            if writeln!(stdout, "Run `storymesh --help` to see available commands.").is_err() {
                return 2;
            }
            0
        }
    }
}

fn execute(
    args: ScanArgs,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    print: fn(&CoverageReport, &mut dyn Write) -> io::Result<u8>,
) -> u8 {
    let framework = args.framework.into();
    match scan(&args.path, framework) {
        Ok(report) => match print(&report, stdout) {
            Ok(code) => code,
            Err(error) => {
                let _ = writeln!(stderr, "failed to write output: {error}");
                2
            }
        },
        Err(error) => {
            let _ = writeln!(stderr, "error: {error}");
            2
        }
    }
}

fn print_check(report: &CoverageReport, output: &mut dyn Write) -> io::Result<u8> {
    let missing = report.missing().collect::<Vec<_>>();
    if missing.is_empty() {
        writeln!(
            output,
            "All {} {} components have stories.",
            report.components.len(),
            report.framework.name()
        )?;
        return Ok(0);
    }

    writeln!(
        output,
        "Missing stories for {} {} component(s):",
        missing.len(),
        report.framework.name()
    )?;
    for path in missing {
        writeln!(output, "{}", path.display())?;
    }
    Ok(1)
}

fn print_coverage(report: &CoverageReport, output: &mut dyn Write) -> io::Result<u8> {
    writeln!(
        output,
        "{} Storybook coverage: {:.1}% ({}/{} components)",
        report.framework.name(),
        report.percentage(),
        report.covered_count(),
        report.components.len()
    )?;
    Ok(0)
}

fn print_report(report: &CoverageReport, output: &mut dyn Write) -> io::Result<u8> {
    print_coverage(report, output)?;
    let missing = report.missing().collect::<Vec<_>>();
    writeln!(output, "Missing: {}", missing.len())?;
    for path in missing {
        writeln!(output, "{}", path.display())?;
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use clap::Parser;
    use storymesh::{ComponentCoverage, CoverageReport, Framework};

    use super::{Cli, Command, print_check, run};

    #[test]
    fn parses_check_target() {
        let cli = Cli::try_parse_from(["storymesh", "check", "components"])
            .expect("the check command should parse");

        assert!(matches!(
            cli.command,
            Some(Command::Check(args)) if args.path == Path::new("components")
        ));
    }

    #[test]
    fn check_returns_an_error_for_a_missing_directory() {
        let cli = Cli::try_parse_from(["storymesh", "check", "directory-that-does-not-exist"])
            .expect("the check command should parse");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = run(cli, &mut stdout, &mut stderr);

        assert_eq!(code, 2);
        assert!(stdout.is_empty());
        assert!(String::from_utf8_lossy(&stderr).contains("directory-that-does-not-exist"));
    }

    #[test]
    fn check_lists_missing_components_and_returns_one() {
        let report = CoverageReport {
            framework: Framework::React,
            components: vec![
                ComponentCoverage {
                    component: PathBuf::from("Button.tsx"),
                    story: Some(PathBuf::from("Button.stories.tsx")),
                },
                ComponentCoverage {
                    component: PathBuf::from("Card.tsx"),
                    story: None,
                },
            ],
        };
        let mut output = Vec::new();

        let code = print_check(&report, &mut output).expect("check output should be writable");

        assert_eq!(code, 1);
        let output = String::from_utf8_lossy(&output);
        assert!(output.contains("Missing stories for 1 React component(s)"));
        assert!(output.contains("Card.tsx"));
        assert!(!output.contains("Button.tsx"));
    }
}
