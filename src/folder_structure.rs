use jiff::Zoned;

use std::path::{Path, PathBuf};

use crate::cli::BenchmarkConfig;

#[derive(Debug, Clone)]
pub struct BenchmarkFolder {
    folder: PathBuf,
}

impl BenchmarkFolder {
    pub fn new<P: AsRef<Path>>(
        base_output_folder: P,
        benchmark_name: &str,
        config: &BenchmarkConfig,
    ) -> Self {
        let mut folder = base_output_folder.as_ref().to_path_buf();
        folder.push(benchmark_name);

        let input_tag = format!("{}_in_{}", config.queries, config.reference);

        folder.push(input_tag);

        let mut subfolder_name: String = Zoned::now().strftime("%F--%H-%M-%S").to_string();

        if let Some(tag) = &config.tag {
            subfolder_name.push('_');
            subfolder_name.push_str(tag);
        }
        folder.push(&subfolder_name);

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
