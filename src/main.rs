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
            emit_type,
        } => crate::split_bam::split_bam(
            bam_file,
            unblocked_read_ids,
            prefix,
            split_type,
            qual_thresh,
            emit_type,
        )
        .unwrap(),
    };
}
