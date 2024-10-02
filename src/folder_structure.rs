use anyhow::Result;
use jiff::Zoned;

use std::{
    fs,
    path::{Path, PathBuf},
};

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

pub fn all_plots_folder<P: AsRef<Path>>(base_output_folder: P) -> PathBuf {
    let mut base_output_folder = base_output_folder.as_ref().to_path_buf();
    base_output_folder.push("all_plots");
    base_output_folder
}

pub fn index_folder<P: AsRef<Path>>(base_output_folder: P) -> PathBuf {
    let mut base_output_folder = base_output_folder.as_ref().to_path_buf();
    base_output_folder.push("indices");
    base_output_folder
}

pub fn setup<P: AsRef<Path>>(base_output_folder: P) -> Result<()> {
    let index_folder = index_folder(base_output_folder.as_ref());
    if !index_folder.exists() {
        fs::create_dir_all(index_folder)?;
    }

    let all_plots_dir = all_plots_folder(base_output_folder);
    if !all_plots_dir.exists() {
        fs::create_dir_all(all_plots_dir)?;
    }

    Ok(())
}
