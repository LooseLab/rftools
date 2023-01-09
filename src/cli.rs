use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(version, about = "Helper tools for after running readfish", long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
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
}
