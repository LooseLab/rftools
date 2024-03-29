use crate::_splitting::read_unblocked_read_ids;
use needletail::parse_fastx_file;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    str,
};

const SPACE: u8 = 32;
const NEWLINE_SLICE: &[u8] = &[10];
// https://play.rust-lang.org/?version=stable&mode=release&edition=2018&gist=9b6ce8097023be0ac3206abca036c35d

/// Convert FASTX header line to an id by splitting on SPACE
/// and converting to utf8. We don't check that all bytes are
/// actually utf8, which we could with `bytes.it_ascii()`.
///
/// # Example
///
/// ```
/// let a = [65, 108, 101, 120, 32, 119, 97, 115, 32, 104, 101, 114, 101];
/// let id = header_to_id(&a);
/// assert_eq!(id, "Alex");
/// ```
fn header_to_id(bytes: &[u8]) -> &str {
    match bytes.iter().position(|&char| char == SPACE) {
        None => str::from_utf8(bytes).unwrap(),
        Some(x) => str::from_utf8(&bytes[..x]).unwrap(),
    }
}

pub fn split(
    unblocked_read_ids: PathBuf,
    prefix: String,
    input_fastq: Vec<PathBuf>,
    write_unblocked: bool,
) {
    // Read our unblocked read ids into a hash set
    let rejected_reads = match read_unblocked_read_ids(unblocked_read_ids) {
        Ok(hs) => hs,
        Err(e) => {
            eprintln!("Error: could not read unblocked_read_ids\n{}", e);
            std::process::exit(1)
        }
    };
    let seq_fn: String;
    let unb_fn: String;

    // Parse if we have a prefix
    if prefix.is_empty() {
        seq_fn = String::from("sequenced.fastq");
        unb_fn = String::from("unblocked.fastq");
    } else {
        seq_fn = format!("{}.sequenced.fastq", prefix);
        unb_fn = format!("{}.unblocked.fastq", prefix);
    }

    let mut _sequenced_reads = match File::create(&seq_fn) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Could not create output file: {}\n{}", &seq_fn, err);
            std::process::exit(1)
        }
    };
    let mut sequenced_reads: BufWriter<File> = BufWriter::new(_sequenced_reads);
    let mut unblocked_reads: Option<BufWriter<File>>;
    if write_unblocked {
        unblocked_reads = match File::create(&unb_fn) {
            Ok(file) => Some(BufWriter::new(file)),
            Err(err) => {
                eprintln!("Could not create output file: {}\n{}", &unb_fn, err);
                std::process::exit(1)
            }
        };
    } else {
        unblocked_reads = None
    }

    // Process each FASTQ file
    for path in input_fastq {
        let mut reader = match parse_fastx_file(&path) {
            Ok(reader) => reader,
            Err(_) => {
                eprintln!("Could not read FASTA/Q file: {:#?}", path);
                std::process::exit(1)
            }
        };
        while let Some(record) = &reader.next() {
            let record = match record.as_ref() {
                Ok(rec) => rec,
                Err(err) => {
                    eprintln!("Invalid record in file {:#?}\n{}", path, err);
                    std::process::exit(1)
                }
            };
            let id = header_to_id(record.id());
            if rejected_reads.contains(id) {
                // ID in HashSet, unblock was sent
                if let Some(ref mut file) = unblocked_reads {
                    file.write_all(record.all()).expect("???");
                    file.write_all(NEWLINE_SLICE).expect("askjd");
                }
            } else {
                sequenced_reads.write_all(record.all()).expect("???");
                sequenced_reads.write_all(NEWLINE_SLICE).expect("askjd");
            }
        }
    }
}
