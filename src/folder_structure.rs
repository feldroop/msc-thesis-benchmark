use jiff::Zoned;

use std::path::{Path, PathBuf};

pub struct BenchmarkFolder {
    base_output_folder: PathBuf,
}

impl BenchmarkFolder {
    pub fn new<P: AsRef<Path>>(base_output_folder: P, benchmark_name: &str) -> Self {
        let mut base_output_folder = base_output_folder.as_ref().to_path_buf();
        base_output_folder.push(benchmark_name);

        let timestamp: String = Zoned::now().strftime("%F--%H-%M-%S").to_string();
        base_output_folder.push(&timestamp);

        Self { base_output_folder }
    }

    pub fn get(&self) -> &Path {
        &self.base_output_folder
    }

    pub fn plot_folder(&self) -> PathBuf {
        let mut folder = self.base_output_folder.clone();
        folder.push("plots");
        folder
    }
}
