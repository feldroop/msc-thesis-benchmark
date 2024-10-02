use crate::config::BenchmarkSuiteConfig;
use crate::floxer::{
    AnchorGroupOrder, FloxerAlgorithmConfig, FloxerConfig, FloxerResult, IntervalOptimization,
    PexTreeConstruction, QueryErrors, VerificationAlgorithm,
};
use crate::floxer::{Queries, Reference};
use crate::folder_structure::BenchmarkFolder;
use crate::plots;

use anyhow::{bail, Result};
use clap::ValueEnum;
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, Copy, EnumIter, ValueEnum)]
pub enum Benchmark {
    AnchorGroupOrder,
    Debug,
    IntervalOptimization,
    PexSeedErrors,
    PexTreeBuilding,
    QueryErrorRate,
    VerificationAlgorithm,
}

impl Benchmark {
    pub fn run(&self, suite_config: &BenchmarkSuiteConfig) -> Result<()> {
        match *self {
            Benchmark::AnchorGroupOrder => anchor_group_order(suite_config),
            Benchmark::Debug => debug_benchmark(suite_config),
            Benchmark::IntervalOptimization => interval_optimization(suite_config),
            Benchmark::PexSeedErrors => pex_seed_errors(suite_config),
            Benchmark::PexTreeBuilding => pex_tree_building(suite_config),
            Benchmark::QueryErrorRate => query_error_rate(suite_config),
            Benchmark::VerificationAlgorithm => verification_algorithm(suite_config),
        }
    }
}

pub fn run_benchmarks<I: IntoIterator<Item = Benchmark>>(
    benchmarks: I,
    suite_config: &BenchmarkSuiteConfig,
) -> Result<()> {
    let mut num_error_runs = 0;
    for benchmark in benchmarks.into_iter() {
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
    floxer_results: Vec<FloxerResult>,
}

fn anchor_group_order(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    FloxerParameterBenchmark {
        floxer_configs_with_names: AnchorGroupOrder::iter().map(|anchor_group_order| {
            (
                FloxerConfig {
                    algorithm_config: FloxerAlgorithmConfig {
                        anchor_group_order,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                anchor_group_order.to_string(),
            )
        }),
        benchmark_name: "anchor_group_order",
    }
    .run(suite_config)
    .map(|_| ())
}

fn debug_benchmark(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    FloxerParameterBenchmark {
        floxer_configs_with_names: PexTreeConstruction::iter().map(|pex_tree_construction| {
            (
                FloxerConfig {
                    reference: Reference::HumanGenomeHg38,
                    queries: Queries::HumanWgsNanopore,
                    algorithm_config: FloxerAlgorithmConfig {
                        pex_tree_construction,
                        extra_verification_ratio: 2.0,
                        num_threads: 1,
                        pex_seed_errors: 1,
                        query_errors: QueryErrors::Exact(2),
                        ..Default::default()
                    },
                },
                pex_tree_construction.to_string(),
            )
        }),
        benchmark_name: "debug_benchmark",
    }
    .run(suite_config)
    .map(|_| ())
}

fn interval_optimization(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    FloxerParameterBenchmark {
        floxer_configs_with_names: IntervalOptimization::iter().map(|interval_optimization| {
            (
                FloxerConfig {
                    algorithm_config: FloxerAlgorithmConfig {
                        interval_optimization,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                interval_optimization.to_string(),
            )
        }),
        benchmark_name: "interval_optimization",
    }
    .run(suite_config)
    .map(|_| ())
}

fn pex_seed_errors(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    FloxerParameterBenchmark {
        floxer_configs_with_names: (0..4).map(|pex_seed_errors| {
            (
                FloxerConfig {
                    algorithm_config: FloxerAlgorithmConfig {
                        pex_seed_errors,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                format!("pex_seed_errors_{pex_seed_errors}"),
            )
        }),
        benchmark_name: "pex_seed_errors",
    }
    .run(suite_config)
    .map(|_| ())
}

fn pex_tree_building(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    FloxerParameterBenchmark {
        floxer_configs_with_names: PexTreeConstruction::iter().map(|pex_tree_construction| {
            (
                FloxerConfig {
                    reference: Reference::HumanGenomeHg38,
                    queries: Queries::HumanWgsNanopore,
                    algorithm_config: FloxerAlgorithmConfig {
                        pex_tree_construction,
                        ..Default::default()
                    },
                },
                pex_tree_construction.to_string(),
            )
        }),
        benchmark_name: "pex_tree_building",
    }
    .run(suite_config)
    .map(|_| ())
}

fn query_error_rate(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    FloxerParameterBenchmark {
        floxer_configs_with_names: [0.03, 0.05, 0.07, 0.09]
            .into_iter()
            .map(|query_error_ratio| {
                (
                    FloxerConfig {
                        reference: Reference::HumanGenomeHg38,
                        queries: Queries::HumanWgsNanopore,
                        algorithm_config: FloxerAlgorithmConfig {
                            query_errors: QueryErrors::Rate(query_error_ratio),
                            ..Default::default()
                        },
                    },
                    format!("query_error_rate_{query_error_ratio}"),
                )
            }),
        benchmark_name: "query_error_rate",
    }
    .run(suite_config)
    .map(|_| ())
}

fn verification_algorithm(suite_config: &BenchmarkSuiteConfig) -> Result<()> {
    FloxerParameterBenchmark {
        floxer_configs_with_names: VerificationAlgorithm::iter().map(|verification_algorithm| {
            (
                FloxerConfig {
                    reference: Reference::HumanGenomeHg38,
                    queries: Queries::HumanWgsNanopore,
                    algorithm_config: FloxerAlgorithmConfig {
                        verification_algorithm,
                        ..Default::default()
                    },
                },
                verification_algorithm.to_string(),
            )
        }),
        benchmark_name: "verification_algorithm",
    }
    .run(suite_config)
    .map(|_| ())
}

struct FloxerParameterBenchmark<I> {
    floxer_configs_with_names: I,
    benchmark_name: &'static str,
}

impl<I> FloxerParameterBenchmark<I>
where
    I: IntoIterator<Item = (FloxerConfig, String)>,
{
    fn run(self, suite_config: &BenchmarkSuiteConfig) -> Result<BenchmarkResult> {
        let benchmark_folder =
            BenchmarkFolder::new(&suite_config.output_folder, self.benchmark_name);

        let mut floxer_results = Vec::new();

        for (floxer_config, instance_name) in self.floxer_configs_with_names {
            let res = floxer_config.run(
                &benchmark_folder,
                self.benchmark_name,
                Some(&instance_name),
                suite_config,
            )?;

            floxer_results.push(res);
        }

        plots::plot_general_floxer_info(
            self.benchmark_name,
            &floxer_results,
            &benchmark_folder,
            &suite_config.output_folder,
        );
        plots::plot_resource_metrics(
            self.benchmark_name,
            floxer_results
                .iter()
                .map(|res| (&res.resource_metrics, res.benchmark_instance_name.as_str())),
            &benchmark_folder,
            &suite_config.output_folder,
        );

        Ok(BenchmarkResult {
            benchmark_name: self.benchmark_name.to_owned(),
            folder: benchmark_folder,
            floxer_results,
        })
    }
}
