use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "rftools",
    about = "Helper tools for after running readfish",
    rename_all = "kebab-case"
)]
pub enum Commands {
    #[structopt(about = "Split FASTQ into sequenced and unblocked")]
    Split {
        // TODO: Maybe accept ONT CSV as either or?
        #[structopt(short, long, parse(from_os_str))]
        /// Unblocked read ids from readfish
        unblocked_read_ids: PathBuf,

        #[structopt(short, long, default_value = "")]
        /// Output file prefix
        prefix: String,

        #[structopt(parse(from_os_str))]
        /// Input FASTQ files from MinKNOW
        input_fastq: Vec<PathBuf>,

        #[structopt(short = "a", long)]
        /// Write rejected reads as well (default is false)
        write_unblocked: bool,
    },
    SplitSeqSum {
        #[structopt(short, long, parse(from_os_str))]
        /// Unblocked read ids from readfish
        unblocked_read_ids: PathBuf,

        #[structopt(short, long, parse(from_os_str))]
        /// sequencing_summary.txt file from MinKNOW
        sequencing_summary: PathBuf,
    },
}
