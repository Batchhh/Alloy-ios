//! Memory scanning and pattern matching utilities

use crate::memory::image;
use crate::memory::info::protection;
use crate::utils::logger;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
/// Errors that can occur during memory scanning
pub enum ScanError {
    /// The pattern string (IDA style or mask) is invalid
    #[error("Invalid pattern format: {0}")]
    InvalidPattern(String),
    /// The specified pattern was not found in the target range
    #[error("Pattern not found")]
    NotFound,
    /// The scan attempted to access invalid or protected memory
    #[error("Memory access violation at {0:#x}")]
    MemoryAccessViolation(usize),
    /// The memory region definition is invalid
    #[error("Invalid memory region")]
    InvalidRegion,
    /// Image lookup failed
    #[error("Image not found: {0}")]
    ImageNotFound(#[from] super::image::ImageError),
}

static SCAN_CACHE: Lazy<Mutex<HashMap<String, Vec<usize>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Parses an IDA-style pattern string (e.g., "A1 ?? B2") into bytes and mask
///
/// # Arguments
/// * `pattern` - The pattern string (e.g., "DE AD BE EF" or "DE ?? BE EF")
///
/// # Returns
/// * `Result<(Vec<u8>, String), ScanError>` - A tuple containing the byte vector and the mask string
pub fn parse_ida_pattern(pattern: &str) -> Result<(Vec<u8>, String), ScanError> {
    let parts: Vec<&str> = pattern.split_whitespace().collect();
    let mut bytes = Vec::new();
    let mut mask = String::new();
    for part in parts {
        if part == "??" {
            bytes.push(0);
            mask.push('?');
        } else if part.len() == 2 {
            bytes.push(
                u8::from_str_radix(part, 16)
                    .map_err(|_| ScanError::InvalidPattern(format!("Invalid hex: {}", part)))?,
            );
            mask.push('x');
        } else {
            return Err(ScanError::InvalidPattern(format!(
                "Invalid pattern part: {}",
                part
            )));
        }
    }
    if bytes.is_empty() {
        return Err(ScanError::InvalidPattern("Empty pattern".to_string()));
    }
    Ok((bytes, mask))
}

/// Scans for a pattern within a memory range, returning all matches
///
/// # Arguments
/// * `start` - The start address of the scan
/// * `size` - The size of the memory range to scan
/// * `pattern` - The byte sequence to find
/// * `mask` - The mask string ('x' for match, '?' for wildcard)
///
/// # Returns
/// * `Result<Vec<usize>, ScanError>` - A list of addresses where the pattern matches
pub fn scan_pattern(
    start: usize,
    size: usize,
    pattern: &[u8],
    mask: &str,
) -> Result<Vec<usize>, ScanError> {
    if pattern.is_empty() || pattern.len() != mask.len() {
        return Err(ScanError::InvalidPattern(
            "Pattern and mask length mismatch".to_string(),
        ));
    }
    if !is_readable_memory(start, size) {
        return Err(ScanError::MemoryAccessViolation(start));
    }
    let mut results = Vec::new();
    let end = start + size - pattern.len();
    for addr in start..=end {
        if pattern_match(addr, pattern, mask) {
            results.push(addr);
        }
    }
    if results.is_empty() {
        return Err(ScanError::NotFound);
    }
    Ok(results)
}

/// Scans for an IDA-style pattern within a memory range, returning all matches
///
/// # Arguments
/// * `start` - The start address
/// * `size` - The size of the range
/// * `ida_pattern` - The pattern string (e.g., "DE ?? BE EF")
///
/// # Returns
/// * `Result<Vec<usize>, ScanError>` - A list of addresses or an error
pub fn scan_ida_pattern(
    start: usize,
    size: usize,
    ida_pattern: &str,
) -> Result<Vec<usize>, ScanError> {
    let (bytes, mask) = parse_ida_pattern(ida_pattern)?;
    scan_pattern(start, size, &bytes, &mask)
}

/// Scans an entire image for an IDA-style pattern, returning all matches
///
/// # Arguments
/// * `image_name` - The name of the image to scan
/// * `ida_pattern` - The pattern string
///
/// # Returns
/// * `Result<Vec<usize>, ScanError>` - A list of addresses or an error
pub fn scan_image(image_name: &str, ida_pattern: &str) -> Result<Vec<usize>, ScanError> {
    let base = image::get_image_base(image_name)?;
    let sections = get_image_sections(base)?;
    let (bytes, mask) = parse_ida_pattern(ida_pattern)?;
    let mut all_results = Vec::new();
    for (section_start, section_size) in sections {
        if let Ok(mut results) = scan_pattern(section_start, section_size, &bytes, &mask) {
            all_results.append(&mut results);
        }
    }
    if all_results.is_empty() {
        return Err(ScanError::NotFound);
    }
    Ok(all_results)
}

/// Scans for an IDA-style pattern with caching support
///
/// Subsequent calls with the same parameters will return cached results.
///
/// # Arguments
/// * `start` - The start address
/// * `size` - The scan size
/// * `ida_pattern` - The pattern string
///
/// # Returns
/// * `Result<Vec<usize>, ScanError>` - A list of addresses or an error
pub fn scan_pattern_cached(
    start: usize,
    size: usize,
    ida_pattern: &str,
) -> Result<Vec<usize>, ScanError> {
    let cache_key = format!("{:#x}_{:#x}_{}", start, size, ida_pattern);
    {
        let cache = SCAN_CACHE.lock();
        if let Some(cached) = cache.get(&cache_key) {
            #[cfg(dev_release)]
            logger::info(&format!("Cache hit for pattern: {}", ida_pattern));
            return Ok(cached.clone());
        }
    }
    let results = scan_ida_pattern(start, size, ida_pattern)?;
    {
        SCAN_CACHE.lock().insert(cache_key, results.clone());
    }
    Ok(results)
}

/// Clears the scan cache
pub fn clear_cache() {
    SCAN_CACHE.lock().clear();
    #[cfg(dev_release)]
    logger::info("Scan cache cleared");
}

/// Checks if a memory region is readable
fn is_readable_memory(addr: usize, size: usize) -> bool {
    match protection::get_region_info(addr) {
        Ok(info) => {
            let region_end = info.address + info.size;
            if (addr + size) > region_end {
                return false;
            }
            info.protection.is_readable()
        }
        Err(_) => false,
    }
}

/// Retrieves the readable sections of a loaded image
fn get_image_sections(base: usize) -> Result<Vec<(usize, usize)>, ScanError> {
    let mut sections = Vec::new();
    let mut address = base;
    let end_address = address + 0x10000000;

    while address < end_address {
        match protection::find_region(address) {
            Ok(info) => {
                if info.address >= end_address {
                    break;
                }
                if info.protection.is_readable() {
                    sections.push((info.address, info.size));
                }
                let next = info.address + info.size;
                if next <= address {
                    break;
                }
                address = next;
            }
            Err(_) => break,
        }
    }
    if sections.is_empty() {
        return Err(ScanError::InvalidRegion);
    }
    Ok(sections)
}

/// Checks if a pattern matches at a specific address
#[inline]
fn pattern_match(addr: usize, pattern: &[u8], mask: &str) -> bool {
    unsafe {
        let ptr = addr as *const u8;
        for (i, &byte) in pattern.iter().enumerate() {
            if mask.as_bytes()[i] == b'x' && *ptr.add(i) != byte {
                return false;
            }
        }
        true
    }
}
