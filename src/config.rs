use serde::Deserialize;
use std::path::PathBuf;

// config that is read from a file

// these are parameters of this program that don't change for
// every benchmark run and therefore should not be passed in every cli invocation
#[derive(Deserialize)]
pub struct Config {
    pub output_dir: PathBuf,
    pub readmapper_binaries: ReadmapperBinaries,
}

#[derive(Deserialize)]
pub struct ReadmapperBinaries {
    pub floxer: PathBuf,
    pub minimap: PathBuf,
    pub ngmlr: PathBuf,
}
