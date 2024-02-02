use clap::ValueEnum;
use fnv::FnvHashSet;
use std::{
    fs::File,
    io,
    io::{BufRead, BufReader},
    path::PathBuf,
};

#[derive(Debug, ValueEnum, Clone, Default)]
pub enum SplitType {
    #[default]
    All,
    UnblockedOnly,
    SequencedOnly,
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
