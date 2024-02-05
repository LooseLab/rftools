use clap::ValueEnum;
use fnv::FnvHashSet;
use noodles::bam;
use noodles_bgzf as bgzf;

use std::{
    fs::File,
    io,
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
};

#[derive(Debug, ValueEnum, Clone, Default, PartialEq)]
pub enum SplitType {
    All,
    UnblockedOnly,
    #[default]
    SequencedOnly,
}

#[derive(Debug, ValueEnum, Clone, Default, PartialEq)]
pub enum EmitType {
    #[default]
    Bam,
    Fastq,
    Fasta,
}

pub enum Wrapper {
    Bam(bam::io::Writer<bgzf::Writer<File>>),
    Fastx(BufWriter<File>),
}

/// Read unblocked_read_ids.txt into a HashSet
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
pub fn _ave_qual(quals: &[u8]) -> f64 {
    let probability_sum = quals
        .iter()
        .map(|q| 10_f64.powf(((*q) as f64) / -10.0))
        .sum::<f64>();
    (probability_sum / quals.len() as f64).log10() * -10.0
}
