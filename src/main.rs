mod cli;
mod run_floxer;

use std::error::Error;

use clap::Parser;
use xshell::{cmd, Shell};

// config that is read from a file
struct Config {}

fn main() -> Result<(), Box<dyn Error>> {
    let sh = Shell::new()?;

    let args = cli::Args::parse();
    let config_file = args.config_file;
    cmd!(sh, "echo {config_file}").run()?;

    Ok(())
}
