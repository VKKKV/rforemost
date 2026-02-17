use anyhow::Result;
use clap::Parser;
use memmap2::MmapOptions;
use rayon::prelude::*;
use rforemost::{Carver, GifCarver, JpegCarver, PdfCarver, PngCarver, save_file};
use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::Arc;

/// A high-performance Rust implementation of the foremost file carving tool.
#[derive(Parser)]
#[command(
    author = "vkkkv",
    version,
    about = "A modern, high-performance file carver written in Rust."
)]
struct Args {
    /// Input disk image or file to scan
    #[arg(short, long)]
    input: PathBuf,

    /// Directory where carved files will be saved
    #[arg(short, long, default_value = "output")]
    output: PathBuf,

    /// Number of threads to use (defaults to CPU count)
    #[arg(short, long)]
    threads: Option<usize>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize the thread pool if specified
    if let Some(t) = args.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(t)
            .build_global()?;
    }

    fs::create_dir_all(&args.output)?;

    let file = File::open(&args.input)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };

    // Register supported carvers
    let carvers: Vec<Arc<dyn Carver>> = vec![
        Arc::new(JpegCarver),
        Arc::new(PngCarver),
        Arc::new(GifCarver),
        Arc::new(PdfCarver),
    ];

    println!(
        "rforemost v{} - Starting scan of {} bytes",
        env!("CARGO_PKG_VERSION"),
        mmap.len()
    );

    // Optimization: Identify the first byte of every possible header magic.
    // This allows us to skip bytes that cannot possibly be the start of a header.
    let mut first_bytes = [false; 256];
    for carver in &carvers {
        first_bytes[carver.header_magic()[0] as usize] = true;
    }
    let first_bytes = Arc::new(first_bytes);

    // Use a larger chunk size to reduce Rayon overhead and improve cache locality.
    let chunk_size = 1024 * 1024; // 1MB chunks
    let total_len = mmap.len();

        (0..total_len)
            .into_par_iter()
            .step_by(chunk_size)
            .for_each(|chunk_start| {
                let data = &mmap[..];
                
                for offset in chunk_start..std::cmp::min(chunk_start + chunk_size, total_len) {
                    // Quick check: skip if the current byte doesn't match any known header start.
                    if !first_bytes[data[offset] as usize] {
                        continue;
                    }
    
                                    for carver in &carvers {
    
                                        if carver.matches_header(data, offset)
    
                                            && let Some(size) = carver.extract(data, offset)
    
                                        {
    
                                            let file_data = &data[offset..offset + size];
    
                                            let filename = format!("file_{:08}.{}", offset, carver.extension());
    
                                            let path = args.output.join(filename);
    
                    
    
                                            if let Err(e) = save_file(&path, file_data) {
    
                                                eprintln!("Error saving file at offset {}: {}", offset, e);
    
                                            }
    
                                        }
    
                                    }
    
                    
                }
            });
        println!("Scan complete. Recovered files are in {:?}", args.output);
    Ok(())
}
