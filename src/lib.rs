use clap::{Parser, Subcommand};
use std::io;

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
pub enum Command {}

impl Cli {
    pub fn run(&mut self) -> io::Result<()> {
        Ok(())
    }
}
