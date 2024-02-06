//! Module for handling splitting operations and related utilities.
//!
//! This module provides functionality for splitting reads, particularly focusing on operations
//! related to unblocked read IDs, quality score calculations, and output file formats.
//!
//! # Enums
//!
//! - [`SplitType`](enum.SplitType.html): Enum representing the type of reads output after splitting.
//!   - `All`: Output all reads.
//!   - `UnblockedOnly`: Output unblocked reads only.
//!   - `SequencedOnly`: Output sequenced reads only.
//!
//! - [`EmitType`](enum.EmitType.html): Enum representing the type of file to write for split records.
//!   - `Bam`: Emit BAM files as output.
//!   - `Fastq`: Emit FASTQ files as output.
//!   - `Fasta`: Emit FASTA files as output.
//!
//! - [`Wrapper`](enum.Wrapper.html): Enum representing different output wrappers for split files.
//!   - `Bam`: Wrapper for BAM output.
//!   - `Fastx`: Wrapper for generic FASTX output.
//!
//! # Functions
//!
//! - [`read_unblocked_read_ids`](fn.read_unblocked_read_ids.html): Read unblocked read IDs from a file into a HashSet.
//!   - Arguments:
//!     - `path`: The path to the unblocked_read_ids.txt file.
//!   - Returns:
//!     - `Result`: A `Result` containing a `FnvHashSet<String>` or an `io::Error`.
//!
//! - [`_ave_qual`](fn._ave_qual.html): Calculate the average quality of a read.
//!   - Arguments:
//!     - `quals`: A slice of Phred scores, already normalized (i.e., base 33 ASCII bytes -33).
//!   - Returns:
//!     - `f64`: Average quality of the read.
//!
use clap::ValueEnum;
use fnv::FnvHashSet;
use noodles::bam;
use noodles_bgzf as bgzf;

use std::{
    fs::File,
    io,
    io::{BufRead, BufReader, BufWriter},
    path::PathBuf,
};

/// Enum representing the type of reads output after splitting, (BAM file splitting only ATM).
#[derive(Debug, ValueEnum, Clone, Default, PartialEq)]
pub enum SplitType {
    /// Output all reads.
    All,
    /// Output unblocked reads only.
    UnblockedOnly,
    /// Output sequenced reads only.
    #[default]
    SequencedOnly,
}

/// Enum representing the type of file to write for split records. (BAM file splitting only ATM)
#[derive(Debug, ValueEnum, Clone, Default, PartialEq)]
pub enum EmitType {
    /// Emit BAM files as output.
    #[default]
    Bam,
    /// Emit FASTQ files as output.
    Fastq,
    /// Emit FASTA files as output.
    Fasta,
}

/// Enum representing different file writers for BAM file output.
pub enum Wrapper {
    /// Wrapper for BAM output.
    Bam(bam::io::Writer<bgzf::Writer<File>>),
    /// Wrapper for generic FASTX output.
    Fastx(BufWriter<File>),
}

/// Read unblocked_read_ids.txt into a HashSet.
///
/// # Arguments
///
/// * `path` - The path to the unblocked_read_ids.txt file.
///
/// # Returns
///
/// Returns a `Result` containing a `FnvHashSet<String>` or an `io::Error`.
///
/// /// # Example
///
/// ```rust,ignore
///    let unblocked_read_ids = PathBuf::from("tests/test_unblocked.txt");
///    match read_unblocked_read_ids(unblocked_read_ids) {
///        Ok(unb_set) => {
///            assert_eq!(unb_set.len(), 2);
///        }
///        Err(_) => {
///            println!("uhoh");
///        }
///    }
/// ````
pub fn read_unblocked_read_ids(path: PathBuf) -> Result<FnvHashSet<String>, io::Error> {
    // let file = File::open(&path);
    match File::open(path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            let rejected_reads: FnvHashSet<String> = reader
                .lines()
                .map(|l| l.expect("Couldn't read line"))
                .collect();
            Ok(rejected_reads)
        }
        Err(err) => Err(err),
    }
}

/// This function calculates the average quality of a read, and does this correctly
/// First the Phred scores are converted to probabilities (10^(q)/-10) and summed
/// and then divided by the number of bases/scores and converted to Phred again -10*log10(average)
/// Taken from https://github.com/wdecoster/chopper/blob/b94a60ff075f3b69aea4ef9cb976c8064ceb5993/src/main.rs#L176
///
///
///
/// # Arguments
///
/// * `quals` - A slice of Phred scores, already normalised (i.e base 33 Ascii bytes -33).
///
/// # Returns
///
/// Returns average quality of the read a `f64`.
///
/// # Example
///
/// ```rust,ignore
/// use crate::_splitting::_ave_qual;
///
/// let quals = vec![30, 20, 25, 35];
/// let average_quality = _ave_qual(&quals);
/// println!("Average Quality: {}", average_quality);
/// ```
pub fn _ave_qual(quals: &[u8]) -> f64 {
    let probability_sum = quals
        .iter()
        .map(|q| 10_f64.powf(((*q) as f64) / -10.0))
        .sum::<f64>();
    (probability_sum / quals.len() as f64).log10() * -10.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_read_unblocked_read_ids() {
        let unblocked_read_ids = PathBuf::from("tests/test_unblocked.txt");
        match read_unblocked_read_ids(unblocked_read_ids) {
            Ok(unb_set) => {
                assert_eq!(unb_set.len(), 2);
            }
            Err(_) => {
                println!("uhoh");
            }
        }
    }

    #[test]
    fn test_empty_read_unblocked_read_ids() {
        let unblocked_read_ids = PathBuf::from("tests/test_empty_unb_ids.txt");
        match read_unblocked_read_ids(unblocked_read_ids) {
            Ok(unb_set) => {
                assert_eq!(unb_set.len(), 0);
            }
            Err(_) => {
                println!("uhoh");
            }
        }
    }
    #[test]
    fn test_ave_qual_empty() {
        // Test when the input slice is empty
        let quals = Vec::new();
        let result = _ave_qual(&quals);

        // Assert that the result is NaN for an empty slice
        assert!(result.is_nan());
    }

    #[test]
    fn test_ave_qual_single() {
        // Test when the input slice has a single Phred score
        let quals = vec![30];
        let result = _ave_qual(&quals);

        // Assert that the result is equal to the single Phred score
        assert_eq!(result, 30.0);
    }
    #[test]
    fn test_ave_qual_multiple() {
        // Test when the input slice has multiple Phred scores
        let quals = vec![30, 20, 25, 35];
        let result = _ave_qual(&quals) as usize;
        assert_eq!(result, 24)
    }
}
