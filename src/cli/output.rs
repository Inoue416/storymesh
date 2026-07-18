use std::{
    io::{self, Write},
    path::PathBuf,
};

use storymesh::CoverageReport;

pub(super) fn print_check(report: &CoverageReport, output: &mut dyn Write) -> io::Result<u8> {
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

pub(super) fn print_coverage(report: &CoverageReport, output: &mut dyn Write) -> io::Result<u8> {
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

pub(super) fn print_generated(paths: &[PathBuf], output: &mut dyn Write) -> io::Result<()> {
    if paths.is_empty() {
        return Ok(());
    }

    writeln!(output, "Generated {} story skeleton(s):", paths.len())?;
    for path in paths {
        writeln!(output, "{}", path.display())?;
    }
    Ok(())
}

pub(super) fn print_report(report: &CoverageReport, output: &mut dyn Write) -> io::Result<u8> {
    print_coverage(report, output)?;
    let missing = report.missing().collect::<Vec<_>>();
    writeln!(output, "Missing: {}", missing.len())?;
    for path in missing {
        writeln!(output, "{}", path.display())?;
    }
    Ok(0)
}
