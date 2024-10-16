use crate::{
    benchmarks::ProfileConfig, config::BenchmarkSuiteConfig, folder_structure::BenchmarkFolder,
};

use std::{fs, path::Path, process::Command};

use anyhow::{bail, Result};
use serde::Deserialize;
use strum::{Display, EnumIter};

const TIME_TOOL_FORMAT_STRING: &str = "wall_clock_seconds = %e
user_cpu_seconds = %U
system_cpu_seconds = %S
peak_memory_kilobytes = %M
average_memory_kilobytes = %K";

#[derive(Debug, Default, Copy, Clone)]
pub enum Reference {
    #[default]
    HumanGenomeHg38,
}

impl Reference {
    fn name_for_files(&self) -> &str {
        match self {
            Self::HumanGenomeHg38 => "human-genome-hg38",
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub enum Queries {
    #[default]
    HumanWgsNanopore,
}

#[derive(Debug, Copy, Clone)]
pub enum IndexStrategy {
    AlwaysRebuild,
    ReadFromDiskIfStored,
}

#[derive(Debug, Copy, Clone)]
pub enum QueryErrors {
    Exact(u16),
    Rate(f64),
}

#[derive(Debug, Copy, Clone, EnumIter, Display)]
#[strum(serialize_all = "snake_case")]
pub enum AnchorGroupOrder {
    ErrorsFirst,
    CountFirst,
    Hybrid,
}

#[derive(Debug, Copy, Clone, EnumIter, Display)]
#[strum(serialize_all = "snake_case")]
pub enum PexTreeConstruction {
    TopDown,
    BottomUp,
}

#[derive(Debug, Copy, Clone, EnumIter, Display)]
pub enum IntervalOptimization {
    #[strum(serialize = "interval_optimization_on")]
    On,
    #[strum(serialize = "interval_optimization_off")]
    Off,
}

#[derive(Debug, Copy, Clone, EnumIter, Display)]
#[strum(serialize_all = "snake_case")]
pub enum VerificationAlgorithm {
    DirectFull,
    Hierarchical,
}

#[derive(Debug)]
pub struct FloxerAlgorithmConfig {
    pub index_strategy: IndexStrategy,
    pub query_errors: QueryErrors,
    pub pex_seed_errors: u8,
    pub max_num_anchors: u32,
    pub anchor_group_order: AnchorGroupOrder,
    pub pex_tree_construction: PexTreeConstruction,
    pub interval_optimization: IntervalOptimization,
    pub extra_verification_ratio: f64,
    pub verification_algorithm: VerificationAlgorithm,
    pub num_threads: u16,
}

impl Default for FloxerAlgorithmConfig {
    fn default() -> Self {
        FloxerAlgorithmConfig {
            index_strategy: IndexStrategy::ReadFromDiskIfStored,
            query_errors: QueryErrors::Rate(0.07),
            pex_seed_errors: 2,
            max_num_anchors: 100,
            anchor_group_order: AnchorGroupOrder::Hybrid,
            pex_tree_construction: PexTreeConstruction::BottomUp,
            interval_optimization: IntervalOptimization::On,
            extra_verification_ratio: 0.02,
            verification_algorithm: VerificationAlgorithm::Hierarchical,
            num_threads: 64,
        }
    }
}

// API to configure a floxer benchmark run.
// the output path will be determined from the other parameters
#[derive(Debug, Default)]
pub struct FloxerConfig {
    pub name: Option<String>,
    pub reference: Reference,
    pub queries: Queries,
    pub algorithm_config: FloxerAlgorithmConfig,
}

impl FloxerConfig {
    // benchmark name should be a valid and good name for a folder
    pub fn run(
        &self,
        benchmark_folder: &BenchmarkFolder,
        benchmark_name: &str,
        suite_config: &BenchmarkSuiteConfig,
        profile_config: ProfileConfig,
    ) -> Result<FloxerRunResult> {
        let mut output_folder = benchmark_folder.get().to_path_buf();

        if let Some(instance_name) = &self.name {
            output_folder.push(instance_name);
        }

        if !output_folder.exists() {
            fs::create_dir_all(&output_folder)?;
        }

        let mut output_path = output_folder.clone();
        output_path.push("mapped_reads.bam");

        let mut logfile_path = output_folder.clone();
        logfile_path.push("log.txt");

        let mut timing_path = output_folder.clone();
        timing_path.push("timing.toml");

        let mut stats_path = output_folder.clone();
        stats_path.push("stats.toml");

        let mut perf_data_path = output_folder.clone();
        perf_data_path.push("perf.data");

        let mut command = match profile_config {
            ProfileConfig::Off => Command::new("/usr/bin/time"),
            ProfileConfig::On => {
                let mut command = Command::new("perf");
                command
                    .arg("record")
                    .arg("-o")
                    .arg(&perf_data_path)
                    .arg("-F")
                    .arg("100")
                    .arg("--")
                    .arg("/usr/bin/time");
                command
            }
        };

        command
            .arg("--output")
            .arg(&timing_path)
            .arg("--format")
            .arg(TIME_TOOL_FORMAT_STRING);

        let reference_path = match self.reference {
            Reference::HumanGenomeHg38 => &suite_config.reference_paths.human_genome_hg38,
        };

        let queries_path = match self.queries {
            Queries::HumanWgsNanopore => &suite_config.query_paths.human_wgs_nanopore,
        };

        // from here on the actual floxer command
        command
            .arg(&suite_config.readmapper_binaries.floxer)
            .arg("--reference")
            .arg(reference_path)
            .arg("--queries")
            .arg(queries_path)
            .arg("--output")
            .arg(&output_path)
            .arg("--logfile")
            .arg(logfile_path)
            .arg("--stats")
            .arg(&stats_path);

        if let IndexStrategy::ReadFromDiskIfStored = self.algorithm_config.index_strategy {
            let mut index_path = crate::folder_structure::index_folder(&suite_config.output_folder);
            let index_file_name = format!("floxer-index-{}.flxi", self.reference.name_for_files());
            index_path.push(index_file_name);

            command.arg("--index");
            command.arg(index_path);
        }

        match self.algorithm_config.query_errors {
            QueryErrors::Exact(num_errors) => {
                command.args(["--query-errors", &num_errors.to_string()]);
            }
            QueryErrors::Rate(error_ratio) => {
                command.args(["--error-probability", &error_ratio.to_string()]);
            }
        }

        command.args([
            "--seed-errors",
            &self.algorithm_config.pex_seed_errors.to_string(),
            "--max-anchors",
            &self.algorithm_config.max_num_anchors.to_string(),
            "--anchor-group-order",
            &self.algorithm_config.anchor_group_order.to_string(),
            "--extra-verification-ratio",
            &self.algorithm_config.extra_verification_ratio.to_string(),
            "--threads",
            &self.algorithm_config.num_threads.to_string(),
        ]);

        if let PexTreeConstruction::BottomUp = self.algorithm_config.pex_tree_construction {
            command.arg("--bottom-up-pex-tree");
        }

        if let IntervalOptimization::On = self.algorithm_config.interval_optimization {
            command.arg("--interval-optimization");
        }

        if let VerificationAlgorithm::DirectFull = self.algorithm_config.verification_algorithm {
            command.arg("--direct-full-verification");
        }

        println!(
            "- Running the benchmark: {}{}",
            benchmark_name,
            if let Some(instance_name) = &self.name {
                format!(" - {instance_name}")
            } else {
                String::new()
            }
        );
        let floxer_proc_output = command.output()?;

        if !floxer_proc_output.status.success()
            || (!floxer_proc_output.stdout.is_empty() && profile_config == ProfileConfig::Off)
        {
            bail!(
                "Something went wrong. Floxer process output: {:?}",
                floxer_proc_output
            );
        }

        let mut profile_path = output_folder.clone();
        profile_path.push("samply_profile.json");

        if let ProfileConfig::On = profile_config {
            create_profile(&perf_data_path, &profile_path, "floxer_profile")?;
        }

        let stats_file_str = fs::read_to_string(stats_path)?;
        let stats: FloxerStats = toml::from_str(&stats_file_str)?;

        let timings_file_str = fs::read_to_string(timing_path)?;
        let resource_metrics: ResourceMetrics = toml::from_str(&timings_file_str)?;

        Ok(FloxerRunResult {
            benchmark_instance_name: self.name.clone().unwrap_or_else(|| String::from("floxer")),
            stats,
            resource_metrics,
        })
    }
}

fn create_profile(perf_data_path: &Path, profile_path: &Path, profile_name: &str) -> Result<()> {
    let samply_output = Command::new("samply")
        .arg("import")
        .arg("--profile-name")
        .arg(profile_name)
        .arg("--save-only")
        .arg("--output")
        .arg(profile_path)
        .arg("--no-open")
        .arg(perf_data_path)
        .output()?;

    if !samply_output.status.success() {
        bail!(
            "Samply import failed with the following output: {:?}",
            samply_output
        )
    }

    let mut flamegraph_path = profile_path.to_owned();
    flamegraph_path.set_file_name("flamegraph");
    flamegraph_path.set_extension("svg");

    let flamegraph_output = Command::new("flamegraph")
        .arg("--deterministic")
        .arg("--perfdata")
        .arg(perf_data_path)
        .arg("--output")
        .arg(flamegraph_path)
        .output()?;

    if !flamegraph_output.status.success() {
        bail!(
            "flamegraph generation failed with the following output: {:?}",
            flamegraph_output
        )
    }

    Ok(())
}

#[derive(Debug)]
pub struct FloxerRunResult {
    pub benchmark_instance_name: String,
    pub stats: FloxerStats,
    pub resource_metrics: ResourceMetrics,
}

#[derive(Debug, Deserialize)]
pub struct FloxerStats {
    pub query_lengths: HistogramData,
    #[serde(flatten)]
    pub seed_stats: SeedStats,
    #[serde(flatten)]
    pub anchor_stats: AnchorStats,
    #[serde(flatten)]
    pub alignment_stats: AlignmentStats,
    pub alignments_per_query: HistogramData,
    pub alignments_edit_distance: HistogramData,
}

impl FloxerStats {
    pub fn iter_general_stats_histograms(&self) -> impl Iterator<Item = &HistogramData> {
        [
            &self.query_lengths,
            &self.alignments_per_query,
            &self.alignments_edit_distance,
        ]
        .into_iter()
    }

    pub fn iter_general_metric_names(&self) -> impl Iterator<Item = &'static str> {
        [
            "Query lenghts",
            "Alignments per query",
            "Edit distances of alignments",
        ]
        .into_iter()
    }
}

#[derive(Debug, Deserialize)]
pub struct SeedStats {
    pub seed_lengths: HistogramData,
    pub errors_per_seed: HistogramData,
    pub seeds_per_query: HistogramData,
}

impl SeedStats {
    pub fn iter_histograms(&self) -> impl Iterator<Item = &HistogramData> {
        [
            &self.seed_lengths,
            &self.errors_per_seed,
            &self.seeds_per_query,
        ]
        .into_iter()
    }

    pub fn iter_metric_names(&self) -> impl Iterator<Item = &'static str> {
        ["Seed lengts", "Errors per seed", "Seeds per query"].into_iter()
    }
}

#[derive(Debug, Deserialize)]
pub struct AnchorStats {
    pub completely_excluded_queries: usize,
    pub anchors_per_non_excluded_seed: HistogramData,
    pub kept_anchors_per_partly_excluded_seed: HistogramData,
    pub raw_anchors_per_fully_excluded_seed: HistogramData,
    pub anchors_per_query_from_non_excluded_seeds: HistogramData,
    pub excluded_raw_anchors_per_query: HistogramData,
}

impl AnchorStats {
    pub fn iter_histograms(&self) -> impl Iterator<Item = &HistogramData> {
        [
            &self.anchors_per_non_excluded_seed,
            &self.kept_anchors_per_partly_excluded_seed,
            &self.raw_anchors_per_fully_excluded_seed,
            &self.anchors_per_query_from_non_excluded_seeds,
            &self.excluded_raw_anchors_per_query,
        ]
        .into_iter()
    }

    pub fn iter_metric_names(&self) -> impl Iterator<Item = &'static str> {
        [
            "Anchors per non excluded seed",
            "Kept anchors per partly excluded seed",
            "Raw anchors per fully excluded seed",
            "Anchors per query from non excluded seeds",
            "Excluded raw anchors per query",
        ]
        .into_iter()
    }
}

#[derive(Debug, Deserialize)]
pub struct AlignmentStats {
    pub reference_span_sizes_aligned_of_inner_nodes: HistogramData,
    pub reference_span_sizes_alignment_avoided_of_inner_nodes: HistogramData,
    pub reference_span_sizes_aligned_of_roots: HistogramData,
    pub reference_span_sizes_alignment_avoided_of_roots: HistogramData,
}

impl AlignmentStats {
    pub fn iter_histograms(&self) -> impl Iterator<Item = &HistogramData> {
        [
            &self.reference_span_sizes_aligned_of_inner_nodes,
            &self.reference_span_sizes_alignment_avoided_of_inner_nodes,
            &self.reference_span_sizes_aligned_of_roots,
            &self.reference_span_sizes_alignment_avoided_of_roots,
        ]
        .into_iter()
    }

    pub fn iter_metric_names(&self) -> impl Iterator<Item = &'static str> {
        [
            "Ref span sizes aligned inner",
            "Ref span sizes alignment avoided inner",
            "Ref span sizes aligned roots",
            "Ref span sizes alignment avoided roots",
        ]
        .into_iter()
    }
}

#[derive(Debug, Deserialize)]
pub struct HistogramData {
    pub num_values: usize,
    pub thresholds: Vec<usize>,
    pub occurrences: Vec<usize>,
    #[serde(flatten)]
    pub descriptive_stats: Option<DescriptiveStats>,
}

impl HistogramData {
    pub fn axis_names(&self) -> Vec<String> {
        self.thresholds
            .iter()
            .map(|threshold| format!("<= {threshold}"))
            .chain([String::from("<= inf")])
            .collect()
    }

    pub fn occurrences_as_i32(&self) -> Vec<i32> {
        self.occurrences.iter().map(|value| *value as i32).collect()
    }
}

#[derive(Debug, Deserialize)]
pub struct DescriptiveStats {
    pub min_value: usize,
    pub mean: f64,
    pub max_value: usize,
}

#[derive(Debug, Deserialize)]
pub struct ResourceMetrics {
    pub wall_clock_seconds: f64,
    pub user_cpu_seconds: f64,
    pub system_cpu_seconds: f64,
    pub peak_memory_kilobytes: usize,
    pub average_memory_kilobytes: usize,
}
