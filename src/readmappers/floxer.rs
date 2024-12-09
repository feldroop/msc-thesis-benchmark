use crate::{
    analyze_mapped_reads::{
        analyze_alignments_simple, verify_simulated_dataset, SimpleMappedReadsStats,
    },
    benchmarks::ProfileConfig,
    cli::BenchmarkConfig,
    config::BenchmarkSuiteConfig,
    folder_structure::{BenchmarkFolder, BenchmarkInstanceFolder},
};

use std::{fs, process::Command};

use super::{IndexStrategy, Queries, Reference, ResourceMetrics};
use anyhow::{bail, Result};
use serde::Deserialize;
use strum::{Display, EnumIter};

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
}

#[derive(Debug, Copy, Clone, EnumIter, Display)]
#[strum(serialize_all = "snake_case")]
pub enum AnchorChoiceStrategy {
    RoundRobin,
    FullGroups,
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

#[derive(Debug, Copy, Clone, Display)]
#[strum(serialize_all = "snake_case")]
pub enum StatsInputHint {
    RealNanopore,
    Simulated,
}

#[derive(Debug, Copy, Clone)]
pub enum CigarOutput {
    On,
    Off,
}

#[derive(Debug)]
pub struct FloxerAlgorithmConfig {
    pub index_strategy: IndexStrategy,
    pub query_errors: QueryErrors,
    pub pex_seed_errors: u8,
    pub max_num_anchors_hard: u64,
    pub max_num_anchors_soft: u64,
    pub anchor_group_order: AnchorGroupOrder,
    pub anchor_choice_strategy: AnchorChoiceStrategy,
    pub seed_sampling_step_size: u16,
    pub pex_tree_construction: PexTreeConstruction,
    pub interval_optimization: IntervalOptimization,
    pub extra_verification_ratio: f64,
    pub verification_algorithm: VerificationAlgorithm,
    pub num_anchors_per_verification_task: usize,
    pub num_threads: u16,
}

pub const DEFAULT_ERROR_RATE: f64 = 0.09;
pub const HIGH_ERROR_RATE: f64 = 0.15;

impl Default for FloxerAlgorithmConfig {
    fn default() -> Self {
        FloxerAlgorithmConfig {
            index_strategy: IndexStrategy::ReadFromDiskIfStored,
            query_errors: QueryErrors::Rate(DEFAULT_ERROR_RATE),
            pex_seed_errors: 2,
            max_num_anchors_hard: u64::MAX,
            max_num_anchors_soft: 100,
            anchor_group_order: AnchorGroupOrder::CountFirst,
            anchor_choice_strategy: AnchorChoiceStrategy::RoundRobin,
            seed_sampling_step_size: 1,
            pex_tree_construction: PexTreeConstruction::BottomUp,
            interval_optimization: IntervalOptimization::On,
            extra_verification_ratio: 0.1,
            verification_algorithm: VerificationAlgorithm::Hierarchical,
            num_anchors_per_verification_task: 3_000,
            num_threads: super::NUM_THREADS_FOR_READMAPPERS,
        }
    }
}

// API to configure a floxer benchmark run.
// the output path will be determined from the other parameters
#[derive(Debug)]
pub struct FloxerConfig {
    pub name: String,
    pub reference: Reference,
    pub queries: Queries,
    pub only_analysis: bool,
    pub algorithm_config: FloxerAlgorithmConfig,
    pub cigar_output: CigarOutput,
}

impl From<&BenchmarkConfig> for FloxerConfig {
    fn from(value: &BenchmarkConfig) -> Self {
        FloxerConfig {
            name: "unnamed_instance".into(),
            reference: value.reference,
            queries: value.queries,
            only_analysis: value.only_analysis,
            algorithm_config: Default::default(),
            cigar_output: CigarOutput::Off,
        }
    }
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
        let instance_folder =
            if self.only_analysis && benchmark_folder.most_recect_previous_run_folder().exists() {
                BenchmarkInstanceFolder::most_recent_previous_run(benchmark_folder, &self.name)?
            } else {
                let instance_folder = BenchmarkInstanceFolder::new(benchmark_folder, &self.name)?;

                self.actually_run(
                    profile_config,
                    &instance_folder,
                    suite_config,
                    benchmark_name,
                )?;

                benchmark_folder.create_or_update_link_to_most_recent()?;

                instance_folder
            };

        if let ProfileConfig::On = profile_config {
            create_profile(&instance_folder)?;
        }

        let stats_file_str = fs::read_to_string(instance_folder.stats_path)?;
        let stats: FloxerStats = toml::from_str(&stats_file_str)?;

        let timings_file_str = fs::read_to_string(instance_folder.timing_path)?;
        let resource_metrics: ResourceMetrics = toml::from_str(&timings_file_str)?;

        let mapped_read_stats = analyze_alignments_simple(&instance_folder.mapped_reads_bam_path)?;

        if self.queries == Queries::Simulated && self.reference == Reference::Simulated {
            let verification_summary =
                verify_simulated_dataset(&instance_folder.mapped_reads_bam_path, suite_config)?;

            verification_summary.print_if_missed();
        }

        Ok(FloxerRunResult {
            benchmark_instance_name: self.name.clone(),
            stats,
            resource_metrics,
            mapped_read_stats,
        })
    }

    fn actually_run(
        &self,
        profile_config: ProfileConfig,
        instance_folder: &BenchmarkInstanceFolder,
        suite_config: &BenchmarkSuiteConfig,
        benchmark_name: &str,
    ) -> Result<()> {
        let mut command = match profile_config {
            ProfileConfig::Off => Command::new("/usr/bin/time"),
            ProfileConfig::On => {
                let mut command = Command::new("perf");
                command
                    .arg("record")
                    .arg("-o")
                    .arg(&instance_folder.perf_data_path)
                    .arg("-F")
                    .arg("100")
                    .arg("--call-graph")
                    .arg("dwarf,16384")
                    .arg("-g") // both user and kernel space
                    .arg("--")
                    .arg("/usr/bin/time");
                command
            }
        };

        super::add_time_args(&mut command, &instance_folder.timing_path);
        let reference_path = self.reference.path(suite_config);
        let queries_path = self.queries.path(suite_config);

        command
            .arg(&suite_config.readmapper_binaries.floxer)
            .arg("--reference")
            .arg(reference_path)
            .arg("--queries")
            .arg(queries_path)
            .arg("--output")
            .arg(&instance_folder.mapped_reads_bam_path)
            .arg("--logfile")
            .arg(&instance_folder.logfile_path)
            .arg("--stats")
            .arg(&instance_folder.stats_path);

        if self.algorithm_config.index_strategy == IndexStrategy::ReadFromDiskIfStored {
            let mut index_path = suite_config.index_folder();
            let index_file_name = format!("floxer-index-{}.flxi", self.reference);
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
            "--max-anchors-hard",
            &self.algorithm_config.max_num_anchors_hard.to_string(),
            "--max-anchors-soft",
            &self.algorithm_config.max_num_anchors_soft.to_string(),
            "--anchor-group-order",
            &self.algorithm_config.anchor_group_order.to_string(),
            "--anchor-choice-strategy",
            &self.algorithm_config.anchor_choice_strategy.to_string(),
            "--seed-sampling-step-size",
            &self.algorithm_config.seed_sampling_step_size.to_string(),
            "--extra-verification-ratio",
            &self.algorithm_config.extra_verification_ratio.to_string(),
            "--threads",
            &self.algorithm_config.num_threads.to_string(),
            "--num-anchors-per-task",
            &self
                .algorithm_config
                .num_anchors_per_verification_task
                .to_string(),
        ]);

        if let PexTreeConstruction::BottomUp = self.algorithm_config.pex_tree_construction {
            command.arg("--bottom-up-pex-tree");
        }

        if let IntervalOptimization::On = self.algorithm_config.interval_optimization {
            command.args(["--interval-optimization"]);
        }

        if let VerificationAlgorithm::DirectFull = self.algorithm_config.verification_algorithm {
            command.arg("--direct-full-verification");
        }

        if let Some(stats_input_hint) = self.queries.floxer_stats_input_hint() {
            command.arg("--stats-input-hint");
            command.arg(stats_input_hint.to_string());
        }

        if let CigarOutput::Off = self.cigar_output {
            command.arg("--without-cigar");
        }

        println!(
            "- Running the benchmark: {}",
            self.full_name(benchmark_name)
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

        Ok(())
    }

    fn full_name(&self, benchmark_name: &str) -> String {
        format!("{}__{}", benchmark_name, self.name)
    }
}

fn create_profile(instance_folder: &BenchmarkInstanceFolder) -> Result<()> {
    let flamegraph_output = Command::new("flamegraph")
        .arg("--deterministic")
        .arg("--perfdata")
        .arg(&instance_folder.perf_data_path)
        .arg("--output")
        .arg(&instance_folder.flamegraph_path)
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
    pub mapped_read_stats: SimpleMappedReadsStats,
}

#[derive(Debug, Deserialize)]
pub struct FloxerStats {
    pub query_lengths: HistogramData,
    #[serde(flatten)]
    pub seed_stats: SeedStats,
    #[serde(flatten)]
    pub anchor_stats_per_query: AnchorStatsPerQuery,
    #[serde(flatten)]
    pub anchor_stats_per_seed: AnchorStatsPerSeed,
    #[serde(flatten)]
    pub alignment_stats: AlignmentStats,
    pub alignments_per_query: HistogramData,
    pub alignments_edit_distance: HistogramData,
    pub milliseconds_spent_in_search_per_query: HistogramData,
    pub milliseconds_spent_in_verification_per_query: HistogramData,
}

impl FloxerStats {
    pub fn iter_general_stats_histograms(&self) -> impl Iterator<Item = &HistogramData> {
        [
            &self.query_lengths,
            &self.alignments_per_query,
            &self.alignments_edit_distance,
            &self.milliseconds_spent_in_search_per_query,
            &self.milliseconds_spent_in_verification_per_query,
        ]
        .into_iter()
    }

    pub fn iter_general_metric_names(&self) -> impl Iterator<Item = &'static str> {
        [
            "Query lenghts",
            "Alignments per query",
            "Edit distances of alignments",
            "Milliseconds spent in search per query",
            "Milliseconds spent in verification per query",
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
        ["Seed lengths", "Errors per seed", "Seeds per query"].into_iter()
    }
}

#[derive(Debug, Deserialize)]
pub struct AnchorStatsPerQuery {
    pub completely_excluded_queries: usize,
    pub fully_excluded_seeds_per_query: HistogramData,
    pub kept_anchors_per_query: HistogramData,
    pub excluded_raw_anchors_by_soft_cap_per_query: HistogramData,
    pub excluded_raw_anchors_by_erase_useless_per_query: HistogramData,
}

impl AnchorStatsPerQuery {
    pub fn iter_histograms(&self) -> impl Iterator<Item = &HistogramData> {
        [
            &self.fully_excluded_seeds_per_query,
            &self.kept_anchors_per_query,
            &self.excluded_raw_anchors_by_soft_cap_per_query,
            &self.excluded_raw_anchors_by_erase_useless_per_query,
        ]
        .into_iter()
    }

    pub fn iter_metric_names(&self) -> impl Iterator<Item = &'static str> {
        [
            "Fully excluded seeds per query",
            "Anchors per query from non excluded seeds",
            "Excluded raw anchors by soft cap per query",
            "Excluded raw anchors by erase useless per query",
        ]
        .into_iter()
    }
}

#[derive(Debug, Deserialize)]
pub struct AnchorStatsPerSeed {
    pub kept_anchors_per_kept_seed: HistogramData,
    pub excluded_raw_anchors_by_soft_cap_per_kept_seed: HistogramData,
    pub excluded_raw_anchors_by_erase_useless_per_kept_seed: HistogramData,
}

impl AnchorStatsPerSeed {
    pub fn iter_histograms(&self) -> impl Iterator<Item = &HistogramData> {
        [
            &self.kept_anchors_per_kept_seed,
            &self.excluded_raw_anchors_by_soft_cap_per_kept_seed,
            &self.excluded_raw_anchors_by_erase_useless_per_kept_seed,
        ]
        .into_iter()
    }

    pub fn iter_metric_names(&self) -> impl Iterator<Item = &'static str> {
        [
            "Kept anchors per kept seed",
            "Excluded raw anchors by soft cap per kept seed",
            "Excluded raw anchors by erase useless per kept seed",
        ]
        .into_iter()
    }
}

#[derive(Debug, Deserialize)]
pub struct AlignmentStats {
    pub reference_span_sizes_aligned_of_inner_nodes: HistogramData,
    pub reference_span_sizes_aligned_of_roots: HistogramData,
    pub reference_span_sizes_alignment_avoided_of_roots: HistogramData,
}

impl AlignmentStats {
    pub fn iter_histograms(&self) -> impl Iterator<Item = &HistogramData> {
        [
            &self.reference_span_sizes_aligned_of_inner_nodes,
            &self.reference_span_sizes_aligned_of_roots,
            &self.reference_span_sizes_alignment_avoided_of_roots,
        ]
        .into_iter()
    }

    pub fn iter_metric_names(&self) -> impl Iterator<Item = &'static str> {
        [
            "Ref span sizes aligned inner",
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

#[derive(Debug, Deserialize, Clone)]
pub struct DescriptiveStats {
    pub min_value: usize,
    pub mean: f64,
    pub max_value: usize,
}
