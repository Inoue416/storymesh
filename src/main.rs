mod cli;

use std::{io, process::ExitCode};

use clap::Parser;

use crate::cli::{Cli, run};

fn main() -> ExitCode {
    let cli = Cli::parse();
    ExitCode::from(run(cli, &mut io::stdout(), &mut io::stderr()))
}
