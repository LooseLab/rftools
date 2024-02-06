rftools
=======
[![Code style: black](https://img.shields.io/badge/code%20style-black-000000.svg)](https://github.com/psf/black)

Rust toolset for working with readfish data/outputs

## Installation

First, ensure you have Rust installed. You can install Rust using [rustup](https://rustup.rs/).

```bash
# Clone the repository
git clone https://github.com/LooseLab/rftools.git
cd rftools

# Build the project
cargo build -r

# Run tests
cargo test
```

This will create an executable, located at `target/release/rftools`. This can be moved to a directory on your `PATH`. 

## Building and Viewing Rust Documentation

The Rust documentation can be built using the `cargo doc` command. This command will generate HTML documentation for all dependencies and your own crate in the `target/doc` directory.

```bash
# Generate documentation
cargo doc
```

Once the documentation is generated, you can open it in your web browser by navigating to the generated HTML files. You can find the entry point to your crate's documentation in `target/doc/{crate_name}/index.html`.

Alternatively, you can use the `--open` flag with `cargo doc` to automatically open the documentation in your default web browser:

```bash
# Generate documentation and open it in the browser
cargo doc --open
```

This will build the documentation and open it in your default web browser automatically.

## Usage

Here's how you can use the project:

```bash
rftools --help
rftools 0.1.0
Helper tools for after running readfish

USAGE:
    rftools <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    help         Print this message or the help of the given subcommand(s)
    split-bam    Split BAM files into sequenced and unblocked
    split-fq     Split FASTQ into sequenced and unblocked
    split-ss     Split Sequenecing summary into sequenced and unblocked
```


> [!WARNING]
> These commands will read the whole of `unblocked_read_ids.txt` into memory!

### Split FQ
Takes in a unblocked_read_ids.txt file, and FASTQ(s). Splits into sequenced and optionally unblocked.

```bash
rftools split-fq --help
rftools-split-fq 0.1.0
Split FASTQ into sequenced and unblocked

USAGE:
    rftools split-fq [OPTIONS] <UNBLOCKED_READ_IDS> [INPUT_FASTQ]...

ARGS:
    <UNBLOCKED_READ_IDS>    Unblocked read ids from readfish
    <INPUT_FASTQ>...        Input FASTQ files from MinKNOW

OPTIONS:
    -a, --write-unblocked    Write rejected reads as well (default is false)
    -h, --help               Print help information
    -p, --prefix <PREFIX>    Output file prefix [default: ]
    -V, --version            Print version information
```


Example:

```bash
# Splitting FASTQ files, writing out both sequenced and unblocked
rftools split-fq --write-unblocked --prefix example_split unblocked_read_ids.txt input.fq
```

### Split sequencing summary
```bash
rftools split-ss --help
rftools-split-ss 0.1.0
Split Sequenecing summary into sequenced and unblocked

USAGE:
    rftools split-ss [OPTIONS] --unblocked-read-ids <UNBLOCKED_READ_IDS> --sequencing-summary <SEQUENCING_SUMMARY>

OPTIONS:
    -h, --help                                       Print help information
    -p, --prefix <PREFIX>                            Output file prefix [default: ]
    -s, --sequencing-summary <SEQUENCING_SUMMARY>    sequencing_summary.txt file from MinKNOW
    -u, --unblocked-read-ids <UNBLOCKED_READ_IDS>    Unblocked read ids from readfish
    -V, --version                                    Print version information
```

Example:
```bash
# Splitting Sequencing Summary file, only writing out sequenced
rftools split-ss --prefix output unblocked_read_ids.txt sequencing_summary.txt
```

### Splitting BAM

```bash
rftools split-bam --help
rftools-split-bam 0.1.0
Split BAM files into sequenced and unblocked

USAGE:
    rftools split-bam [OPTIONS] --unblocked-read-ids <UNBLOCKED_READ_IDS> --bam-file <BAM_FILE>

OPTIONS:
    -b, --bam-file <BAM_FILE>
            Bam file containing reads to be split

    -c, --compression <COMPRESSION>
            Compression type for FASTX output [default: gzipped] [possible values: gzipped,
            bgzipped, uncompressed]

        --emit-type <EMIT_TYPE>
            Write out FASTQ rather than a BAM [default: bam] [possible values: bam, fastq, fasta]

    -h, --help
            Print help information

    -l, --length-thresh <LENGTH_THRESH>
            minimum length threshold - If set reads shorter than this threshold will be filtered
            out. [default: 0]

    -p, --prefix <PREFIX>
            Output file prefix [default: ]

    -q, --qual-thresh <QUAL_THRESH>
            Average read quality threshold. If set, reads below this threshold will be filtered out

    -s, --split-type <SPLIT_TYPE>
            Write only sequenced reads, unblocked reads, or both. Default is sequenced only
            [default: sequenced-only] [possible values: all, unblocked-only, sequenced-only]

    -u, --unblocked-read-ids <UNBLOCKED_READ_IDS>
            Unblocked read ids file from readfish

    -V, --version
            Print version information
```

Examples: 

```bash
# Splitting BAM file, writing out both unblocked and sequenced records into sequenced and unblocked bam files, filtering to Q score > 20.
rftools split-bam --unblocked-read-ids ids.txt --bam-file input.bam --split-type all --qual-thresh 20
# Splitting BAM file, writing out only unblocked records into a FASTQ file.
rftools split-bam --unblocked-read-ids ids.txt --bam-file input.bam --split-type unblocked-only --emit-type fastq
```

By default all Fastx output is gzipped to the current systems default level. This can be disabled by setting

```bash
-c uncompressed
```

## Contributing

We welcome contributions! If you'd like to contribute to this project, please follow these guidelines:

1. Fork the repository.
2. Create a new branch (`git checkout -b feature-or-fix`).
3. Make your changes.
4. Ensure tests pass (`cargo test`).
5. Commit your changes (`git commit -am 'Add new feature'`).
6. Push to the branch (`git push origin feature-or-fix`).
7. Create a new Pull Request.

Please make sure to update tests and documentation as appropriate.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
```
