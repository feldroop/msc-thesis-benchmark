use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::analyze_mapped_reads::analyze_alignments_detailed_comparison;
use crate::cli::BenchmarkConfig;
use crate::config::BenchmarkSuiteConfig;
use crate::folder_structure::BenchmarkFolder;
use crate::plots;
use crate::readmappers::floxer::{
    self, AnchorGroupOrder, FloxerAlgorithmConfig, FloxerConfig, FloxerRunResult,
    IntervalOptimization, PexTreeConstruction, QueryErrors, VerificationAlgorithm,
};
use crate::readmappers::minimap::MinimapConfig;
use crate::readmappers::{IndexStrategy, Queries, Reference};

use anyhow::{bail, Result};
use clap::ValueEnum;
use strum::{EnumIter, IntoEnumIterator};

static UNNAMED_BENCHMARK_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy, EnumIter, ValueEnum, PartialEq, Eq, Hash)]
pub enum Benchmark {
    AnchorGroupOrder,
    AnchorsPerVerificationTask,
    Debug,
    DefaultParams,
    ExtraVerificationRatio,
    IndexBuild,
    IntervalOptimization,
    MaxAnchors,
    Minimap,
    MinimapHighErrorRate,
    PexSeedErrors,
    PexSeedErrorsHighErrorRate,
    PexSeedErrorsNoMaxAnchors,
    PexSeedErrorsNoMaxAnchorsAndHighErrorRate,
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
    pub fn run(
        &self,
        suite_config: &BenchmarkSuiteConfig,
        benchmark_config: &BenchmarkConfig,
    ) -> Result<()> {
        match *self {
            Benchmark::AnchorGroupOrder => anchor_group_order(suite_config, benchmark_config),
            Benchmark::AnchorsPerVerificationTask => {
                anchors_per_verification_task(suite_config, benchmark_config)
            }
            Benchmark::Debug => debug_benchmark(suite_config, benchmark_config),
            Benchmark::DefaultParams => default_params(suite_config, benchmark_config),
            Benchmark::ExtraVerificationRatio => {
                extra_verification_ratio(suite_config, benchmark_config)
            }
            Benchmark::IndexBuild => index_build(suite_config, benchmark_config),
            Benchmark::IntervalOptimization => {
                interval_optimization(suite_config, benchmark_config)
            }
            Benchmark::MaxAnchors => max_anchors(suite_config, benchmark_config),
            Benchmark::Minimap => minimap(suite_config, benchmark_config),
            Benchmark::MinimapHighErrorRate => {
                minimap_high_error_rate(suite_config, benchmark_config)
            }
            Benchmark::PexSeedErrors => pex_seed_errors(suite_config, benchmark_config),
            Benchmark::PexSeedErrorsHighErrorRate => {
                pex_seed_errors_high_error_rate(suite_config, benchmark_config)
            }
            Benchmark::PexSeedErrorsNoMaxAnchors => {
                pex_seed_errors_no_max_anchors(suite_config, benchmark_config)
            }
            Benchmark::PexSeedErrorsNoMaxAnchorsAndHighErrorRate => {
                pex_seed_errors_no_max_anchors_and_high_error_rate(suite_config, benchmark_config)
            }
            Benchmark::PexTreeBuilding => pex_tree_building(suite_config, benchmark_config),
            Benchmark::Profile => profile(suite_config, benchmark_config),
            Benchmark::ProblemQuery => problem_query(suite_config, benchmark_config),
            Benchmark::QueryErrorRate => query_error_rate(suite_config, benchmark_config),
            Benchmark::Threads => threads(suite_config, benchmark_config),
            Benchmark::VerificationAlgorithm => {
                verification_algorithm(suite_config, benchmark_config)
            }
        }
    }
}

pub fn run_benchmarks<I: IntoIterator<Item = Benchmark>>(
    benchmarks: I,
    suite_config: &BenchmarkSuiteConfig,
    benchmark_config: &BenchmarkConfig,
) -> Result<()> {
    let mut num_error_runs = 0;
    for benchmark in benchmarks.into_iter() {
        if let Err(err) = benchmark.run(suite_config, benchmark_config) {
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

pub fn run_all(
    suite_config: &BenchmarkSuiteConfig,
    benchmark_config: &BenchmarkConfig,
) -> Result<()> {
    let skip_for_now: HashSet<_> = [Benchmark::VerificationAlgorithm, Benchmark::ProblemQuery]
        .into_iter()
        .collect();

    run_benchmarks(
        Benchmark::iter().filter(|benchmark| !skip_for_now.contains(benchmark)),
        suite_config,
        benchmark_config,
    )
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

fn anchor_group_order(
    suite_config: &BenchmarkSuiteConfig,
    benchmark_config: &BenchmarkConfig,
) -> Result<()> {
    let res =
        FloxerParameterBenchmark::from_iter(AnchorGroupOrder::iter().map(|anchor_group_order| {
            FloxerConfig {
                algorithm_config: FloxerAlgorithmConfig {
                    anchor_group_order,
                    ..Default::default()
                },
                name: anchor_group_order.to_string(),
                ..From::from(benchmark_config)
            }
        }))
        .name("anchor_group_order")
        .run(suite_config, benchmark_config)?;

    res.plot_anchor_stats(suite_config);

    Ok(())
}

fn anchors_per_verification_task(
    suite_config: &BenchmarkSuiteConfig,
    benchmark_config: &BenchmarkConfig,
) -> Result<()> {
    FloxerParameterBenchmark::from_iter([1000, 3000, 10_000, 1_000_000_000].into_iter().map(
        |num_anchors_per_verification_task| FloxerConfig {
            algorithm_config: FloxerAlgorithmConfig {
                num_anchors_per_verification_task,
                ..Default::default()
            },
            name: format!("anchors_per_task_{num_anchors_per_verification_task}"),
            ..From::from(benchmark_config)
        },
    ))
    .name("anchors_per_verification_task")
    .run(suite_config, benchmark_config)?;

    Ok(())
}

fn debug_benchmark(
    suite_config: &BenchmarkSuiteConfig,
    benchmark_config: &BenchmarkConfig,
) -> Result<()> {
    let benchmark_config = benchmark_config
        .with_reference(Reference::Debug)
        .with_queries(Queries::Debug);

    let res = FloxerParameterBenchmark::from_iter(PexTreeConstruction::iter().map(
        |pex_tree_construction| FloxerConfig {
            algorithm_config: FloxerAlgorithmConfig {
                pex_tree_construction,
                extra_verification_ratio: 2.0,
                num_threads: 1,
                pex_seed_errors: 1,
                query_errors: QueryErrors::Exact(2),
                ..Default::default()
            },
            name: pex_tree_construction.to_string(),
            ..From::from(&benchmark_config)
        },
    ))
    .name("debug")
    .run(suite_config, &benchmark_config)?;

    res.plot_seed_stats(suite_config);
    res.plot_anchor_stats(suite_config);
    res.plot_alignment_stats(suite_config);

    Ok(())
}

fn default_params(
    suite_config: &BenchmarkSuiteConfig,
    benchmark_config: &BenchmarkConfig,
) -> Result<()> {
    let res = FloxerParameterBenchmark::from_iter([FloxerConfig::from(benchmark_config)])
        .name("default")
        .run(suite_config, benchmark_config)?;

    res.plot_seed_stats(suite_config);
    res.plot_anchor_stats(suite_config);
    res.plot_alignment_stats(suite_config);

    Ok(())
}

fn extra_verification_ratio(
    suite_config: &BenchmarkSuiteConfig,
    benchmark_config: &BenchmarkConfig,
) -> Result<()> {
    let res = FloxerParameterBenchmark::from_iter([0.02, 0.05, 0.1, 0.2, 0.3].into_iter().map(
        |extra_verification_ratio| FloxerConfig {
            algorithm_config: FloxerAlgorithmConfig {
                extra_verification_ratio,
                ..Default::default()
            },
            name: format!("extra_verification_ratio_{extra_verification_ratio}").replace('.', "_"),
            ..From::from(benchmark_config)
        },
    ))
    .name("extra_verification_ratio")
    .run(suite_config, benchmark_config)?;

    res.plot_alignment_stats(suite_config);

    Ok(())
}

fn index_build(
    suite_config: &BenchmarkSuiteConfig,
    benchmark_config: &BenchmarkConfig,
) -> Result<()> {
    let benchmark_config = benchmark_config.with_queries(Queries::Debug); // here we only care about the index building and skip the queries

    let name = "index_build";
    let folder = BenchmarkFolder::new(&suite_config.output_folder, name, &benchmark_config);
    let floxer_res = FloxerConfig {
        algorithm_config: FloxerAlgorithmConfig {
            index_strategy: IndexStrategy::AlwaysRebuild,
            ..Default::default()
        },
        name: String::from("floxer"),
        ..From::from(&benchmark_config)
    }
    .run(&folder, name, suite_config, ProfileConfig::Off)?;

    let minimap_res = MinimapConfig {
        index_strategy: IndexStrategy::AlwaysRebuild,
        ..From::from(&benchmark_config)
    }
    .run(&folder, suite_config)?;

    plots::plot_resource_metrics(
        name,
        [
            (&floxer_res.resource_metrics, "floxer"),
            (
                &minimap_res
                    .index_resource_metrics
                    .expect("minimap index resource metrics"),
                "minimap",
            ),
        ],
        &folder,
        suite_config,
    );

    Ok(())
}

fn interval_optimization(
    suite_config: &BenchmarkSuiteConfig,
    benchmark_config: &BenchmarkConfig,
) -> Result<()> {
    let benchmark_config = benchmark_config.with_smaller_queries();

    let res = FloxerParameterBenchmark::from_iter(IntervalOptimization::iter().map(
        |interval_optimization| FloxerConfig {
            algorithm_config: FloxerAlgorithmConfig {
                interval_optimization,
                ..Default::default()
            },
            name: interval_optimization.to_string(),
            ..From::from(&benchmark_config)
        },
    ))
    .name("interval_optimization")
    .run(suite_config, &benchmark_config)?;

    res.plot_alignment_stats(suite_config);

    Ok(())
}

fn max_anchors(
    suite_config: &BenchmarkSuiteConfig,
    benchmark_config: &BenchmarkConfig,
) -> Result<()> {
    let mut values = vec![20, 50, 100, 200, 1000];
    if benchmark_config.reference == Reference::Simulated {
        values.push(u64::MAX);
    }

    let res = FloxerParameterBenchmark::from_iter(values.into_iter().map(|max_num_anchors| {
        FloxerConfig {
            algorithm_config: FloxerAlgorithmConfig {
                max_num_anchors,
                ..Default::default()
            },
            name: format!("max_anchors_{max_num_anchors}"),
            ..From::from(benchmark_config)
        }
    }))
    .name("max_anchors")
    .run(suite_config, benchmark_config)?;

    res.plot_seed_stats(suite_config);
    res.plot_anchor_stats(suite_config);

    Ok(())
}

fn minimap(suite_config: &BenchmarkSuiteConfig, benchmark_config: &BenchmarkConfig) -> Result<()> {
    let name = "minimap";
    let folder = BenchmarkFolder::new(&suite_config.output_folder, name, benchmark_config);
    let floxer_res = FloxerConfig {
        name: String::from("floxer"),
        ..From::from(benchmark_config)
    }
    .run(&folder, name, suite_config, ProfileConfig::Off)?;

    let minimap_res = MinimapConfig::from(benchmark_config).run(&folder, suite_config)?;

    plots::plot_resource_metrics(
        name,
        [
            (&floxer_res.resource_metrics, "floxer"),
            (&minimap_res.map_resource_metrics, "minimap"),
        ],
        &folder,
        suite_config,
    );

    let mut floxer_mapped_reads_path = folder.most_recect_previous_run_folder();
    floxer_mapped_reads_path.push("floxer");
    floxer_mapped_reads_path.push("mapped_reads.bam");

    let mut minimap_mapped_reads_path = folder.most_recect_previous_run_folder();
    minimap_mapped_reads_path.push("minimap");
    minimap_mapped_reads_path.push("mapped_reads.sam");

    let aligner_comparison = analyze_alignments_detailed_comparison(
        &floxer_mapped_reads_path,
        &minimap_mapped_reads_path,
        floxer::DEFAULT_ERROR_RATE,
        &folder,
        suite_config,
    )?;

    plots::create_floxer_vs_minimap_plots(&aligner_comparison, &folder, suite_config);

    Ok(())
}

fn minimap_high_error_rate(
    suite_config: &BenchmarkSuiteConfig,
    benchmark_config: &BenchmarkConfig,
) -> Result<()> {
    let benchmark_name = "minimap_high_error_rate";
    let floxer_instance_name =
        format!("floxer_query_error_rate_{}", floxer::HIGH_ERROR_RATE).replace('.', "_");
    let folder = BenchmarkFolder::new(
        &suite_config.output_folder,
        benchmark_name,
        benchmark_config,
    );
    let floxer_res = FloxerConfig {
        name: floxer_instance_name.clone(),
        algorithm_config: FloxerAlgorithmConfig {
            query_errors: QueryErrors::Rate(floxer::HIGH_ERROR_RATE),
            ..Default::default()
        },
        ..From::from(benchmark_config)
    }
    .run(&folder, benchmark_name, suite_config, ProfileConfig::Off)?;

    let minimap_res = MinimapConfig::from(benchmark_config).run(&folder, suite_config)?;

    plots::plot_resource_metrics(
        benchmark_name,
        [
            (&floxer_res.resource_metrics, floxer_instance_name.as_str()),
            (&minimap_res.map_resource_metrics, "minimap"),
        ],
        &folder,
        suite_config,
    );

    let mut floxer_mapped_reads_path = folder.most_recect_previous_run_folder();
    floxer_mapped_reads_path.push(floxer_instance_name);
    floxer_mapped_reads_path.push("mapped_reads.bam");

    let mut minimap_mapped_reads_path = folder.most_recect_previous_run_folder();
    minimap_mapped_reads_path.push("minimap");
    minimap_mapped_reads_path.push("mapped_reads.sam");

    let aligner_comparison = analyze_alignments_detailed_comparison(
        &floxer_mapped_reads_path,
        &minimap_mapped_reads_path,
        floxer::HIGH_ERROR_RATE,
        &folder,
        suite_config,
    )?;

    plots::create_floxer_vs_minimap_plots(&aligner_comparison, &folder, suite_config);

    Ok(())
}

// some code duplication here for the pex seed, but I'll live with it for now.
fn pex_seed_errors(
    suite_config: &BenchmarkSuiteConfig,
    benchmark_config: &BenchmarkConfig,
) -> Result<()> {
    let res = FloxerParameterBenchmark::from_iter((0..4).map(|pex_seed_errors| FloxerConfig {
        algorithm_config: FloxerAlgorithmConfig {
            pex_seed_errors,
            ..Default::default()
        },
        name: format!("pex_seed_errors_{pex_seed_errors}"),
        ..From::from(benchmark_config)
    }))
    .name("pex_seed_errors")
    .run(suite_config, benchmark_config)?;

    res.plot_seed_stats(suite_config);
    res.plot_anchor_stats(suite_config);
    res.plot_alignment_stats(suite_config);

    Ok(())
}

fn pex_seed_errors_high_error_rate(
    suite_config: &BenchmarkSuiteConfig,
    benchmark_config: &BenchmarkConfig,
) -> Result<()> {
    // number of matched starts to significantly decline at 0.17 (0.16 lost exactly one query)
    let res = FloxerParameterBenchmark::from_iter((0..4).map(|pex_seed_errors| FloxerConfig {
        algorithm_config: FloxerAlgorithmConfig {
            query_errors: QueryErrors::Rate(0.17),
            pex_seed_errors,
            ..Default::default()
        },
        name: format!("pex_seed_errors_{pex_seed_errors}"),
        ..From::from(benchmark_config)
    }))
    .name("pex_seed_errors_high_error_rate")
    .run(suite_config, benchmark_config)?;

    res.plot_seed_stats(suite_config);
    res.plot_anchor_stats(suite_config);
    res.plot_alignment_stats(suite_config);

    Ok(())
}

fn pex_seed_errors_no_max_anchors(
    suite_config: &BenchmarkSuiteConfig,
    benchmark_config: &BenchmarkConfig,
) -> Result<()> {
    if benchmark_config.reference == Reference::HumanGenomeHg38 {
        bail!("no_max_anchors benchmark skipped for real human genome (repeats would cause ENORMOUS performance issues without max anchors)");
    }

    let res = FloxerParameterBenchmark::from_iter((0..4).map(|pex_seed_errors| FloxerConfig {
        algorithm_config: FloxerAlgorithmConfig {
            max_num_anchors: u64::MAX,
            pex_seed_errors,
            ..Default::default()
        },
        name: format!("pex_seed_errors_{pex_seed_errors}"),
        ..From::from(benchmark_config)
    }))
    .name("pex_seed_errors_no_max_anchors")
    .run(suite_config, benchmark_config)?;

    res.plot_seed_stats(suite_config);
    res.plot_anchor_stats(suite_config);
    res.plot_alignment_stats(suite_config);

    Ok(())
}

fn pex_seed_errors_no_max_anchors_and_high_error_rate(
    suite_config: &BenchmarkSuiteConfig,
    benchmark_config: &BenchmarkConfig,
) -> Result<()> {
    if benchmark_config.reference == Reference::HumanGenomeHg38 {
        bail!("no_max_anchors benchmark skipped for real human genome (repeats would cause ENORMOUS performance issues without max anchors)");
    }

    // 0 skipped, because it takes over 1 TB of space
    let res = FloxerParameterBenchmark::from_iter((1..4).map(|pex_seed_errors| FloxerConfig {
        algorithm_config: FloxerAlgorithmConfig {
            max_num_anchors: u64::MAX,
            query_errors: QueryErrors::Rate(0.17),
            pex_seed_errors,
            ..Default::default()
        },
        name: format!("pex_seed_errors_{pex_seed_errors}"),
        ..From::from(benchmark_config)
    }))
    .name("pex_seed_errors_no_max_anchors_and_high_error_rate")
    .run(suite_config, benchmark_config)?;

    res.plot_seed_stats(suite_config);
    res.plot_anchor_stats(suite_config);
    res.plot_alignment_stats(suite_config);

    Ok(())
}

fn pex_tree_building(
    suite_config: &BenchmarkSuiteConfig,
    benchmark_config: &BenchmarkConfig,
) -> Result<()> {
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
            name: format!("{}_{}_seed_errors", pex_tree_construction, pex_seed_errors),
            ..From::from(benchmark_config)
        }),
    )
    .name("pex_tree_building")
    .run(suite_config, benchmark_config)?;

    res.plot_seed_stats(suite_config);
    res.plot_anchor_stats(suite_config);

    Ok(())
}

fn profile(suite_config: &BenchmarkSuiteConfig, benchmark_config: &BenchmarkConfig) -> Result<()> {
    let _ = FloxerParameterBenchmark::from_iter([From::from(benchmark_config)])
        .name("profile")
        .with_profile()
        .run(suite_config, benchmark_config);

    Ok(())
}

fn problem_query(
    suite_config: &BenchmarkSuiteConfig,
    benchmark_config: &BenchmarkConfig,
) -> Result<()> {
    let benchmark_config = benchmark_config.with_queries(Queries::ProblemQuery);

    // do multiple times for non-deterministic bugs like race conditions
    let res = FloxerParameterBenchmark::from_iter((0..5).map(|i| FloxerConfig {
        name: format!("problem_{i}"),
        ..From::from(&benchmark_config)
    }))
    .name("problem_query")
    .with_profile()
    .run(suite_config, &benchmark_config)?;

    res.plot_seed_stats(suite_config);
    res.plot_anchor_stats(suite_config);
    res.plot_alignment_stats(suite_config);

    Ok(())
}

fn query_error_rate(
    suite_config: &BenchmarkSuiteConfig,
    benchmark_config: &BenchmarkConfig,
) -> Result<()> {
    let res =
        FloxerParameterBenchmark::from_iter([0.05, 0.07, 0.09, 0.11, 0.13, 0.15].into_iter().map(
            |query_error_ratio| FloxerConfig {
                algorithm_config: FloxerAlgorithmConfig {
                    query_errors: QueryErrors::Rate(query_error_ratio),
                    ..Default::default()
                },
                name: format!("query_error_rate_{query_error_ratio}").replace('.', "_"),
                ..From::from(benchmark_config)
            },
        ))
        .name("query_error_rate")
        .run(suite_config, benchmark_config)?;

    res.plot_seed_stats(suite_config);
    res.plot_anchor_stats(suite_config);

    Ok(())
}

fn threads(suite_config: &BenchmarkSuiteConfig, benchmark_config: &BenchmarkConfig) -> Result<()> {
    FloxerParameterBenchmark::from_iter([16, 20, 24, 28].into_iter().map(|num_threads| {
        FloxerConfig {
            algorithm_config: FloxerAlgorithmConfig {
                num_threads,
                ..Default::default()
            },
            name: format!("num_threads_{num_threads}"),
            ..From::from(benchmark_config)
        }
    }))
    .name("threads")
    .run(suite_config, benchmark_config)?;

    Ok(())
}

fn verification_algorithm(
    suite_config: &BenchmarkSuiteConfig,
    benchmark_config: &BenchmarkConfig,
) -> Result<()> {
    let benchmark_config = benchmark_config.with_smaller_queries();

    let res = FloxerParameterBenchmark::from_iter(VerificationAlgorithm::iter().map(
        |verification_algorithm| FloxerConfig {
            algorithm_config: FloxerAlgorithmConfig {
                verification_algorithm,
                ..Default::default()
            },
            name: verification_algorithm.to_string(),
            ..From::from(&benchmark_config)
        },
    ))
    .name("verification_algorithm")
    .run(suite_config, &benchmark_config)?;

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

    fn run(
        &self,
        suite_config: &BenchmarkSuiteConfig,
        benchmark_config: &BenchmarkConfig,
    ) -> Result<BenchmarkResult> {
        let benchmark_folder = BenchmarkFolder::new(
            &suite_config.output_folder,
            &self.benchmark_name,
            benchmark_config,
        );

        let mut floxer_results = Vec::new();
        let mut instance_names = Vec::new();

        for floxer_config in self.floxer_configs.iter() {
            let res = floxer_config.run(
                &benchmark_folder,
                &self.benchmark_name,
                suite_config,
                self.profile_config,
            )?;

            floxer_results.push(res);
            instance_names.push(floxer_config.name.clone());
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
