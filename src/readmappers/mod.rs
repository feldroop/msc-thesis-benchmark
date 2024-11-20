use std::{path::Path, process::Command};

use clap::ValueEnum;
use serde::Deserialize;
use strum::Display;

use crate::config::BenchmarkSuiteConfig;

pub mod floxer;
pub mod minimap;

const TIME_TOOL_FORMAT_STRING: &str = "wall_clock_seconds = %e
user_cpu_seconds = %U
system_cpu_seconds = %S
peak_memory_kilobytes = %M";

const NUM_THREADS_FOR_READMAPPERS: u16 = 16;

fn add_time_args(command: &mut Command, timing_path: &Path) {
    command
        .arg("--output")
        .arg(timing_path)
        .arg("--format")
        .arg(TIME_TOOL_FORMAT_STRING);
}

#[derive(Debug, Default, Copy, Clone, ValueEnum, Display)]
#[strum(serialize_all = "snake_case")]
pub enum Reference {
    #[default]
    HumanGenomeHg38,
    Debug,
    Simulated,
}

impl Reference {
    fn path<'a>(&self, suite_config: &'a BenchmarkSuiteConfig) -> &'a Path {
        match self {
            Reference::HumanGenomeHg38 => &suite_config.reference_paths.human_genome_hg38,
            Reference::Debug => &suite_config.reference_paths.debug,
            Reference::Simulated => &suite_config.reference_paths.simulated,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, ValueEnum, Display)]
#[strum(serialize_all = "snake_case")]
pub enum Queries {
    #[default]
    HumanWgsNanopore,
    HumanWgsNanoporeSmall,
    Debug,
    ProblemQuery,
    Simulated,
}

impl Queries {
    fn path<'a>(&self, suite_config: &'a BenchmarkSuiteConfig) -> &'a Path {
        match self {
            Queries::HumanWgsNanopore => &suite_config.query_paths.human_wgs_nanopore,
            Queries::HumanWgsNanoporeSmall => &suite_config.query_paths.human_wgs_nanopore_small,
            Queries::Debug => &suite_config.query_paths.debug,
            Queries::ProblemQuery => &suite_config.query_paths.problem_query,
            Queries::Simulated => &suite_config.query_paths.simulated,
        }
    }

    fn minimap_preset(&self) -> &str {
        match self {
            Queries::HumanWgsNanopore => "map-ont",
            Queries::HumanWgsNanoporeSmall => "map-ont",
            Queries::Debug => "map-ont",
            Queries::ProblemQuery => "map-ont",
            Queries::Simulated => "map-ont",
        }
    }

    pub fn smaller_equivalent(&self) -> Self {
        match self {
            Queries::HumanWgsNanopore | Queries::HumanWgsNanoporeSmall => {
                Queries::HumanWgsNanoporeSmall
            }
            Queries::Debug | Queries::ProblemQuery => *self,
            _ => panic!("No smaller equivalent for given query set: {}", self),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IndexStrategy {
    AlwaysRebuild,
    ReadFromDiskIfStored,
}

#[derive(Debug, Deserialize)]
pub struct ResourceMetrics {
    pub wall_clock_seconds: f64,
    pub user_cpu_seconds: f64,
    pub system_cpu_seconds: f64,
    pub peak_memory_kilobytes: usize,
}
