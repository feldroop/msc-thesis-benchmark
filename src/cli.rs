use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command()]
pub struct Args {
    #[arg(default_value = "benchmark_config.toml")]
    pub config_file: PathBuf,
}
