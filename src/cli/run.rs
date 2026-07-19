use std::io::{self, Write};

use storymesh::{CoverageReport, ScanOptions, generate_story_skeletons, scan_with_options};

use super::{
    args::{CheckArgs, Cli, Command, ScanArgs},
    output::{print_check, print_coverage, print_generated, print_report},
};

pub(crate) fn run(cli: Cli, stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    match cli.command {
        Some(Command::Check(args)) => execute_check(args, stdout, stderr),
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

fn execute_check(args: CheckArgs, stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    let framework = args.scan.framework.into();
    let report = match scan_with_options(&args.scan.path, framework, &scan_options(&args.scan)) {
        Ok(report) => report,
        Err(error) => {
            let _ = writeln!(stderr, "error: {error}");
            return 2;
        }
    };

    let generated = if args.generate {
        match generate_story_skeletons(&args.scan.path, &report) {
            Ok(paths) => paths,
            Err(error) => {
                let _ = writeln!(stderr, "error: {error}");
                return 2;
            }
        }
    } else {
        Vec::new()
    };

    let code = match print_check(&report, stdout) {
        Ok(code) => code,
        Err(error) => {
            let _ = writeln!(stderr, "failed to write output: {error}");
            return 2;
        }
    };
    if let Err(error) = print_generated(&generated, stdout) {
        let _ = writeln!(stderr, "failed to write output: {error}");
        return 2;
    }
    if args.generate { 0 } else { code }
}

fn execute(
    args: ScanArgs,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    print: fn(&CoverageReport, &mut dyn Write) -> io::Result<u8>,
) -> u8 {
    let framework = args.framework.into();
    match scan_with_options(&args.path, framework, &scan_options(&args)) {
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

fn scan_options(args: &ScanArgs) -> ScanOptions {
    ScanOptions {
        ignore_patterns: args.ignore.clone(),
        ignore_files: args.ignore_file.clone(),
    }
}
