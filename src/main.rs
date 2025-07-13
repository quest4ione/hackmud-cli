use std::process::ExitCode;

use clap::Parser;
use hackmud::Cli;

fn main() -> ExitCode {
    Cli::parse().run()
}
