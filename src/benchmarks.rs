use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::config::BenchmarkSuiteConfig;
use crate::folder_structure::BenchmarkFolder;
use crate::plots;
use crate::readmappers::floxer::{
    AnchorGroupOrder, FloxerAlgorithmConfig, FloxerConfig, FloxerRunResult, IntervalOptimization,
    PexTreeConstruction, QueryErrors, VerificationAlgorithm,
};
use crate::readmappers::minimap::MinimapConfig;
use crate::readmappers::{IndexStrategy, Queries, Reference};

use anyhow::{bail, Result};
use clap::ValueEnum;
use strum::{EnumIter, IntoEnumIterator};

static UNNAMED_BENCHMARK_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy, EnumIter, ValueEnum, PartialEq, Eq, Hash)]
pub enum Benchmark {
    AllowedIntervalOverlapRatio,
    AnchorGroupOrder,
    AnchorsPerVerificationTask,
    Debug,
    ExtraVerificationRatio,
    IndexBuild,
    IntervalOptimization,
    MaxAnchors,
    Minimap,
    PexSeedErrors,
    PexTreeBuilding,
    ProblemQuery,
    Profile,
    QueryErrorRate,
    Threads,
    VerificationAlgorithm,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ProfileConfig {
    #[default]
    Off,
    On,
}

impl Benchmark {
    pub fn run(&self, suite_config: &BenchmarkSuiteConfig) -> Result<()> {
        match *self {
            Benchmark::AllowedIntervalOverlapRatio => allowed_interval_overlap_ratio(suite_config),
            Benchmark::AnchorGroupOrder => anchor_group_order(suite_config),
            Benchmark::AnchorsPerVerificationTask => anchors_per_verification_task(suite_config),
            Benchmark::Debug => debug_benchmark(suite_config),
            Benchmark::ExtraVerificationRatio => extra_verification_ratio(suite_config),
            Benchmark::IndexBuild => index_build(suite_config),
            Benchmark::IntervalOptimization => interval_optimization(suite_config),
            Benchmark::MaxAnchors => max_anchors(suite_config),
            Benchmark::Minimap => minimap(suite_config),
            Benchmark::PexSeedErrors => pex_seed_errors(suite_config),
            Benchmark::PexTreeBuilding => pex_tree_building(suite_config),
            Benchmark::Profile => profile(suite_config),
            Benchmark::ProblemQuery => problem_query(suite_config),
            Benchmark::QueryErrorRate => query_error_rate(suite_config),
            Benchmark::Threads => threads(suite_config),
            Benchmark::VerificationAlgorithm => verification_algorithm(suite_config),
        }
    }
}

pub fn run_benchmarks<I: IntoIterator<Item = Benchmark>>(
    benchmarks: I,
    suite_config: &BenchmarkSuiteConfig,
) -> Result<()> {
    let skip_for_now: HashSet<_> = [Benchmark::VerificationAlgorithm].into_iter().collect();

    let mut num_error_runs = 0;
    for benchmark in benchmarks.into_iter() {
        if skip_for_now.contains(&benchmark) {
            continue;
        }

        if let Err(err) = benchmark.run(suite_config) {
            println!("{}", err);
            num_error_runs += 1;
        } else {
            continue;
        }
    }

    if num_error_runs != 0 {
        bail!("errors occurred in at least {num_error_runs} run(s)")
    }

    Ok(())
}

pub fn run_all(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    run_benchmarks(Benchmark::iter(), suite_config)
}

struct BenchmarkResult {
    benchmark_name: String,
    folder: BenchmarkFolder,
    floxer_results: Vec<FloxerRunResult>,
}

impl BenchmarkResult {
    pub fn plot_seed_stats(&self, suite_config: &BenchmarkSuiteConfig) {
        plots::plot_histogram_data_in_grid(
            self.floxer_results
                .iter()
                .map(|run| run.stats.seed_stats.iter_histograms()),
            &format!("{} seed stats", self.benchmark_name),
            self.floxer_results
                .iter()
                .map(|run| &run.benchmark_instance_name),
            self.floxer_results[0].stats.seed_stats.iter_metric_names(),
            &self.folder,
            suite_config,
        );
    }

    pub fn plot_anchor_stats(&self, suite_config: &BenchmarkSuiteConfig) {
        plots::plot_histogram_data_in_grid(
            self.floxer_results
                .iter()
                .map(|run| run.stats.anchor_stats.iter_histograms()),
            &format!("{} anchor stats", self.benchmark_name),
            self.floxer_results.iter().map(|run| {
                format!(
                    "{} (#fully exc. query: {})",
                    run.benchmark_instance_name, run.stats.anchor_stats.completely_excluded_queries
                )
            }),
            self.floxer_results[0]
                .stats
                .anchor_stats
                .iter_metric_names(),
            &self.folder,
            suite_config,
        );
    }

    pub fn plot_alignment_stats(&self, suite_config: &BenchmarkSuiteConfig) {
        plots::plot_histogram_data_in_grid(
            self.floxer_results
                .iter()
                .map(|run| run.stats.alignment_stats.iter_histograms()),
            &format!("{} alignment stats", self.benchmark_name),
            self.floxer_results
                .iter()
                .map(|run| &run.benchmark_instance_name),
            self.floxer_results[0]
                .stats
                .alignment_stats
                .iter_metric_names(),
            &self.folder,
            suite_config,
        );
    }

    pub fn plot_general_stats(&self, suite_config: &BenchmarkSuiteConfig) {
        plots::plot_histogram_data_in_grid(
            self.floxer_results
                .iter()
                .map(|run| run.stats.iter_general_stats_histograms()),
            &format!("{} general stats", self.benchmark_name),
            self.floxer_results
                .iter()
                .map(|run| &run.benchmark_instance_name),
            self.floxer_results[0].stats.iter_general_metric_names(),
            &self.folder,
            suite_config,
        );
    }

    pub fn plot_mapped_reads_stats(&self, suite_config: &BenchmarkSuiteConfig) {
        plots::plot_mapped_reads_stats(
            self.floxer_results.iter().map(|res| &res.mapped_read_stats),
            &format!("{} mapped reads stats", self.benchmark_name),
            self.floxer_results
                .iter()
                .map(|run| &run.benchmark_instance_name),
            &self.folder,
            suite_config,
        );
    }
}

fn allowed_interval_overlap_ratio(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    let res = FloxerParameterBenchmark::from_iter([1.0, 0.95, 0.9, 0.5, 0.01].into_iter().map(
        |allowed_interval_overlap_ratio| {
            FloxerConfig {
                algorithm_config: FloxerAlgorithmConfig {
                    allowed_interval_overlap_ratio,
                    ..Default::default()
                },
                name: Some(
                    format!("allowed_interval_overlap_ratio_{allowed_interval_overlap_ratio}")
                        .replace('.', "_"),
                ),
                ..Default::default()
            }
        },
    ))
    .name("allowed_interval_overlap_ratio")
    .run(suite_config)?;

    res.plot_alignment_stats(suite_config);

    Ok(())
}

fn anchor_group_order(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    let res =
        FloxerParameterBenchmark::from_iter(AnchorGroupOrder::iter().map(|anchor_group_order| {
            FloxerConfig {
                algorithm_config: FloxerAlgorithmConfig {
                    anchor_group_order,
                    ..Default::default()
                },
                name: Some(anchor_group_order.to_string()),
                ..Default::default()
            }
        }))
        .name("anchor_group_order")
        .run(suite_config)?;

    res.plot_anchor_stats(suite_config);

    Ok(())
}

fn anchors_per_verification_task(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    FloxerParameterBenchmark::from_iter([3000, 10_000, 1_000_000_000].into_iter().map(
        |num_anchors_per_verification_task| FloxerConfig {
            algorithm_config: FloxerAlgorithmConfig {
                num_anchors_per_verification_task,
                ..Default::default()
            },
            name: Some(format!(
                "anchors_per_task_{num_anchors_per_verification_task}"
            )),
            ..Default::default()
        },
    ))
    .name("anchors_per_verification_task")
    .run(suite_config)?;

    Ok(())
}

fn debug_benchmark(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    let _ = FloxerParameterBenchmark::from_iter(PexTreeConstruction::iter().map(
        |pex_tree_construction| FloxerConfig {
            reference: Reference::Debug,
            queries: Queries::Debug,
            algorithm_config: FloxerAlgorithmConfig {
                pex_tree_construction,
                extra_verification_ratio: 2.0,
                num_threads: 1,
                pex_seed_errors: 1,
                query_errors: QueryErrors::Exact(2),
                ..Default::default()
            },
            name: Some(pex_tree_construction.to_string()),
        },
    ))
    .name("debug")
    .run(suite_config)?;

    Ok(())
}

fn extra_verification_ratio(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    let res = FloxerParameterBenchmark::from_iter([0.01, 0.02, 0.05, 0.1, 0.2].into_iter().map(
        |extra_verification_ratio| FloxerConfig {
            algorithm_config: FloxerAlgorithmConfig {
                extra_verification_ratio,
                ..Default::default()
            },
            name: Some(
                format!("extra_verification_ratio_{extra_verification_ratio}").replace('.', "_"),
            ),
            ..Default::default()
        },
    ))
    .name("extra_verification_ratio")
    .run(suite_config)?;

    res.plot_alignment_stats(suite_config);

    Ok(())
}

fn index_build(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    let _ = FloxerParameterBenchmark::from_iter([FloxerConfig {
        algorithm_config: FloxerAlgorithmConfig {
            index_strategy: IndexStrategy::AlwaysRebuild,
            ..Default::default()
        },
        queries: Queries::Debug, // here we only care about the index building and skip the queries
        ..Default::default()
    }])
    .name("index_build")
    .run(suite_config)?;

    Ok(())
}

fn interval_optimization(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    let res = FloxerParameterBenchmark::from_iter(IntervalOptimization::iter().map(
        |interval_optimization| FloxerConfig {
            queries: Queries::HumanWgsNanoporeSmall,
            algorithm_config: FloxerAlgorithmConfig {
                interval_optimization,
                ..Default::default()
            },
            name: Some(interval_optimization.to_string()),
            ..Default::default()
        },
    ))
    .name("interval_optimization")
    .run(suite_config)?;

    res.plot_alignment_stats(suite_config);

    Ok(())
}

fn max_anchors(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    let res = FloxerParameterBenchmark::from_iter([20, 50, 100, 200].into_iter().map(
        |max_num_anchors| FloxerConfig {
            algorithm_config: FloxerAlgorithmConfig {
                max_num_anchors,
                ..Default::default()
            },
            name: Some(format!("max_anchors_{max_num_anchors}")),
            ..Default::default()
        },
    ))
    .name("max_anchors")
    .run(suite_config)?;

    res.plot_seed_stats(suite_config);
    res.plot_anchor_stats(suite_config);

    Ok(())
}

fn minimap(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    let name = "vs_minimap";
    let folder = BenchmarkFolder::new(&suite_config.output_folder, name);
    let floxer_res =
        FloxerConfig::default().run(&folder, "floxer", suite_config, ProfileConfig::Off)?;

    let minimap_res = MinimapConfig::default().run(&folder, suite_config)?;

    plots::plot_resource_metrics(
        name,
        [
            (&floxer_res.resource_metrics, "floxer"),
            (&minimap_res.map_resource_metrics, "minimap"),
        ]
        .into_iter(),
        &folder,
        suite_config,
    );

    // TODO create function that runs and parses compare_readmapper_output and creates nice visualizations (sankey + more basic)
    Ok(())
}

fn pex_seed_errors(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    let res = FloxerParameterBenchmark::from_iter((0..4).map(|pex_seed_errors| FloxerConfig {
        algorithm_config: FloxerAlgorithmConfig {
            pex_seed_errors,
            ..Default::default()
        },
        name: Some(format!("pex_seed_errors_{pex_seed_errors}")),
        ..Default::default()
    }))
    .name("pex_seed_errors")
    .run(suite_config)?;

    res.plot_seed_stats(suite_config);
    res.plot_anchor_stats(suite_config);

    Ok(())
}

fn pex_tree_building(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    let res = FloxerParameterBenchmark::from_iter(
        [
            PexTreeConstruction::TopDown,
            PexTreeConstruction::TopDown,
            PexTreeConstruction::BottomUp,
            PexTreeConstruction::BottomUp,
        ]
        .into_iter()
        .zip([1, 2, 1, 2])
        .map(|(pex_tree_construction, pex_seed_errors)| FloxerConfig {
            algorithm_config: FloxerAlgorithmConfig {
                pex_tree_construction,
                pex_seed_errors,
                ..Default::default()
            },
            name: Some(format!(
                "{}_{}_seed_errors",
                pex_tree_construction, pex_seed_errors
            )),
            ..Default::default()
        }),
    )
    .name("pex_tree_building")
    .run(suite_config)?;

    res.plot_seed_stats(suite_config);
    res.plot_anchor_stats(suite_config);

    Ok(())
}

fn profile(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    let _ = FloxerParameterBenchmark::from_iter([Default::default()])
        .name("profile")
        .with_profile()
        .run(suite_config);

    Ok(())
}

fn problem_query(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    let _ = FloxerParameterBenchmark::from_iter([FloxerConfig {
        queries: Queries::ProblemQuery,
        algorithm_config: FloxerAlgorithmConfig {
            pex_seed_errors: 1,
            allowed_interval_overlap_ratio: 0.8,
            ..Default::default()
        },
        ..Default::default()
    }])
    .name("problem_query")
    .with_profile()
    .run(suite_config);

    Ok(())
}

fn query_error_rate(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    let res = FloxerParameterBenchmark::from_iter([0.05, 0.07, 0.09, 0.11, 0.13].into_iter().map(
        |query_error_ratio| FloxerConfig {
            algorithm_config: FloxerAlgorithmConfig {
                query_errors: QueryErrors::Rate(query_error_ratio),
                ..Default::default()
            },
            name: Some(format!("query_error_rate_{query_error_ratio}").replace('.', "_")),
            ..Default::default()
        },
    ))
    .name("query_error_rate")
    .run(suite_config)?;

    res.plot_seed_stats(suite_config);
    res.plot_anchor_stats(suite_config);

    Ok(())
}

fn threads(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    FloxerParameterBenchmark::from_iter([4, 8, 12, 16].into_iter().map(|num_threads| {
        FloxerConfig {
            algorithm_config: FloxerAlgorithmConfig {
                num_threads,
                ..Default::default()
            },
            name: Some(format!("num_threads_{num_threads}")),
            ..Default::default()
        }
    }))
    .name("num_threads")
    .run(suite_config)?;

    Ok(())
}

fn verification_algorithm(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    let res = FloxerParameterBenchmark::from_iter(VerificationAlgorithm::iter().map(
        |verification_algorithm| FloxerConfig {
            queries: Queries::HumanWgsNanoporeSmall,
            algorithm_config: FloxerAlgorithmConfig {
                verification_algorithm,
                ..Default::default()
            },
            name: Some(verification_algorithm.to_string()),
            ..Default::default()
        },
    ))
    .name("verification_algorithm")
    .run(suite_config)?;

    res.plot_alignment_stats(suite_config);

    Ok(())
}

#[derive(Default)]
struct FloxerParameterBenchmark {
    floxer_configs: Vec<FloxerConfig>,
    benchmark_name: String,
    profile_config: ProfileConfig,
}

impl FromIterator<FloxerConfig> for FloxerParameterBenchmark {
    fn from_iter<T: IntoIterator<Item = FloxerConfig>>(iter: T) -> Self {
        Self {
            floxer_configs: iter.into_iter().collect(),
            benchmark_name: format!(
                "benchmark_{}",
                UNNAMED_BENCHMARK_ID.fetch_add(1, Ordering::SeqCst)
            ),
            profile_config: ProfileConfig::Off,
        }
    }
}

impl FloxerParameterBenchmark {
    fn name<S: AsRef<str>>(mut self, benchmark_name: S) -> Self {
        self.benchmark_name = benchmark_name.as_ref().to_owned();
        self
    }

    fn with_profile(mut self) -> Self {
        self.profile_config = ProfileConfig::On;
        self
    }

    fn run(&self, suite_config: &BenchmarkSuiteConfig) -> Result<BenchmarkResult> {
        let benchmark_folder =
            BenchmarkFolder::new(&suite_config.output_folder, &self.benchmark_name);

        let mut floxer_results = Vec::new();
        let mut instance_names = Vec::new();

        for (index, floxer_config) in self.floxer_configs.iter().enumerate() {
            let res = floxer_config.run(
                &benchmark_folder,
                &self.benchmark_name,
                suite_config,
                self.profile_config,
            )?;

            floxer_results.push(res);
            instance_names.push(
                floxer_config
                    .name
                    .clone()
                    .unwrap_or_else(|| format!("benchmark_instance_{index}")),
            );
        }

        plots::plot_resource_metrics(
            &self.benchmark_name,
            floxer_results
                .iter()
                .map(|res| (&res.resource_metrics, res.benchmark_instance_name.as_str())),
            &benchmark_folder,
            suite_config,
        );

        let res = BenchmarkResult {
            benchmark_name: self.benchmark_name.to_owned(),
            folder: benchmark_folder,
            floxer_results,
        };

        res.plot_general_stats(suite_config);
        res.plot_mapped_reads_stats(suite_config);

        Ok(res)
    }
}
