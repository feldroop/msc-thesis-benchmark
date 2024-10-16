use serde::Deserialize;
use std::path::PathBuf;

// config that is read from a file.
// these are parameters of this program that don't change for every benchmark
// run and therefore should not be passed in every cli invocation
#[derive(Deserialize)]
pub struct BenchmarkSuiteConfig {
    pub output_folder: PathBuf,
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
}

#[derive(Deserialize)]
pub struct QueryPaths {
    pub human_wgs_nanopore: PathBuf,
}
