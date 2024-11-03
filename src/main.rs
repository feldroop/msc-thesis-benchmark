mod analyze_mapped_reads;
mod benchmarks;
mod cli;
mod config;
mod folder_structure;
mod plots;
mod readmappers;

use std::{error::Error, fs};

use clap::Parser;
use config::BenchmarkSuiteConfig;

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Args::parse();

    let config_file_str = fs::read_to_string(&args.config_file)?;
    let suite_config: BenchmarkSuiteConfig = toml::from_str(&config_file_str)?;

    suite_config.setup()?;

    if let Some(benchmarks) = args.benchmarks {
        benchmarks::run_benchmarks(benchmarks, &suite_config)?;
    } else {
        benchmarks::run_all(&suite_config)?;
    }

    Ok(())
}
