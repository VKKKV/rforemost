# rforemost

A modern, high-performance file carver written in Rust, inspired by the classic `foremost` tool.

`rforemost` uses memory-mapped files and multi-threaded scanning to recover files from disk images or raw data by searching for known headers and trailers.

## Features

- **Blazing Fast**: Leverages `Rayon` for parallel scanning and `memmap2` for efficient I/O.
- **Modular Architecture**: Decoupled engine and carver logic via the `Carver` trait.
- **Supported Formats**:
  - JPEG (`.jpg`)
  - PNG (`.png`)
  - GIF (`.gif`)
  - PDF (`.pdf`)
- **Extensible**: Easily add support for new file formats by implementing a single trait.

## Installation

Ensure you have Rust and Cargo installed.

```bash
git clone https://github.com/vkkkv/foremost-rust.git
cd foremost-rust
cargo build --release
```

The binary will be available at `target/release/rforemost`.

## Usage

```bash
# Scan a disk image and output results to the 'recovered' directory
./rforemost --input disk.img --output recovered

# Use a specific number of threads
./rforemost --input disk.img --threads 4
```

### Options

- `-i, --input <INPUT>`: Path to the input file or disk image.
- `-o, --output <OUTPUT>`: Directory where carved files will be saved (default: `output`).
- `-t, --threads <THREADS>`: Number of threads to use (defaults to CPU count).
- `-h, --help`: Print help information.
- `-V, --version`: Print version information.

## Architecture

`rforemost` is designed for performance and extensibility. 

1. **Memory Mapping**: The input file is mapped into memory, allowing for extremely fast random access without the overhead of repeated syscalls.
2. **Parallel Scanning**: The engine divides the file into chunks and scans them in parallel.
3. **Trait-based Carvers**: Each file format is defined by a `Carver` implementation that handles header matching and length calculation.

## Contributing

Support for more file formats is highly welcome! To add a new format, implement the `Carver` trait in `src/lib.rs` and register it in `src/main.rs`.

## Author

Created by [vkkkv](https://github.com/vkkkv).

## License

This project is licensed under the MIT License - see the LICENSE file for details.
