use jiff::Zoned;

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct BenchmarkFolder {
    folder: PathBuf,
}

impl BenchmarkFolder {
    pub fn new<P: AsRef<Path>>(base_output_folder: P, benchmark_name: &str) -> Self {
        let mut folder = base_output_folder.as_ref().to_path_buf();
        folder.push(benchmark_name);

        let timestamp: String = Zoned::now().strftime("%F--%H-%M-%S").to_string();
        folder.push(&timestamp);

        Self { folder }
    }

    pub fn get(&self) -> &Path {
        &self.folder
    }

    pub fn plot_folder(&self) -> PathBuf {
        let mut folder = self.folder.clone();
        folder.push("plots");
        folder
    }
}
