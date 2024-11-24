use core::str;
use std::{fs, path::Path, process::Command};

use anyhow::{bail, Context, Result};
use rust_htslib::bam::{self, record::Aux, Read};
use serde::Deserialize;

use crate::{config::BenchmarkSuiteConfig, folder_structure::BenchmarkFolder};

pub fn analyze_alignments_simple<P: AsRef<Path>>(
    mapped_reads_path: P,
) -> Result<SimpleMappedReadsStats> {
    let mut bam = bam::Reader::from_path(mapped_reads_path.as_ref())?;

    let mut primary_alignment_edit_distances = Vec::new();

    for record in bam.records() {
        let record = record?;

        if record.is_supplementary() {
            bail!("unexpected supplementary bam record in floxer output")
        }

        if record.is_unmapped() {
            continue;
        }

        if !record.is_secondary() {
            let edit_distance_record = record.aux(b"NM")?;
            // no idea why the htslib sometimes returns different number types...
            let edit_distance = match edit_distance_record {
                Aux::I32(value) => value,
                Aux::I8(value) => value as i32,
                Aux::U8(value) => value as i32,
                Aux::I16(value) => value as i32,
                Aux::U16(value) => value as i32,
                Aux::U32(value) => value as i32,
                _ => bail!("wrong edit distance tag type: {:?}", edit_distance_record),
            };

            primary_alignment_edit_distances.push(edit_distance);
        }
    }

    Ok(SimpleMappedReadsStats {
        num_mapped: primary_alignment_edit_distances.len() as i32,
        primary_alignment_edit_distances,
    })
}

#[derive(Debug, Clone)]
pub struct SimpleMappedReadsStats {
    pub num_mapped: i32,
    pub primary_alignment_edit_distances: Vec<i32>,
}

// runs a C++ program to do this comparison that I wrote earlier
pub fn analyze_alignments_detailed_comparison(
    mapped_reads_path_floxer: impl AsRef<Path>,
    mapped_reads_path_minimap: impl AsRef<Path>,
    floxer_query_error_rate: f64,
    benchmark_folder: &BenchmarkFolder,
    suite_config: &BenchmarkSuiteConfig,
) -> Result<DetailedMappedReadsComparison> {
    let output = Command::new(&suite_config.compare_aligner_outputs_binary)
        .arg("--new")
        .arg(mapped_reads_path_floxer.as_ref())
        .arg("--reference")
        .arg(mapped_reads_path_minimap.as_ref())
        .arg("--error-rate")
        .arg(floxer_query_error_rate.to_string())
        .output()?;

    if !output.status.success() {
        bail!(
            "compare_aligner_outputs failed with the following stderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let result_str = str::from_utf8(&output.stdout)?;
    let mut result_file_path = benchmark_folder.get().to_owned();
    result_file_path.push("detailed_aligner_comparison.toml");

    fs::write(result_file_path, result_str)?;

    toml::from_str(result_str).context("compare_aligner_outputs stdout deserialize")
}

#[derive(Debug, Deserialize)]
pub struct DetailedMappedReadsComparison {
    pub general_stats: FullStats,
    pub floxer_stats_if_floxer_mapped: ScopedStats,
    pub minimap_stats_if_minimap_mapped: ScopedStats,
    pub minimap_stats_if_both_mapped: ScopedStats,
    pub minimap_stats_if_only_minimap_mapped: ScopedStats,
}

#[derive(Debug, Deserialize)]
pub struct FullStats {
    pub number_of_queries: usize,
    pub both_mapped: usize,
    pub both_unmapped: usize,
    pub floxer_mapped: usize,
    pub floxer_unmapped: usize,
    pub minimap_mapped: usize,
    pub minimap_unmapped: usize,
    pub floxer_unmapped_and_minimap_mapped: usize,
    pub minimap_unmapped_and_floxer_mapped: usize,
}

#[derive(Debug, Deserialize)]
pub struct ScopedStats {
    pub num_queries: usize,
    pub primary_chimeric: usize,
    pub primary_linear_basic: usize,
    pub primary_linear_clipped: usize,
    pub primary_high_edit_distance: usize,
    pub primary_inversion: usize,
    pub multiple_mapping: usize,
    pub primary_not_basic_secondary_basic: usize,
    pub average_longest_indel: f64,
    pub average_error_rate_of_primary_basic_alignments: f64,
}

// ----- for simulate dataset -----

pub fn verify_simulated_dataset(
    mapped_reads_path: &Path,
    suite_config: &BenchmarkSuiteConfig,
) -> Result<SimulatedDatasetVerificationSummary> {
    let output = Command::new(&suite_config.simulated_dataset_binary)
        .arg("verify")
        .arg("--alignments")
        .arg(mapped_reads_path)
        .output()?;

    if !output.status.success() {
        bail!(
            "Failed to verify mapped reads of simulated data set with stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let str_data = String::from_utf8(output.stdout)?;
    let data: VerifiedSimulatedDataset = toml::from_str(&str_data)?;

    let mut summary = SimulatedDatasetVerificationSummary {
        num_optimal_mapped: 0,
        suboptimal_mapped_queries: Vec::new(),
        unmapped_queries: Vec::new(),
    };

    for query in data.queries {
        match query.status {
            MappingStatus::NotFound => summary.unmapped_queries.push(query),
            MappingStatus::FoundOptimal => summary.num_optimal_mapped += 1,
            MappingStatus::FoundSuboptimal { .. } => summary.suboptimal_mapped_queries.push(query),
        }
    }

    Ok(summary) // TODO integrate this somewhere to happen automatically
}

pub struct SimulatedDatasetVerificationSummary {
    num_optimal_mapped: usize,
    suboptimal_mapped_queries: Vec<VerifiedSimulatedQuery>,
    unmapped_queries: Vec<VerifiedSimulatedQuery>,
}

impl SimulatedDatasetVerificationSummary {
    pub fn print_if_missed(&self) {
        for query in self
            .unmapped_queries
            .iter()
            .chain(&self.suboptimal_mapped_queries)
        {
            println!("Query {}: {:?}", query.id, query.status);
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct VerifiedSimulatedDataset {
    queries: Vec<VerifiedSimulatedQuery>,
}

#[derive(Debug, Deserialize)]
pub struct VerifiedSimulatedQuery {
    id: String,
    status: MappingStatus,
}

#[derive(Debug, Deserialize)]
pub enum MappingStatus {
    NotFound,
    FoundOptimal,
    #[allow(unused)]
    FoundSuboptimal {
        pos_diff_expected_num_errors: usize,
        pos_diff_higher_num_errors: usize,
    },
}
