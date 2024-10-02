use clap::Parser;
use std::path::PathBuf;

use crate::benchmarks::Benchmark;

#[derive(Parser)]
pub struct Args {
    #[arg(short, long, default_value = "benchmark_config.toml")]
    pub config_file: PathBuf,

    /// Give benchmark names that should be run.
    /// If none are given, all will be run
    #[arg(value_enum)]
    pub benchmarks: Option<Vec<Benchmark>>,
}
