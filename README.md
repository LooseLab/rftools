rftools
=======

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
# Splitting FASTQ files, writing out both sequenced and unblocked
rftools split-fq --write-unblocked --prefix example_split unblocked_read_ids.txt input.fq
# Splitting Sequencing Summary file, only writing out sequenced
rftools split-ss --prefix output unblocked_read_ids.txt sequencing_summary.txt
# Splitting BAM file, writing out both unblocked and sequenced records into sequenced and unblocked bam files, filtering to Q score > 20.
rftools split-bam --unblocked-read-ids ids.txt --bam-file input.bam --split-type all --qual-thresh 20
# Splitting BAM file, writing out only unblocked records into a FASTQ file.
rftools split-bam --unblocked-read-ids ids.txt --bam-file input.bam --split-type unblocked-only --emit-type fastq
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
