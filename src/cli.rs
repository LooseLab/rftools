//! Module for defining the command-line interface (CLI) using the `clap` crate.
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::_splitting::{CompressionType, EmitType, SplitType};

/// Represents the command-line arguments for the application.
#[derive(Debug, Parser)]
#[clap(version, about = "Helper tools for after running readfish", long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    /// The subcommand to execute.
    #[clap(subcommand)]
    pub command: Commands,
}

/// Represents the available commands for the application.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Command to split FASTQ into sequenced and unblocked reads.
    #[clap(about = "Split FASTQ into sequenced and unblocked")]
    SplitFQ {
        #[clap(short, long, default_value = "")]
        /// Output file prefix
        prefix: String,

        #[clap(short = 'a', long)]
        /// Write rejected reads as well (default is false)
        write_unblocked: bool,

        // TODO: Maybe accept ONT CSV as either or?
        #[clap(parse(from_os_str))]
        /// Unblocked read ids from readfish
        unblocked_read_ids: PathBuf,

        #[clap(parse(from_os_str))]
        /// Input FASTQ files from MinKNOW
        input_fastq: Vec<PathBuf>,
    },
    #[clap(about = "Split Sequenecing summary into sequenced and unblocked")]
    SplitSS {
        #[clap(short, long, default_value = "")]
        /// Output file prefix
        prefix: String,

        #[clap(parse(from_os_str))]
        /// Unblocked read ids from readfish
        unblocked_read_ids: PathBuf,

        #[clap(parse(from_os_str))]
        /// sequencing_summary.txt file from MinKNOW
        sequencing_summary: PathBuf,
    },
    #[clap(about = "Split BAM files into sequenced and unblocked")]
    SplitBam {
        #[clap(short, long, default_value = "")]
        /// Output file prefix
        prefix: String,
        #[clap(short, long, parse(from_os_str))]
        /// Unblocked read ids file from readfish
        unblocked_read_ids: PathBuf,
        #[clap(short, long, parse(from_os_str))]
        /// Bam file containing reads to be split.
        bam_file: PathBuf,
        /// Write only sequenced reads, unblocked reads, or both. Default is sequenced only.
        #[clap(short, long, default_value_t, value_enum)]
        split_type: SplitType,
        /// Average read quality threshold. If set, reads below this threshold will be filtered out.
        #[clap(short, long)]
        qual_thresh: Option<usize>,
        /// minimum length threshold - If set reads shorter than this threshold will be filtered out..
        #[clap(short, long)]
        length_thresh: Option<usize>,
        /// Write out FASTQ rather than a BAM
        #[clap(long, default_value_t, value_enum)]
        emit_type: EmitType,
        /// Compression type for FASTX output.
        #[clap(short, long, default_value_t, value_enum)]
        compression: CompressionType,
    },
}
