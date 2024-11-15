use anyhow::Result;
use serde::Deserialize;
use std::{fs, path::PathBuf};

// config that is read from a file.
// these are parameters of this program that don't change for every benchmark
// run and therefore should not be passed in every cli invocation
#[derive(Deserialize)]
pub struct BenchmarkSuiteConfig {
    pub output_folder: PathBuf,
    pub compare_aligner_outputs_binary: PathBuf,
    pub readmapper_binaries: ReadmapperBinaries,
    pub reference_paths: ReferencePaths,
    pub query_paths: QueryPaths,
}

#[derive(Deserialize)]
pub struct ReadmapperBinaries {
    pub floxer: PathBuf,
    pub minimap: PathBuf,
}

#[derive(Deserialize)]
pub struct ReferencePaths {
    pub human_genome_hg38: PathBuf,
    pub debug: PathBuf,
    pub simulated: PathBuf,
}

#[derive(Deserialize)]
pub struct QueryPaths {
    pub human_wgs_nanopore: PathBuf,
    pub human_wgs_nanopore_small: PathBuf,
    pub debug: PathBuf,
    pub problem_query: PathBuf,
    pub simulated_simple: PathBuf,
    pub simulated_pbsim: PathBuf,
}

impl BenchmarkSuiteConfig {
    pub fn all_plots_folder(&self) -> PathBuf {
        let mut base_output_folder = self.output_folder.clone();
        base_output_folder.push("all_plots");
        base_output_folder
    }

    pub fn index_folder(&self) -> PathBuf {
        let mut base_output_folder = self.output_folder.clone();
        base_output_folder.push("indices");
        base_output_folder
    }

    pub fn setup(&self) -> Result<()> {
        let index_folder = self.index_folder();
        if !index_folder.exists() {
            fs::create_dir_all(index_folder)?;
        }

        let all_plots_dir = self.all_plots_folder();
        if !all_plots_dir.exists() {
            fs::create_dir_all(all_plots_dir)?;
        }

        Ok(())
    }
}
