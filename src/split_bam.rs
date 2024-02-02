use crate::_splitting::{read_unblocked_read_ids, SplitType};
use noodles::bam;
use noodles_bgzf as bgzf;
use std::io::{BufWriter, Write};
use std::{fs::File, io::Error, num::NonZeroUsize, path::PathBuf, thread};
/// Split a bam file into sequenced and unblocked records.
pub fn split_bam(
    bam_file: PathBuf,
    unblocked_read_ids: PathBuf,
    prefix: String,
    split_type: SplitType,
) -> Result<(), Error> {
    let file = File::open(bam_file)?;

    let worker_count = thread::available_parallelism().unwrap_or(NonZeroUsize::MIN);
    let decoder = bgzf::MultithreadedReader::with_worker_count(worker_count, file);

    let mut bam_reader = bam::Reader::from(decoder);

    let unblocked_read_ids = read_unblocked_read_ids(unblocked_read_ids).unwrap();
    let mut record = bam::lazy::Record::default();
    let mut valid_number_reads = 0;

    let (seq_fn, unb_fn) = if prefix.is_empty() {
        (
            String::from("sequenced.fastq"),
            String::from("unblocked.fastq"),
        )
    } else {
        (
            format!("{}.sequenced.fastq", prefix),
            format!("{}.unblocked.fastq", prefix),
        )
    };
    let (sequenced_reads_file, unblocked_reads_file) = match split_type {
        SplitType::All => (
            Some(File::create(seq_fn).expect("Failed to create file")),
            Some(File::create(unb_fn).expect("Failed to create file")),
        ),
        SplitType::SequencedOnly => (
            Some(File::create(seq_fn).expect("Failed to create file")),
            None,
        ),
        SplitType::UnblockedOnly => (
            None,
            Some(File::create(seq_fn).expect("Failed to create file")),
        ),
    };
    // Open a file for writing
    let file = File::create("output.txt").expect("Failed to create file");

    // Wrap the file in a BufWriter
    let mut buf_writer = BufWriter::new(file);
    // Create outfile writers

    while bam_reader.read_lazy_record(&mut record)? != 0 {
        let readid = record.read_name().expect("missing read id on BAM record");
        if filter(&record) {
            valid_number_reads += 1;
            if unblocked_read_ids.contains(&String::from_utf8(readid.as_bytes().to_vec()).unwrap())
            {
                println!("BAm");
            }
        }

        // bar.inc(1)
    }
    Ok(())
}

/// Filters a BAM record based on mapping quality and flags.
fn filter(record: &bam::lazy::Record) -> bool {
    let flags = record.flags();

    !flags.is_supplementary() && !flags.is_secondary()
}
