use anyhow::Result;
use byteorder::{BigEndian, ByteOrder};
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// The core trait for file recovery.
///
/// A `Carver` is responsible for identifying the start of a file (header)
/// and determining its length by parsing its internal structure.
pub trait Carver: Send + Sync {
    /// Returns the file extension associated with this carver (e.g., "jpg").
    fn extension(&self) -> &str;

    /// Returns the magic bytes used to identify the file format's header.
    fn header_magic(&self) -> &[u8];

    /// Checks if the data at the given offset matches the format's header.
    fn matches_header(&self, data: &[u8], offset: usize) -> bool {
        let magic = self.header_magic();
        offset + magic.len() <= data.len() && &data[offset..offset + magic.len()] == magic
    }

    /// Calculates the file size by parsing internal structure.
    ///
    /// Returns the total size from `start_offset` if a valid file is found.
    fn extract(&self, data: &[u8], start_offset: usize) -> Option<usize>;
}

/// JPEG File Carver implementation.
pub struct JpegCarver;

impl Carver for JpegCarver {
    fn extension(&self) -> &str {
        "jpg"
    }

    fn header_magic(&self) -> &[u8] {
        &[0xFF, 0xD8]
    }

    fn extract(&self, data: &[u8], start_offset: usize) -> Option<usize> {
        if !self.matches_header(data, start_offset) {
            return None;
        }

        let mut pos = start_offset + 2;
        while pos + 1 < data.len() {
            if data[pos] != 0xFF {
                return None;
            }
            let marker = data[pos + 1];
            match marker {
                0xD9 => return Some(pos + 2 - start_offset), // End of Image found
                0xDA => {
                    // Start of Scan - The compressed bitstream follows.
                    // We must scan for the EOI marker.
                    let segment_len = BigEndian::read_u16(&data[pos + 2..pos + 4]) as usize;
                    let bitstream_start = pos + 2 + segment_len;
                    return data[bitstream_start..]
                        .windows(2)
                        .position(|w| w == [0xFF, 0xD9])
                        .map(|off| bitstream_start + off + 2 - start_offset);
                }
                0x01 | 0xD0..=0xD7 => pos += 2, // Standalone markers
                _ => {
                    if pos + 4 > data.len() {
                        return None;
                    }
                    let segment_len = BigEndian::read_u16(&data[pos + 2..pos + 4]) as usize;
                    pos += 2 + segment_len; // Skip the segment
                }
            }
        }
        None
    }
}

/// PNG File Carver implementation.
pub struct PngCarver;

impl Carver for PngCarver {
    fn extension(&self) -> &str {
        "png"
    }

    fn header_magic(&self) -> &[u8] {
        &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
    }

    fn extract(&self, data: &[u8], start_offset: usize) -> Option<usize> {
        if !self.matches_header(data, start_offset) {
            return None;
        }

        let mut pos = start_offset + 8; // Skip PNG signature
        while pos + 8 <= data.len() {
            let length = BigEndian::read_u32(&data[pos..pos + 4]) as usize;
            let chunk_type = &data[pos + 4..pos + 8];
            let chunk_total = 12 + length; // 4 (len) + 4 (type) + length + 4 (crc)

            pos += chunk_total;
            if chunk_type == b"IEND" {
                return Some(pos - start_offset);
            }
            if pos > data.len() {
                break;
            }
        }
        None
    }
}

/// GIF File Carver implementation.
pub struct GifCarver;

impl Carver for GifCarver {
    fn extension(&self) -> &str {
        "gif"
    }

    fn header_magic(&self) -> &[u8] {
        // GIF supports two main versions. We'll use GIF87a as the primary magic
        // but handle GIF89a in matches_header if needed. For simplicity, we use a custom check.
        b"GIF8"
    }

    fn matches_header(&self, data: &[u8], offset: usize) -> bool {
        offset + 6 <= data.len()
            && (&data[offset..offset + 6] == b"GIF87a" || &data[offset..offset + 6] == b"GIF89a")
    }

    fn extract(&self, data: &[u8], start_offset: usize) -> Option<usize> {
        if !self.matches_header(data, start_offset) {
            return None;
        }

        // GIF trailer is 0x3B. We scan for the first trailer byte.
        data[start_offset..]
            .iter()
            .position(|&b| b == 0x3B)
            .map(|pos| pos + 1)
    }
}

/// PDF File Carver implementation.
pub struct PdfCarver;

impl Carver for PdfCarver {
    fn extension(&self) -> &str {
        "pdf"
    }

    fn header_magic(&self) -> &[u8] {
        b"%PDF"
    }

    fn extract(&self, data: &[u8], start_offset: usize) -> Option<usize> {
        if !self.matches_header(data, start_offset) {
            return None;
        }

        // PDF carving is complex due to incremental updates.
        // We look for the last %%EOF within a 10MB window.
        let limit = 10 * 1024 * 1024;
        let end = std::cmp::min(data.len(), start_offset + limit);
        let search_range = &data[start_offset..end];

        let trailer = b"%%EOF";
        search_range
            .windows(trailer.len())
            .enumerate()
            .rev() // Search from the end for the last %%EOF
            .find(|(_, w)| *w == trailer)
            .map(|(offset, _)| offset + trailer.len())
    }
}

/// Saves the carved data to the specified path.
pub fn save_file(path: &Path, data: &[u8]) -> Result<()> {
    let mut f = File::create(path)?;
    f.write_all(data)?;
    Ok(())
}
