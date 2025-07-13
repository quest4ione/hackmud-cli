use std::process::ExitCode;

pub mod sync;

use clap::{Parser, Subcommand};
use sync::Sync;

#[derive(Parser)]
/// Cli tools for working with scripts in the game hackmud
///
/// Provides some handy dandy tools, like printing this help message and the current version
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Sync(Sync),
}

impl Cli {
    pub fn run(&mut self) -> ExitCode {
        let res = match &self.command {
            Command::Sync(cmd) => cmd.run(self),
        };

        match res {
            Ok(_) => ExitCode::SUCCESS,
            Err(_) => ExitCode::FAILURE,
        }
    }
}
