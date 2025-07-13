use std::process::ExitCode;

use clap::Parser;
use hm_scripts::Cli;

fn main() -> ExitCode {
    Cli::parse().run()
}
