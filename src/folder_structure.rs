use anyhow::Result;
use jiff::Zoned;

use std::{
    fs,
    os::unix,
    path::{Path, PathBuf},
};

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

    pub fn most_recect_previous_run_folder(&self) -> PathBuf {
        let mut most_recent_link = self.folder.clone();
        most_recent_link.pop();
        most_recent_link.push("most_recent");

        most_recent_link
    }

    pub fn create_or_update_link_to_most_recent(&self) -> Result<()> {
        let most_recent_link = self.most_recect_previous_run_folder();

        if most_recent_link.exists() {
            fs::remove_file(&most_recent_link)?;
        }

        unix::fs::symlink(&self.folder, most_recent_link)?;

        Ok(())
    }
}

pub struct BenchmarkInstanceFolder {
    pub mapped_reads_sam_path: PathBuf,
    pub mapped_reads_bam_path: PathBuf,
    pub logfile_path: PathBuf,
    pub timing_path: PathBuf,
    pub index_timing_path: PathBuf,
    pub stats_path: PathBuf,
    pub perf_data_path: PathBuf,
    pub samply_profile_path: PathBuf,
    pub flamegraph_path: PathBuf,
}

impl BenchmarkInstanceFolder {
    pub fn new(benchmark_folder: &BenchmarkFolder, instance_name: &str) -> Result<Self> {
        let mut base_folder = benchmark_folder.get().to_path_buf();
        base_folder.push(instance_name);

        Self::from_parts(base_folder, instance_name)
    }

    pub fn most_recent_previous_run(
        benchmark_folder: &BenchmarkFolder,
        instance_name: &str,
    ) -> Result<Self> {
        let mut base_folder = benchmark_folder.most_recect_previous_run_folder();
        base_folder.push(instance_name);

        Self::from_parts(base_folder, instance_name)
    }

    fn from_parts(base_folder: PathBuf, instance_name: &str) -> Result<Self> {
        if !base_folder.exists() {
            fs::create_dir_all(&base_folder)?;
        }

        let mut mapped_reads_sam_path = base_folder.clone();
        mapped_reads_sam_path.push("mapped_reads.sam");

        let mut mapped_reads_bam_path = base_folder.clone();
        mapped_reads_bam_path.push("mapped_reads.bam");

        let mut logfile_path = base_folder.clone();
        logfile_path.push("log.txt");

        let mut timing_path = base_folder.clone();
        timing_path.push("timing.toml");

        let mut index_timing_path = base_folder.clone();
        index_timing_path.push("index_timing.toml");

        let mut stats_path = base_folder.clone();
        stats_path.push("stats.toml");

        let mut perf_data_path = base_folder.clone();
        perf_data_path.push("perf.data");

        let mut samply_profile_path = base_folder.clone();
        samply_profile_path.push("samply_profile.json");

        let mut flamegraph_path = base_folder.clone();
        flamegraph_path.push(format!("flamegraph_{}.svg", instance_name));

        Ok(Self {
            mapped_reads_sam_path,
            mapped_reads_bam_path,
            logfile_path,
            timing_path,
            index_timing_path,
            stats_path,
            perf_data_path,
            samply_profile_path,
            flamegraph_path,
        })
    }
}
