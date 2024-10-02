use crate::{config::BenchmarkSuiteConfig, folder_structure::BenchmarkFolder};

use std::{fs, process::Command};

use anyhow::{bail, Result};
use serde::Deserialize;
use strum::{Display, EnumIter};

const TIME_TOOL_FORMAT_STRING: &str = "wall_clock_seconds = %e
user_cpu_seconds = %U
system_cpu_seconds = %S
peak_memory_kilobytes = %M
average_memory_kilobytes = %K";

pub enum Reference {
    HumanGenomeHg38,
}

impl Reference {
    fn name_for_files(&self) -> &str {
        match self {
            Self::HumanGenomeHg38 => "human-genome-hg38",
        }
    }
}
pub enum Queries {
    HumanWgsNanopore,
}

pub enum IndexStrategy {
    AlwaysRebuild,
    ReadFromDiskIfStored,
}

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
            num_threads: 8,
        }
    }
}

// API to configure a floxer benchmark run.
// the output path will be determined from the other parameters
pub struct FloxerConfig {
    pub reference: Reference,
    pub queries: Queries,
    pub algorithm_config: FloxerAlgorithmConfig,
}

impl Default for FloxerConfig {
    fn default() -> Self {
        Self {
            reference: Reference::HumanGenomeHg38,
            queries: Queries::HumanWgsNanopore,
            algorithm_config: Default::default(),
        }
    }
}

impl FloxerConfig {
    // benchmark name should be a valid and good name for a folder
    pub fn run(
        &self,
        benchmark_folder: &BenchmarkFolder,
        benchmark_name: &str,
        instance_name: Option<&str>,
        suite_config: &BenchmarkSuiteConfig,
    ) -> Result<FloxerResult> {
        let mut output_folder = benchmark_folder.get().to_path_buf();

        if let Some(instance_name) = instance_name {
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

        let mut command = Command::new("/usr/bin/time");

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
            if let Some(instance_name) = instance_name {
                format!(" - {instance_name}")
            } else {
                String::new()
            }
        );
        let floxer_proc_output = command.output()?;

        if !floxer_proc_output.status.success() || !floxer_proc_output.stdout.is_empty() {
            bail!(
                "Something went wrong. Floxer process output: {:?}",
                floxer_proc_output
            );
        }

        let stats_file_str = fs::read_to_string(stats_path)?;
        let stats: FloxerStats = toml::from_str(&stats_file_str)?;

        let timings_file_str = fs::read_to_string(timing_path)?;
        let resource_metrics: ResourceMetrics = toml::from_str(&timings_file_str)?;

        Ok(FloxerResult {
            benchmark_instance_name: instance_name
                .map_or_else(|| String::from("floxer"), |name| name.to_owned()),
            stats,
            resource_metrics,
        })
    }
}

#[derive(Debug)]
pub struct FloxerResult {
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

#[derive(Debug, Deserialize)]
pub struct SeedStats {
    pub completely_excluded_queries: usize,
    pub seed_lengths: HistogramData,
    pub errors_per_seed: HistogramData,
    pub seeds_per_query: HistogramData,
}

#[derive(Debug, Deserialize)]
pub struct AnchorStats {
    pub anchors_per_non_excluded_seed: HistogramData,
    pub kept_anchors_per_partly_excluded_seed: HistogramData,
    pub raw_anchors_per_fully_excluded_seed: HistogramData,
    pub anchors_per_query_from_non_excluded_seeds: HistogramData,
    pub excluded_raw_anchors_per_query: HistogramData,
}

#[derive(Debug, Deserialize)]
pub struct AlignmentStats {
    pub reference_span_sizes_aligned_of_inner_nodes: HistogramData,
    pub reference_span_sizes_alignment_avoided_of_inner_nodes: HistogramData,
    pub reference_span_sizes_aligned_of_roots: HistogramData,
    pub reference_span_sizes_alignment_avoided_of_roots: HistogramData,
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
