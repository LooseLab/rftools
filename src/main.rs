#![forbid(unsafe_code)]
//! # RfTools
//!
//! This is a simple, fast Rust project that implements simple parsing of ONT sequencing
//! runs using the the `unblocked_read_ids` file.
//!
//!
//! ## Modules
//!
//! - `_splitting`: Module for shared splitting operations.
//! - `cli`: Module for command-line interface (CLI) handling, using Clap.
//! - `split_bam`: Module for splitting BAM files.
//! - `split_fq`: Module for splitting FASTQ files.
//! - `split_ss`: Module for splitting sequencing summary files.
//!
//! ## Usage
//!
//! This application provides functionality for splitting various types of files.
//! See the individual modules for more details.
//!
//! ## Example
//!
//! ```bash
//! # Splitting FASTQ files, writing out both sequenced and unblocked
//! cargo run -- split-fq --write-unblocked --prefix example_split unblocked_read_ids.txt input.fq
//!
//! # Splitting Sequencing Summary file, only writing out sequenced
//! cargo run -- split-ss --prefix output unblocked_read_ids.txt sequencing_summary.txt
//!
//! # Splitting BAM file, writing out both unblocked and sequenced records into sequenced and unblocked bam files, filtering to Q score > 20.
//! cargo run -- split-bam --unblocked-read-ids ids.txt --bam-file input.bam --split-type all --qual-thresh 20
//!
//! # Splitting BAM file, writing out only unblocked records into a FASTQ file.
//! cargo run -- split-bam --unblocked-read-ids ids.txt --bam-file input.bam --split-type unblocked-only --emit-type fastq
//! ```
//!
//! ## Error Handling
//!
//! Error handling follows the guidelines described in [BurntSushi's Blog](https://blog.burntsushi.net/rust-error-handling/).
//!
mod _splitting;
mod cli;
mod split_bam;
mod split_fq;
mod split_ss;
use crate::cli::{Cli, Commands};
use clap::Parser;

fn main() {
    // Better error handling, crate funcs should return
    //   errors that can be propagated back here.
    // https://blog.burntsushi.net/rust-error-handling/
    let args = Cli::parse();

    // let res = match args {
    match args.command {
        Commands::SplitFQ {
            unblocked_read_ids,
            prefix,
            input_fastq,
            write_unblocked,
        } => crate::split_fq::split(unblocked_read_ids, prefix, input_fastq, write_unblocked),
        Commands::SplitSS {
            unblocked_read_ids,
            prefix,
            sequencing_summary,
        } => crate::split_ss::split(unblocked_read_ids, prefix, sequencing_summary),
        Commands::SplitBam {
            prefix,
            unblocked_read_ids,
            bam_file,
            split_type,
            qual_thresh,
            length_thresh,
            emit_type,
            compression,
        } => crate::split_bam::split_bam(
            bam_file,
            unblocked_read_ids,
            prefix,
            split_type,
            qual_thresh,
            length_thresh,
            emit_type,
            compression,
        )
        .unwrap(),
    };
}
