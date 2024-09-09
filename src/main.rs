mod cli;
mod config;
mod run_floxer;

use std::{error::Error, fs};

use clap::Parser;
use config::Config;
use xshell::{cmd, Shell};

fn main() -> Result<(), Box<dyn Error>> {
    let sh = Shell::new()?;

    let args = cli::Args::parse();

    let config_file_str = fs::read_to_string(&args.config_file)?;
    let config: Config = toml::from_str(&config_file_str)?;

    let floxer_bin = config.readmapper_binaries.floxer;
    cmd!(sh, "{floxer_bin} --help").run()?;

    Ok(())
}
