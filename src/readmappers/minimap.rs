use std::{fs, process::Command};

use crate::{
    cli::BenchmarkConfig,
    config::BenchmarkSuiteConfig,
    folder_structure::{BenchmarkFolder, BenchmarkInstanceFolder},
};

use super::{IndexStrategy, Queries, Reference, ResourceMetrics};
use anyhow::{bail, Result};

#[derive(Debug)]
pub struct MinimapConfig {
    pub reference: Reference,
    pub queries: Queries,
    pub index_strategy: IndexStrategy,
    pub num_threads: u16,
}

impl From<&BenchmarkConfig> for MinimapConfig {
    fn from(value: &BenchmarkConfig) -> Self {
        Self {
            reference: value.reference,
            queries: value.queries,
            index_strategy: IndexStrategy::ReadFromDiskIfStored,
            num_threads: super::NUM_THREADS_FOR_READMAPPERS,
        }
    }
}

impl MinimapConfig {
    pub fn run(
        &self,
        benchmark_folder: &BenchmarkFolder,
        suite_config: &BenchmarkSuiteConfig,
    ) -> Result<MinimapRunResult> {
        let instance_folder = BenchmarkInstanceFolder::from_benchmark_folder_and_instance_name(
            benchmark_folder,
            "minimap",
        )?;

        let mut index_path = suite_config.index_folder();
        let index_file_name = format!("minimap-index-{}-{}.mmi", self.reference, self.queries);
        index_path.push(index_file_name);

        println!(
            "- Running minimap for reference {} and queries {}",
            self.reference, self.queries
        );

        let index_resource_metrics = if self.index_strategy == IndexStrategy::ReadFromDiskIfStored
            || !index_path.exists()
        {
            let mut index_command = Command::new("/usr/bin/time");

            super::add_time_args(&mut index_command, &instance_folder.index_timing_path);

            index_command.arg(&suite_config.readmapper_binaries.minimap);
            index_command.arg("-x");
            index_command.arg(self.queries.minimap_preset());
            index_command.arg("-d");
            index_command.arg(&index_path);
            index_command.arg(self.reference.path(suite_config));
            index_command.arg("-t");
            index_command.arg(self.num_threads.to_string());

            let minimap_index_proc_output = index_command.output()?;
            if !minimap_index_proc_output.status.success() {
                bail!(
                    "minimap indexing errored with stderr: {}",
                    String::from_utf8_lossy(&minimap_index_proc_output.stderr)
                );
            }

            let index_timings_file_str = fs::read_to_string(instance_folder.index_timing_path)?;
            let index_resource_metrics: ResourceMetrics = toml::from_str(&index_timings_file_str)?;
            Some(index_resource_metrics)
        } else {
            None
        };

        let mut map_command = Command::new("/usr/bin/time");
        super::add_time_args(&mut map_command, &instance_folder.timing_path);

        map_command
            .arg(&suite_config.readmapper_binaries.minimap)
            .arg("-a")
            .arg(&index_path)
            .arg(self.queries.path(suite_config))
            .arg("-t")
            .arg(self.num_threads.to_string())
            .arg("-o")
            .arg(&instance_folder.mapped_reads_sam_path);

        let minimap_map_proc_output = map_command.output()?;
        if !minimap_map_proc_output.status.success() {
            bail!(
                "minimap mapping errored with stderr: {}",
                String::from_utf8_lossy(&minimap_map_proc_output.stderr)
            );
        }

        let map_timings_file_str = fs::read_to_string(&instance_folder.timing_path)?;
        let map_resource_metrics: ResourceMetrics = toml::from_str(&map_timings_file_str)?;

        Ok(MinimapRunResult {
            map_resource_metrics,
            index_resource_metrics,
        })
    }
}

pub struct MinimapRunResult {
    pub map_resource_metrics: ResourceMetrics,
    pub index_resource_metrics: Option<ResourceMetrics>,
}
