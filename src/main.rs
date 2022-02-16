use structopt::StructOpt;
mod cli;
mod split;
mod split_seq_sum;
use crate::cli::Commands;

fn main() {
    // https://blog.burntsushi.net/rust-error-handling/
    let args = Commands::from_args();

    // let res = match args {
    match args {
        Commands::Split {
            unblocked_read_ids,
            prefix,
            input_fastq,
            write_unblocked,
        } => crate::split::split(unblocked_read_ids, prefix, input_fastq, write_unblocked),
        Commands::SplitSeqSum {
            unblocked_read_ids,
            sequencing_summary,
        } => crate::split_seq_sum::split(unblocked_read_ids, sequencing_summary),
    };
    // match res {
    //     Ok(()) => {
    //         std::process::exit(0);
    //     }
    //     Err(err) => {
    //         eprintln!("{}", err);
    //         std::process::exit(1);
    //     }
    // }
}

