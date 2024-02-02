use fnv::FnvHashSet;
use std::{
    fs::File,
    io,
    io::{BufRead, BufReader},
    path::PathBuf,
};

/// Read unblocked_read_ids.txt into a HashSet
fn read_unblocked_read_ids(path: PathBuf) -> Result<FnvHashSet<String>, io::Error> {
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

fn get_key_col(first_line: &csv::ByteRecord) -> Result<usize, String> {
    for (i, field) in first_line.iter().enumerate() {
        if field == b"read_id" {
            return Ok(i);
        }
    }
    Err("Could not find field".to_owned())
}

pub fn split(unblocked_read_ids: PathBuf, prefix: String, sequencing_summary: PathBuf) {
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
        seq_fn = String::from("sequenced.txt");
        unb_fn = String::from("unblocked.txt");
    } else {
        seq_fn = format!("{}.sequenced.txt", prefix);
        unb_fn = format!("{}.unblocked.txt", prefix);
    }

    let mut rdr = match File::open(sequencing_summary) {
        Ok(file) => csv::ReaderBuilder::new().delimiter(b'\t').from_reader(file),
        _ => {
            eprintln!("Error CSV reader!");
            std::process::exit(1);
        }
    };
    let key_col = get_key_col(rdr.byte_headers().expect("A")).expect("asdbf");
    let headers = &rdr.byte_headers().expect("a").clone();
    let mut wtr_s = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_path(seq_fn)
        .expect("seq wtr");
    let mut wtr_r = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_path(unb_fn)
        .expect("unb wtr");
    wtr_r.write_byte_record(headers).expect("1");
    wtr_s.write_byte_record(headers).expect("2");
    let mut row = csv::ByteRecord::new();
    let mut s = 0;
    let mut r = 0;
    while rdr.read_byte_record(&mut row).expect("x") {
        // Decide what file to put this in.
        if rejected_reads.contains(std::str::from_utf8(&row[key_col]).unwrap()) {
            r += 1;
            wtr_r.write_byte_record(&row).expect("3");
        } else {
            s += 1;
            wtr_s.write_byte_record(&row).expect("4");
        }
    }
    println!("Sequenced: {:?}\nUnblocked: {:?}", s, r);
}
