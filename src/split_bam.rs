use crate::_splitting::{_ave_qual, read_unblocked_read_ids, EmitType, SplitType, Wrapper};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use needletail::sequence;
use noodles::{
    bam::{self},
    sam::alignment::{record::QualityScores, Record},
    sam::record::Sequence,
};
use noodles_bgzf as bgzf;
use std::{
    fs::File,
    io::{BufWriter, Error, Write},
    num::NonZeroUsize,
    path::PathBuf,
    thread,
    u8::MIN,
};
const NEWLINE_SLICE: &[u8] = &[10];
const MIN_VALUE: u8 = b'!';
/// Split a bam file into sequenced and unblocked records.
pub fn split_bam(
    bam_file: PathBuf,
    unblocked_read_ids: PathBuf,
    prefix: String,
    split_type: SplitType,
    qual_thresh: Option<usize>,
    emit_type: EmitType,
) -> Result<(), Error> {
    assert!(bam_file.exists());
    let file = File::open(bam_file)?;

    let worker_count = thread::available_parallelism().unwrap_or(NonZeroUsize::MIN);
    let decoder = bgzf::MultithreadedReader::with_worker_count(worker_count, file);

    let mut bam_reader = bam::io::Reader::from(decoder);
    let _header = bam_reader.read_header()?;
    let unblocked_read_ids = read_unblocked_read_ids(unblocked_read_ids).unwrap();
    let mut valid_number_reads = 0;

    // Choose output file names
    let (seq_fn, unb_fn) = if prefix.is_empty() {
        match emit_type {
            EmitType::Bam => (String::from("sequenced.bam"), String::from("unblocked.bam")),
            EmitType::Fasta => (
                String::from("sequenced.fasta"),
                String::from("unblocked.fasta"),
            ),
            EmitType::Fastq => (
                String::from("sequenced.fastq"),
                String::from("unblocked.fastq"),
            ),
        }
    } else {
        match emit_type {
            EmitType::Bam => (
                format!("{}.sequenced.bam", prefix),
                format!("{}.unblocked.bam", prefix),
            ),
            EmitType::Fasta => (
                format!("{}.sequenced.fasta", prefix),
                format!("{}.unblocked.fasta", prefix),
            ),
            EmitType::Fastq => (
                format!("{}.sequenced.fastq", prefix),
                format!("{}.unblocked.fastq", prefix),
            ),
        }
    };
    // Create outfile writers> first choose sequenecd, unblocked, both then emit type -> Fastx or BAM
    let (mut sequenced_reads_writer, mut unblocked_reads_writer) = match split_type {
        SplitType::All => {
            // What are we emitting
            match emit_type {
                EmitType::Bam => {
                    let mut sbam: bam::io::Writer<bgzf::Writer<File>> =
                        bam::io::Writer::new(File::create(seq_fn).expect("Failed to create file"));
                    let mut ubam =
                        bam::io::Writer::new(File::create(unb_fn).expect("Failed to create file"));
                    sbam.write_header(&_header).unwrap();
                    ubam.write_header(&_header).unwrap();
                    (Some(Wrapper::Bam(sbam)), Some(Wrapper::Bam(ubam)))
                }
                _ => {
                    let mut _sequenced_reads = match File::create(&seq_fn) {
                        Ok(file) => BufWriter::new(file),
                        Err(err) => {
                            eprintln!("Could not create output file: {}\n{}", &seq_fn, err);
                            std::process::exit(1)
                        }
                    };
                    let mut unblocked_reads = match File::create(&unb_fn) {
                        Ok(file) => BufWriter::new(file),
                        Err(err) => {
                            eprintln!("Could not create output file: {}\n{}", &unb_fn, err);
                            std::process::exit(1)
                        }
                    };
                    (
                        Some(Wrapper::Fastx(_sequenced_reads)),
                        Some(Wrapper::Fastx(unblocked_reads)),
                    )
                }
            }
        }
        SplitType::SequencedOnly => match emit_type {
            EmitType::Bam => {
                let mut sbam: bam::io::Writer<bgzf::Writer<File>> =
                    bam::io::Writer::new(File::create(seq_fn).expect("Failed to create file"));
                sbam.write_header(&_header).unwrap();
                (Some(Wrapper::Bam(sbam)), None)
            }
            _ => {
                let mut _sequenced_reads = match File::create(&seq_fn) {
                    Ok(file) => BufWriter::new(file),
                    Err(err) => {
                        eprintln!("Could not create output file: {}\n{}", &seq_fn, err);
                        std::process::exit(1)
                    }
                };
                (Some(Wrapper::Fastx(_sequenced_reads)), None)
            }
        },
        SplitType::UnblockedOnly => match emit_type {
            EmitType::Bam => {
                let mut ubam: bam::io::Writer<bgzf::Writer<File>> =
                    bam::io::Writer::new(File::create(unb_fn).expect("Failed to create file"));
                ubam.write_header(&_header).unwrap();
                (None, Some(Wrapper::Bam(ubam)))
            }
            _ => {
                let mut unblocked_reads = match File::create(&unb_fn) {
                    Ok(file) => BufWriter::new(file),
                    Err(err) => {
                        eprintln!("Could not create output file: {}\n{}", &seq_fn, err);
                        std::process::exit(1)
                    }
                };
                (None, Some(Wrapper::Fastx(unblocked_reads)))
            }
        },
    };
    let mut record = noodles::bam::Record::default();
    let write_unblock = split_type != SplitType::SequencedOnly;
    let mut seq: Vec<u8>;
    let mut qual: Vec<u8>;

    // Setup progress bar
    let bar = ProgressBar::with_draw_target(None, ProgressDrawTarget::stdout())
        .with_message("BAM Records");
    bar.set_style(
        ProgressStyle::with_template("[{elapsed_precise}] {spinner} {pos:>7} {msg}")
            .unwrap()
            // For more spinners check out the cli-spinners project:
            // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
            .tick_strings(&["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"]),
    );

    while bam_reader.read_record(&mut record).unwrap() != 0 {
        let readid = record.name().expect("missing read id on BAM record");
        // Access the sequence bytes

        // Convert the sequence bytes to a string
        if filter(&record, qual_thresh) {
            valid_number_reads += 1;

            let read_id = readid.as_bytes();
            if write_unblock
                && unblocked_read_ids.contains(&String::from_utf8(read_id.to_vec()).unwrap())
            {
                match unblocked_reads_writer.as_mut().unwrap() {
                    Wrapper::Bam(unblocked_bam_writer) => {
                        unblocked_bam_writer
                            .write_record(&_header, &record)
                            .unwrap();
                    }
                    Wrapper::Fastx(unblocked_fastx_writer) => match emit_type {
                        EmitType::Fasta => {
                            seq = record.sequence().iter().map(u8::from).collect();
                            unblocked_fastx_writer.write_all(b">")?;
                            unblocked_fastx_writer.write_all(read_id)?;
                            unblocked_fastx_writer.write_all(NEWLINE_SLICE)?;
                            unblocked_fastx_writer.write_all(&seq)?;
                            unblocked_fastx_writer.write_all(NEWLINE_SLICE)?;
                        }
                        EmitType::Fastq => {
                            seq = record.sequence().iter().map(u8::from).collect();
                            qual = record
                                .quality_scores()
                                .iter()
                                .map(|x| x + MIN_VALUE)
                                .collect();
                            unblocked_fastx_writer.write_all(b"@")?;
                            unblocked_fastx_writer.write_all(read_id)?;
                            unblocked_fastx_writer.write_all(NEWLINE_SLICE)?;
                            unblocked_fastx_writer.write_all(&seq)?;
                            unblocked_fastx_writer.write_all(NEWLINE_SLICE)?;
                            unblocked_fastx_writer.write_all(b"+")?;
                            unblocked_fastx_writer.write_all(NEWLINE_SLICE)?;
                            unblocked_fastx_writer.write_all(&qual)?;
                            unblocked_fastx_writer.write_all(NEWLINE_SLICE)?;
                        }
                        _ => unreachable!(),
                    },
                }
            } else if let Some(ref mut sequence_writer) = sequenced_reads_writer {
                match sequence_writer {
                    Wrapper::Bam(sequenced_bam_writer) => {
                        sequenced_bam_writer
                            .write_record(&_header, &record)
                            .unwrap();
                    }
                    Wrapper::Fastx(sequenced_fastx_writer) => {
                        seq = record.sequence().iter().map(u8::from).collect();
                        qual = record
                            .quality_scores()
                            .iter()
                            .map(|x| x + MIN_VALUE)
                            .collect();
                        sequenced_fastx_writer.write_all(b"@")?;
                        sequenced_fastx_writer.write_all(read_id)?;
                        sequenced_fastx_writer.write_all(NEWLINE_SLICE)?;
                        sequenced_fastx_writer.write_all(&seq)?;
                        sequenced_fastx_writer.write_all(NEWLINE_SLICE)?;
                        sequenced_fastx_writer.write_all(b"+")?;
                        sequenced_fastx_writer.write_all(NEWLINE_SLICE)?;
                        sequenced_fastx_writer.write_all(&qual)?;
                        sequenced_fastx_writer.write_all(NEWLINE_SLICE)?;
                    }
                }
            }
        }

        bar.inc(1)
    }
    Ok(())
}

/// Filters a BAM record based on mapping quality and flags.
fn filter(record: &bam::Record, qual: Option<usize>) -> bool {
    let x = if let Some(qual_thresh) = qual {
        let q = _ave_qual(record.quality_scores().as_ref());
        q > qual_thresh as f64
    } else {
        true
    };
    let flags = record.flags();

    x && !flags.is_supplementary() && !flags.is_secondary()
}
