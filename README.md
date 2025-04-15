# Master's Thesis Benchmark

The code I use to evaluate the read mapper [floxer](https://github.com/feldroop/floxer) I developed for my master's thesis. This project is not intented for general use. Some of the code is an absolute mess, because when the thesis
deadline approached, things had to be done quickly. However, I still believe that this program can be used to
reproduce the results from my thesis without too much effort.

## Usage

The tool is implemented in Rust and a Rust toolchain is needed to compile it. (can be installed [here](https://rustup.rs/))

The tool expects a config file in TOML format that contains a number of paths that are used in the analysis.
The following keys are required:

```toml
output_folder = "<path>"
compare_aligner_outputs_binary = "<path>" # code for this binary is in floxer repository
simulated_dataset_binary = "<path>" # code for this binary is in floxer repository

[readmapper_binaries]
floxer = "<path>"
minimap = "<path>"

[reference_paths]
human_genome_hg38 = "<path>"
masked_human_genome_hg38 = "<path>"
debug = "<path>"
simulated = "<path>"

[query_paths]
human_wgs_nanopore = "<path>"
human_wgs_nanopore_small = "<path>"
debug = "<path>"
problem_query = "<path>"
simulated = "<path>"
simulated_small = "<path>"
```

Then, the program can be run using the command

```sh
cargo run --release -- <floxer-options...>
```

The help page gives detailed instructions on how to choose and customize benchmarks to be run.

```
Usage: msc-thesis-benchmark [OPTIONS] [BENCHMARKS]...

Arguments:
  [BENCHMARKS]...  Give benchmark names that should be run. If none are given, all will be run [possible values: anchor-group-order-and-choice-strategy, anchors-per-verification-task,
  debug, default-params, extra-verification-ratio, index-build, interval-optimization, max-anchors, minimap, minimap-high-error-rate, pex-seed-errors, pex-seed-errors-high-error-rate,
  pex-seed-errors-no-max-anchors, pex-seed-errors-no-max-anchors-and-high-error-rate, pex-tree-building, problem-query, profile, query-error-rate, seed-sampling-step-size, threads,
  verification-algorithm]

Options:
  -c, --config-file <CONFIG_FILE>    [default: benchmark_config.toml]
  -o, --only-analysis                If given, only the analysis is rerun on the results of the most recent run of floxer (if there was one)
  -t, --tag <TAG>                    If given, this tag is appended to the folder name of all benchmarks
  -r, --reference <REFERENCE>        [default: human-genome-hg38] [possible values: human-genome-hg38, masked-human-genome-hg38, debug, simulated]
  -q, --queries <QUERIES>            [default: human-wgs-nanopore] [possible values: human-wgs-nanopore, human-wgs-nanopore-small, debug, problem-query, simulated, simulated-small]
  -c, --cigar-output <CIGAR_OUTPUT>  [default: off] [possible values: on, off]
  -h, --help                         Print help
```
