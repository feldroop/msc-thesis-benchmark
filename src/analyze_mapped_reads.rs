use std::{collections::HashMap, path::Path};

use anyhow::{bail, Result};
use rust_htslib::bam::{self, record::Aux, Read};

pub fn analyze_alignments<P: AsRef<Path>>(mapped_reads_path: P) -> Result<MappedReadsStats> {
    let mut bam = bam::Reader::from_path(mapped_reads_path.as_ref())?;

    let mut num_unmapped = 0;
    let mut primary_alignment_edit_distances = Vec::new();
    let mut num_alignments_per_aligned_query = HashMap::new();

    for record in bam.records() {
        let record = record?;

        if record.is_supplementary() {
            bail!("unexpected supplementary bam record in floxer output")
        }

        if record.is_unmapped() {
            num_unmapped += 1;
            continue;
        }

        *num_alignments_per_aligned_query
            .entry(record.qname().to_vec())
            .or_insert(0i32) += 1;

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

    Ok(MappedReadsStats {
        num_mapped: primary_alignment_edit_distances.len() as i32,
        num_unmapped,
        primary_alignment_edit_distances,
        num_alignments_of_aligned_queries: num_alignments_per_aligned_query
            .values()
            .copied()
            .collect(),
    })
}

#[derive(Debug, Clone)]
pub struct MappedReadsStats {
    pub num_mapped: i32,
    pub num_unmapped: i32,
    pub primary_alignment_edit_distances: Vec<i32>,
    pub num_alignments_of_aligned_queries: Vec<i32>,
}
