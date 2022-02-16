use fst::Set;
// use rustc_hash::FxHashSet;
use std::{
    // collections::HashSet,
    fs::File,
    // io::{BufRead, BufReader},
    io::Read,
    path::PathBuf,
};

fn get_key_col(first_line: &csv::ByteRecord) -> Result<usize, String> {
    for (i, field) in first_line.iter().enumerate() {
        if field == b"read_id" {
            return Ok(i);
        }
    }
    Err("Could not find field".to_owned())
}

pub fn split(unblocked_read_ids: PathBuf, sequencing_summary: PathBuf) {
    // Read our unblocked read ids into a hash set
    // TODO: Do a file size check here, if greater than CLI 
    //       value maybe use memmap instead
    // Here instead of a HashSet unblocked ids are expected to be an FST
    // which is produced by the fst-bin crate:
    //   sort -o <file> <file>
    //   fst set --sorted <file> <file.fst>
    let set = match File::open(unblocked_read_ids) {
        Ok(mut file) => {
            let mut bytes = vec![];
            file.read_to_end(&mut bytes).expect("?");
            Set::new(bytes).expect("msg")
        },
        _ => {
            eprintln!("Error!");
            std::process::exit(1);
        }
    };

    // Finally, we can query.
    println!("number of elements: {}", set.len());

    let mut rdr = match File::open(sequencing_summary) {
        Ok(file) => csv::ReaderBuilder::new().delimiter(b'\t').from_reader(file),
        _ => {
            eprintln!("Error CSV reader!");
            std::process::exit(1);
        }
    };
    let key_col = get_key_col(&rdr.byte_headers().expect("A")).expect("asdbf"); 
    let headers = &rdr.byte_headers().expect("a").clone();
    let mut wtr_s = csv::WriterBuilder::new().delimiter(b'\t').from_path("sequenced.rs.txt").expect("seq wtr");
    let mut wtr_r = csv::WriterBuilder::new().delimiter(b'\t').from_path("unblocked.rs.txt").expect("unb wtr");
    wtr_r.write_byte_record(&headers).expect("1");
    wtr_s.write_byte_record(&headers).expect("2");
    let mut row = csv::ByteRecord::new();
    let mut s = 0;
    let mut r = 0;
    while rdr.read_byte_record(&mut row).expect("x") {
        // Decide what file to put this in.
        // let read_id = &row[key_col];
        if set.contains(&row[key_col]) {
            r += 1;
            wtr_r.write_byte_record(&row).expect("3");
        } else {
            s += 1;
            wtr_s.write_byte_record(&row).expect("4");
        }
        // println!("{:?} {:?}", std::str::from_utf8(&read_id), set.contains(&read_id));
        // break
    }
    println!("Sequenced: {:?}\nUnblocked: {:?}", s, r);


    // let hs = match File::open(unblocked_read_ids) {
    //     Ok(file) => {
    //         let reader = BufReader::new(file);
    //         let rejected_reads: HashSet<String> = reader
    //             .lines()
    //             .map(|l| l.expect("a"))
    //             .collect();
    //         rejected_reads
    //     }
    //     _ => {
    //         eprintln!("Error!");
    //         std::process::exit(1);
    //     }
    // };
    // println!("Got {} entires", hs.len());
}
