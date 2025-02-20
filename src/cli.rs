use clap::{Args, Parser};
use std::path::PathBuf;

use crate::benchmarks::Benchmark;
use crate::readmappers::floxer::CigarOutput;
use crate::readmappers::{Queries, Reference};

#[derive(Parser)]
pub struct CliArgs {
    #[arg(short, long, default_value = "benchmark_config.toml")]
    pub config_file: PathBuf,

    /// Give benchmark names that should be run. If none are given, all will be run
    #[arg(value_enum)]
    pub benchmarks: Option<Vec<Benchmark>>,

    #[command(flatten)]
    pub benchmark_config: BenchmarkConfig,
}

#[derive(Args, Clone)]
pub struct BenchmarkConfig {
    /// If given, only the analysis is rerun on the results of the most recent run of floxer (if there was one)
    #[arg(short, long)]
    pub only_analysis: bool,

    /// If given, this tag is appended to the folder name of all benchmarks
    #[arg(short, long)]
    pub tag: Option<String>,

    #[arg(short, long, value_enum, default_value_t = Reference::HumanGenomeHg38)]
    pub reference: Reference,

    #[arg(short, long, value_enum, default_value_t = Queries::HumanWgsNanopore)]
    pub queries: Queries,

    #[arg(short, long, value_enum, default_value_t = CigarOutput::Off)]
    pub cigar_output: CigarOutput,
}

impl BenchmarkConfig {
    pub fn with_reference(&self, reference: Reference) -> Self {
        BenchmarkConfig {
            reference,
            ..self.clone()
        }
    }

    pub fn with_queries(&self, queries: Queries) -> Self {
        BenchmarkConfig {
            queries,
            ..self.clone()
        }
    }

    pub fn with_smaller_queries(&self) -> Self {
        BenchmarkConfig {
            queries: self.queries.smaller_equivalent(),
            ..self.clone()
        }
    }
}
