use clap::Parser;
use hm_scripts::Cli;

fn main() {
    let mut cli = Cli::parse();
    cli.run().unwrap();
}
