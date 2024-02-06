//! Module for splitting BAM files into sequenced and unblocked records.
//!
//! This module provides functionality to split BAM files into two categories: sequenced and unblocked records.
//! It includes the `split_bam` function that takes a BAM file, a list of unblocked read IDs, and other parameters,
//! and writes the sequenced and unblocked records to separate output files in either BAM, FASTA, or FASTQ format.
//!
//! # Function
//!
//! - [`split_bam`](fn.split_bam.html): Split a BAM file into sequenced and unblocked records.
//!
//! # Example
//!
//! ```rust
//! # use crate::_splitting::{EmitType, SplitType};
//! # use std::path::PathBuf;
//!
//! let bam_file = PathBuf::from("path/to/input.bam");
//! let unblocked_read_ids = PathBuf::from("path/to/unblocked_read_ids.txt");
//! let prefix = String::from("output_prefix");
//! let split_type = SplitType::All;
//! let qual_thresh = Some(30);
//! let emit_type = EmitType::Bam;
//!
//! match split_bam(bam_file, unblocked_read_ids, prefix, split_type, qual_thresh, emit_type) {
//!     Ok(_) => println!("BAM file successfully split into sequenced and unblocked records."),
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! ```
//!
//! # Filters
//!
//! The splitting process includes optional filters based on quality scores and flags:
//!
//! - Quality Threshold: Sequences with an average quality score below the specified threshold are filtered out.
//! - Flags: Supplementary and secondary alignments are excluded from the output.
//!
//! # Output
//!
//! The output files are generated based on the specified parameters:
//!
//! - Sequenced Records: Output files containing only sequenced records.
//! - Unblocked Records: Output files containing only unblocked records.
//!
//! The output files can be in BAM, FASTA, or FASTQ format, depending on the chosen `EmitType`.
//!
use crate::_splitting::{
    _ave_qual, read_unblocked_read_ids, CompressionType, EmitType, SplitType, Wrapper,
};
use flate2::{write::GzEncoder, Compression};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use noodles::{
    bam::{self, Record},
    sam::alignment::record::QualityScores,
};
use noodles_bgzf as bgzf;
use std::{
    fs::File,
    io::{BufWriter, Error, Write},
    num::NonZeroUsize,
    path::PathBuf,
    thread,
};
/// DO the Newline SLice
const NEWLINE_SLICE: &[u8] = &[10];
/// Minimum value for a Phred quality score WHAT A DUMB SYSTEm
const MIN_VALUE: u8 = b'!';

/// Write a Record to a Fasta file
fn write_fasta_record<W: Write>(
    record: &Record,
    mut writer: W,
    seq: &mut Vec<u8>,
    read_id: &[u8],
) -> Result<(), Error> {
    *seq = record.sequence().iter().map(u8::from).collect();
    writer.write_all(b">")?;
    writer.write_all(read_id)?;
    writer.write_all(NEWLINE_SLICE)?;
    writer.write_all(seq)?;
    writer.write_all(NEWLINE_SLICE)?;
    Ok(())
}

/// write a record to a Fastq file
fn write_fastq_record<W: Write>(
    record: &Record,
    mut writer: W,
    seq: &mut Vec<u8>,
    qual: &mut Vec<u8>,
    read_id: &[u8],
) -> Result<(), Error> {
    *seq = record.sequence().iter().map(u8::from).collect();
    *qual = record
        .quality_scores()
        .iter()
        .map(|x| x + MIN_VALUE)
        .collect();
    writer.write_all(b"@")?;
    writer.write_all(read_id)?;
    writer.write_all(NEWLINE_SLICE)?;
    writer.write_all(seq)?;
    writer.write_all(NEWLINE_SLICE)?;
    writer.write_all(b"+")?;
    writer.write_all(NEWLINE_SLICE)?;
    writer.write_all(qual)?;
    writer.write_all(NEWLINE_SLICE)?;
    Ok(())
}

/// Write a record to a Fasta file

/// Split a BAM file into sequenced and unblocked records.
///
/// This function takes a BAM file, a list of unblocked read IDs, and other parameters to split the input
/// BAM file into two categories: sequenced and unblocked records. The output is written to separate files
/// in either BAM, FASTA, or FASTQ format, depending on the specified `EmitType`.
///
/// # Arguments
///
/// * `bam_file` - The path to the input BAM file.
/// * `unblocked_read_ids` - The path to the file containing unblocked read IDs.
/// * `prefix` - The output file prefix. If empty, default filenames will be used.
/// * `split_type` - The type of reads to output: `All`, `SequencedOnly`, or `UnblockedOnly`.
/// * `qual_thresh` - Optional quality threshold. If set, sequences below this average quality will be filtered out.
/// * `length_thresh` - Optional minimum read length. If set, reads shorter than this will be filtered out.
/// * `emit_type` - The type of file to write for split records: `Bam`, `Fastq`, or `Fasta`.
/// * `compression` - The type of compression to use for FASTX output: `Gzipped`, `Uncompressed`
///
/// # Returns
///
/// Returns `Result<(), Error>` where `Error` is an IO error if any occurs during file operations.
///
/// # Examples
///
/// Split a BAM file into sequenced and unblocked records, writing output to default filenames:
///
/// ```rust,ignore
/// use crate::_splitting::{EmitType, SplitType};
/// use std::path::PathBuf;
///
/// let bam_file = PathBuf::from("path/to/input.bam");
/// let unblocked_read_ids = PathBuf::from("path/to/unblocked_read_ids.txt");
/// let prefix = String::new(); // Empty prefix for default filenames
/// let split_type = SplitType::All;
/// let qual_thresh = None; // No quality threshold
/// let emit_type = EmitType::Bam;
///
/// match split_bam(bam_file, unblocked_read_ids, prefix, split_type, qual_thresh, emit_type) {
///     Ok(_) => println!("BAM file successfully split into sequenced and unblocked records."),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
///
/// Split a BAM file into sequenced records only, applying a quality threshold and writing to custom filenames:
///
/// ```rust,ignore
/// use crate::_splitting::{EmitType, SplitType};
/// use std::path::PathBuf;
///
/// let bam_file = PathBuf::from("path/to/input.bam");
/// let unblocked_read_ids = PathBuf::from("path/to/unblocked_read_ids.txt");
/// let prefix = String::from("custom_output");
/// let split_type = SplitType::SequencedOnly;
/// let qual_thresh = Some(30); // Quality threshold set to 30
/// let emit_type = EmitType::Fastq; // Output as FASTQ
///
/// match split_bam(bam_file, unblocked_read_ids, prefix, split_type, qual_thresh, emit_type) {
///     Ok(_) => println!("BAM file successfully split into sequenced records with quality filtering."),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ````
#[allow(clippy::too_many_arguments)]
pub fn split_bam(
    bam_file: PathBuf,
    unblocked_read_ids: PathBuf,
    prefix: String,
    split_type: SplitType,
    qual_thresh: Option<usize>,
    length_thresh: Option<usize>,
    emit_type: EmitType,
    compression: CompressionType,
) -> Result<(), Error> {
    assert!(bam_file.exists());
    let file = File::open(bam_file)?;

    let worker_count = thread::available_parallelism().unwrap_or(NonZeroUsize::MIN);
    let decoder = bgzf::MultithreadedReader::with_worker_count(worker_count, file);

    let mut bam_reader = bam::io::Reader::from(decoder);
    let _header = bam_reader.read_header()?;
    let unblocked_read_ids = read_unblocked_read_ids(unblocked_read_ids).unwrap();
    let suffix = match compression {
        CompressionType::Gzipped => ".gz",
        _ => "",
    };
    // Choose output file names
    let (seq_fn, unb_fn) = if prefix.is_empty() {
        match emit_type {
            EmitType::Bam => (String::from("sequenced.bam"), String::from("unblocked.bam")),
            EmitType::Fasta => (
                String::from("sequenced.fasta"),
                String::from("unblocked.fasta"),
            ),
            EmitType::Fastq => (
                String::from("sequenced.fastq"),
                String::from("unblocked.fastq"),
            ),
        }
    } else {
        match emit_type {
            EmitType::Bam => (
                format!("{}.sequenced.bam", prefix),
                format!("{}.unblocked.bam", prefix),
            ),
            EmitType::Fasta => (
                format!("{}.sequenced.fasta", prefix),
                format!("{}.unblocked.fasta", prefix),
            ),
            EmitType::Fastq => (
                format!("{}.sequenced.fastq", prefix),
                format!("{}.unblocked.fastq", prefix),
            ),
        }
    };
    let (seq_fn, unb_fn) = (
        format!("{}{}", seq_fn, suffix),
        format!("{}{}", unb_fn, suffix),
    );
    // Create outfile writers> first choose sequenecd, unblocked, both then emit type -> Fastx or BAM
    let (mut sequenced_reads_writer, mut unblocked_reads_writer) = match split_type {
        SplitType::All => {
            // What are we emitting
            match emit_type {
                EmitType::Bam => {
                    let mut sbam: bam::io::Writer<bgzf::Writer<File>> =
                        bam::io::Writer::new(File::create(seq_fn).expect("Failed to create file"));
                    let mut ubam =
                        bam::io::Writer::new(File::create(unb_fn).expect("Failed to create file"));
                    sbam.write_header(&_header).unwrap();
                    ubam.write_header(&_header).unwrap();
                    (Some(Wrapper::Bam(sbam)), Some(Wrapper::Bam(ubam)))
                }
                _ => {
                    let mut _sequenced_reads = match File::create(&seq_fn) {
                        Ok(file) => BufWriter::new(file),
                        Err(err) => {
                            eprintln!("Could not create output file: {}\n{}", &seq_fn, err);
                            std::process::exit(1)
                        }
                    };
                    let unblocked_reads = match File::create(&unb_fn) {
                        Ok(file) => BufWriter::new(file),
                        Err(err) => {
                            eprintln!("Could not create output file: {}\n{}", &unb_fn, err);
                            std::process::exit(1)
                        }
                    };
                    match compression {
                        CompressionType::Uncompressed => (
                            Some(Wrapper::Fastx(_sequenced_reads)),
                            Some(Wrapper::Fastx(unblocked_reads)),
                        ),
                        CompressionType::Gzipped => {
                            let gz_seq = GzEncoder::new(_sequenced_reads, Compression::default());
                            let gz_unb: GzEncoder<BufWriter<File>> =
                                GzEncoder::new(unblocked_reads, Compression::default());
                            (
                                Some(Wrapper::GzFastx(gz_seq)),
                                Some(Wrapper::GzFastx(gz_unb)),
                            )
                        }
                        _ => {
                            unimplemented!()
                        }
                    }
                }
            }
        }
        SplitType::SequencedOnly => match emit_type {
            EmitType::Bam => {
                let mut sbam: bam::io::Writer<bgzf::Writer<File>> =
                    bam::io::Writer::new(File::create(seq_fn).expect("Failed to create file"));
                sbam.write_header(&_header).unwrap();
                (Some(Wrapper::Bam(sbam)), None)
            }
            _ => {
                let mut _sequenced_reads = match File::create(&seq_fn) {
                    Ok(file) => BufWriter::new(file),
                    Err(err) => {
                        eprintln!("Could not create output file: {}\n{}", &seq_fn, err);
                        std::process::exit(1)
                    }
                };
                (Some(Wrapper::Fastx(_sequenced_reads)), None)
            }
        },
        SplitType::UnblockedOnly => match emit_type {
            EmitType::Bam => {
                let mut ubam: bam::io::Writer<bgzf::Writer<File>> =
                    bam::io::Writer::new(File::create(unb_fn).expect("Failed to create file"));
                ubam.write_header(&_header).unwrap();
                (None, Some(Wrapper::Bam(ubam)))
            }
            _ => {
                let unblocked_reads = match File::create(&unb_fn) {
                    Ok(file) => BufWriter::new(file),
                    Err(err) => {
                        eprintln!("Could not create output file: {}\n{}", &seq_fn, err);
                        std::process::exit(1)
                    }
                };
                (None, Some(Wrapper::Fastx(unblocked_reads)))
            }
        },
    };
    let mut record = noodles::bam::Record::default();
    let write_unblock = split_type != SplitType::SequencedOnly;
    let mut seq: Vec<u8> = vec![];
    let mut qual: Vec<u8> = vec![];

    // Setup progress bar
    let bar = ProgressBar::with_draw_target(None, ProgressDrawTarget::stdout())
        .with_message("BAM Records");
    bar.set_style(
        ProgressStyle::with_template("[{elapsed_precise}] {spinner} {pos:>7} {msg}")
            .unwrap()
            // For more spinners check out the cli-spinners project:
            // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
            .tick_strings(&["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"]),
    );

    while bam_reader.read_record(&mut record).unwrap() != 0 {
        let readid = record.name().expect("missing read id on BAM record");
        if filter(&record, qual_thresh, length_thresh) {
            let read_id = readid.as_bytes();
            let was_unblocked = if let Some(Ok(_tag)) = record.data().get(b"dx") {
                read_id
                    .split(|b| b == &b';')
                    .any(|b| unblocked_read_ids.contains(&String::from_utf8(b.to_vec()).unwrap()))
            } else {
                unblocked_read_ids.contains(&String::from_utf8(read_id.to_vec()).unwrap())
            };
            if write_unblock && was_unblocked {
                match unblocked_reads_writer.as_mut().unwrap() {
                    Wrapper::Bam(unblocked_bam_writer) => {
                        unblocked_bam_writer
                            .write_record(&_header, &record)
                            .unwrap();
                    }
                    Wrapper::Fastx(unblocked_fastx_writer) => match emit_type {
                        EmitType::Fasta => {
                            write_fasta_record(&record, unblocked_fastx_writer, &mut seq, read_id)?;
                        }
                        EmitType::Fastq => {
                            write_fastq_record(
                                &record,
                                unblocked_fastx_writer,
                                &mut seq,
                                &mut qual,
                                read_id,
                            )?;
                        }
                        _ => unreachable!(),
                    },
                    Wrapper::GzFastx(unblocked_fastx_writer_gz) => match emit_type {
                        EmitType::Fasta => {
                            write_fasta_record(
                                &record,
                                unblocked_fastx_writer_gz,
                                &mut seq,
                                read_id,
                            )?;
                        }
                        EmitType::Fastq => {
                            write_fastq_record(
                                &record,
                                unblocked_fastx_writer_gz,
                                &mut seq,
                                &mut qual,
                                read_id,
                            )?;
                        }
                        _ => unreachable!(),
                    },
                }
            } else if sequenced_reads_writer.is_some() && !was_unblocked {
                let sequence_writer = sequenced_reads_writer.as_mut().unwrap();
                match sequence_writer {
                    Wrapper::Bam(sequenced_bam_writer) => {
                        sequenced_bam_writer
                            .write_record(&_header, &record)
                            .unwrap();
                    }
                    Wrapper::Fastx(sequenced_fastx_writer) => match emit_type {
                        EmitType::Fasta => {
                            write_fasta_record(&record, sequenced_fastx_writer, &mut seq, read_id)?;
                        }
                        EmitType::Fastq => {
                            write_fastq_record(
                                &record,
                                sequenced_fastx_writer,
                                &mut seq,
                                &mut qual,
                                read_id,
                            )?;
                        }
                        _ => unreachable!(),
                    },
                    Wrapper::GzFastx(sequenced_fastx_writer_gz) => match emit_type {
                        EmitType::Fasta => {
                            write_fasta_record(
                                &record,
                                sequenced_fastx_writer_gz,
                                &mut seq,
                                read_id,
                            )?;
                        }
                        EmitType::Fastq => {
                            write_fastq_record(
                                &record,
                                sequenced_fastx_writer_gz,
                                &mut seq,
                                &mut qual,
                                read_id,
                            )?;
                        }
                        _ => unreachable!(),
                    },
                }
            }
        }

        bar.inc(1)
    }
    Ok(())
}

/// Filters a BAM record based on mapping quality and flags.
///
/// This function takes a BAM record and an optional quality threshold (`qual`) as arguments.
/// It returns `true` if the record passes the filters, and `false` otherwise.
///
/// The filters applied are as follows:
///
/// - Quality Threshold: If a `qual` threshold is provided, the average quality of the record
///   must be greater than the specified threshold for the record to pass.
/// - Flags: Records with supplementary or secondary alignment flags are excluded from passing
///   the filter.
///
/// # Arguments
///
/// * `record` - The BAM record to filter.
/// * `qual` - Optional quality threshold. If set, sequences below this average quality will be filtered out.
///
/// # Returns
///
/// Returns `true` if the record passes the filters, and `false` otherwise.
///
/// # Examples
///
/// Filtering a BAM record with no quality threshold:
///
/// ```
/// use crate::_splitting::{filter, _ave_qual};
/// use noodles::bam::{self, record::{Flags, QualityScores}};
///
/// // Create a mock BAM record with quality scores and flags
/// let mut record = bam::Record::default();
/// record.set_quality_scores(QualityScores::from(vec![30, 40, 25, 35]));
/// record.set_flags(Flags::default());
///
/// // Filter the record with no quality threshold
/// assert_eq!(filter(&record, None), true);
/// ```
///
/// Filtering a BAM record with a quality threshold:
///
/// ```
/// use crate::_splitting::{filter, _ave_qual};
/// use noodles::bam::{self, record::{Flags, QualityScores}};
///
/// // Create a mock BAM record with quality scores and flags
/// let mut record = bam::Record::default();
/// record.set_quality_scores(QualityScores::from(vec![30, 40, 25, 35]));
/// record.set_flags(Flags::default());
///
/// // Filter the record with a quality threshold of 35
/// assert_eq!(filter(&record, Some(35)), false);
/// ````
fn filter(record: &bam::Record, qual: Option<usize>, length_thresh: Option<usize>) -> bool {
    let x = if let Some(qual_thresh) = qual {
        let q = _ave_qual(record.quality_scores().as_ref());
        q > qual_thresh as f64
    } else {
        true
    };

    let long_enough = if let Some(length_thresh) = length_thresh {
        let l = record.sequence().len();
        l > length_thresh
    } else {
        true
    };
    let flags = record.flags();

    long_enough && x && !flags.is_supplementary() && !flags.is_secondary()
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use noodles::bam::{self, record::QualityScores, Record};
//     use noodles::sam::alignment::record::Flags;
//     use noodles::sam::alignment::RecordBuf;
//     use std::any::Any;
//     use std::borrow::BorrowMut;
//     use std::path::PathBuf;
//     #[test]
//     fn test_filter_passes_no_quality_threshold() {}
// }
