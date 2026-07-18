use std::io::{self, Write};

use storymesh::{CoverageReport, scan};

use super::{
    args::{Cli, Command, ScanArgs},
    output::{print_check, print_coverage, print_report},
};

pub(crate) fn run(cli: Cli, stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
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
