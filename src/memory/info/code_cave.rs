//! Code cave finder and manager

use crate::memory::info::{image, scan};
use crate::utils::logger;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;
use thiserror::Error;

/// ARM64 NOP instruction encoding
const ARM64_NOP: u32 = 0x1F2003D5;

#[derive(Error, Debug)]
/// Errors that can occur during code cave operations
pub enum CodeCaveError {
    /// No suitable cave was found for the requested size
    #[error("No suitable cave found")]
    NoCaveFound,
    /// The specified cave is already allocated
    #[error("Cave already allocated at {0:#x}")]
    AlreadyAllocated(usize),
    /// The specified cave was not found in the registry
    #[error("Cave not found at {0:#x}")]
    NotFound(usize),
    /// The requested size is invalid (e.g., 0)
    #[error("Invalid size: {0}")]
    InvalidSize(usize),
    /// Image lookup failed
    #[error("Image not found: {0}")]
    ImageNotFound(#[from] image::ImageError),
    /// Memory scan failed
    #[error("Scan error: {0}")]
    ScanError(#[from] scan::ScanError),
    /// Custom error message
    #[error("{0}")]
    Custom(String),
}

/// Represents a code cave (unused memory region)
#[derive(Debug, Clone)]
pub struct CodeCave {
    /// The start address of the cave
    pub address: usize,
    /// The size of the cave in bytes
    pub size: usize,
    /// Whether the cave is currently in use
    pub allocated: bool,
    /// Optional description of what the cave is used for or how it was found
    pub description: Option<String>,
}

impl CodeCave {
    /// Creates a new code cave
    ///
    /// # Arguments
    /// * `address` - The start address
    /// * `size` - The size in bytes
    pub fn new(address: usize, size: usize) -> Self {
        Self {
            address,
            size,
            allocated: false,
            description: None,
        }
    }

    /// Creates a new code cave with a description
    ///
    /// # Arguments
    /// * `address` - The start address
    /// * `size` - The size in bytes
    /// * `description` - A description of the cave
    pub fn with_description(address: usize, size: usize, description: String) -> Self {
        Self {
            address,
            size,
            allocated: false,
            description: Some(description),
        }
    }

    /// Marks the cave as allocated
    pub fn allocate(&mut self) {
        self.allocated = true;
    }

    /// Marks the cave as free
    pub fn free(&mut self) {
        self.allocated = false;
    }
}

/// Registry to track allocated code caves
struct CaveRegistry {
    caves: HashMap<usize, CodeCave>,
}

impl CaveRegistry {
    fn new() -> Self {
        Self {
            caves: HashMap::new(),
        }
    }

    fn register(&mut self, cave: CodeCave) -> Result<(), CodeCaveError> {
        if self.caves.contains_key(&cave.address) {
            return Err(CodeCaveError::AlreadyAllocated(cave.address));
        }
        self.caves.insert(cave.address, cave);
        Ok(())
    }

    fn unregister(&mut self, address: usize) -> Result<CodeCave, CodeCaveError> {
        self.caves
            .remove(&address)
            .ok_or(CodeCaveError::NotFound(address))
    }

    fn get(&self, address: usize) -> Option<&CodeCave> {
        self.caves.get(&address)
    }

    fn list_all(&self) -> Vec<CodeCave> {
        self.caves.values().cloned().collect()
    }

    fn clear(&mut self) {
        self.caves.clear();
    }
}

static REGISTRY: Lazy<Mutex<CaveRegistry>> = Lazy::new(|| Mutex::new(CaveRegistry::new()));

/// Finds sequences of NOP instructions in a memory range
///
/// # Arguments
/// * `start` - The start address of the range
/// * `size` - The size of the range
/// * `min_count` - The minimum number of NOP instructions to consider a cave
///
/// # Returns
/// * `Result<Vec<CodeCave>, CodeCaveError>` - A list of found caves or an error
pub fn find_nop_sequences(
    start: usize,
    size: usize,
    min_count: usize,
) -> Result<Vec<CodeCave>, CodeCaveError> {
    if min_count == 0 {
        return Err(CodeCaveError::InvalidSize(min_count));
    }

    let min_size = min_count * 4;
    let mut caves = Vec::new();

    unsafe {
        let mut current_addr = start;
        let end_addr = start + size;

        while current_addr < end_addr {
            if let Ok(instr) = crate::memory::rw::read::<u32>(current_addr) {
                if instr == ARM64_NOP {
                    let cave_start = current_addr;
                    let mut cave_size = 0;
                    let mut temp_addr = current_addr;

                    while temp_addr < end_addr {
                        if let Ok(i) = crate::memory::rw::read::<u32>(temp_addr) {
                            if i == ARM64_NOP {
                                cave_size += 4;
                                temp_addr += 4;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    if cave_size >= min_size {
                        caves.push(CodeCave::with_description(
                            cave_start,
                            cave_size,
                            format!("NOP sequence ({} instructions)", cave_size / 4),
                        ));
                    }

                    current_addr = temp_addr;
                } else {
                    current_addr += 4;
                }
            } else {
                current_addr += 4;
            }
        }
    }

    Ok(caves)
}

/// Finds alignment padding (zero bytes) in a memory range
///
/// # Arguments
/// * `start` - The start address of the range
/// * `size` - The size of the range
///
/// # Returns
/// * `Result<Vec<CodeCave>, CodeCaveError>` - A list of found caves or an error
pub fn find_alignment_padding(start: usize, size: usize) -> Result<Vec<CodeCave>, CodeCaveError> {
    let mut caves = Vec::new();

    unsafe {
        let mut current_addr = start;
        let end_addr = start + size;

        while current_addr < end_addr {
            if let Ok(byte) = crate::memory::rw::read::<u8>(current_addr) {
                if byte == 0x00 {
                    let cave_start = current_addr;
                    let mut cave_size = 0;
                    let mut temp_addr = current_addr;

                    while temp_addr < end_addr {
                        if let Ok(b) = crate::memory::rw::read::<u8>(temp_addr) {
                            if b == 0x00 {
                                cave_size += 1;
                                temp_addr += 1;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    if cave_size >= 16 {
                        caves.push(CodeCave::with_description(
                            cave_start,
                            cave_size,
                            format!("Padding ({} bytes)", cave_size),
                        ));
                    }

                    current_addr = temp_addr;
                } else {
                    current_addr += 1;
                }
            } else {
                current_addr += 1;
            }
        }
    }

    Ok(caves)
}

/// Finds all code caves (NOPs and padding) in a memory range
///
/// # Arguments
/// * `start` - The start address of the range
/// * `size` - The size of the range
/// * `min_size` - The minimum size in bytes
///
/// # Returns
/// * `Result<Vec<CodeCave>, CodeCaveError>` - A list of found caves or an error
pub fn find_caves(
    start: usize,
    size: usize,
    min_size: usize,
) -> Result<Vec<CodeCave>, CodeCaveError> {
    let mut all_caves = Vec::new();

    let min_nops = (min_size + 3) / 4;
    if let Ok(nop_caves) = find_nop_sequences(start, size, min_nops) {
        all_caves.extend(nop_caves);
    }

    if let Ok(padding_caves) = find_alignment_padding(start, size) {
        all_caves.extend(padding_caves.into_iter().filter(|c| c.size >= min_size));
    }

    all_caves.sort_by_key(|c| c.address);

    Ok(all_caves)
}

/// Finds code caves in an entire image
///
/// # Arguments
/// * `image_name` - The name of the image to scan
/// * `min_size` - The minimum size in bytes
///
/// # Returns
/// * `Result<Vec<CodeCave>, CodeCaveError>` - A list of found caves or an error
pub fn find_caves_in_image(
    image_name: &str,
    min_size: usize,
) -> Result<Vec<CodeCave>, CodeCaveError> {
    let base = image::get_image_base(image_name)?;

    let scan_size = 32 * 1024 * 1024; // 32MB

    #[cfg(dev_release)]
    logger::info(&format!(
        "Scanning image '{}' at {:#x} for code caves (min size: {} bytes)",
        image_name, base, min_size
    ));

    find_caves(base, scan_size, min_size)
}

/// Allocates a code cave of the requested size
///
/// This function scans the target image for a suitable cave, marks it as allocated, and returns it.
///
/// # Arguments
/// * `size` - The required size in bytes
///
/// # Returns
/// * `Result<CodeCave, CodeCaveError>` - The allocated cave or an error
pub fn allocate_cave(size: usize) -> Result<CodeCave, CodeCaveError> {
    if size == 0 {
        return Err(CodeCaveError::InvalidSize(size));
    }

    let caves = find_caves_in_image(crate::config::TARGET_IMAGE_NAME, size)?;

    for mut cave in caves {
        let registry = REGISTRY.lock();
        if registry.get(cave.address).is_some() {
            continue;
        }
        drop(registry);

        if cave.size >= size {
            cave.allocate();
            REGISTRY.lock().register(cave.clone())?;
            #[cfg(dev_release)]
            logger::info(&format!(
                "Allocated code cave at {:#x} (size: {} bytes)",
                cave.address, cave.size
            ));
            return Ok(cave);
        }
    }

    Err(CodeCaveError::NoCaveFound)
}

/// Allocates a code cave near a target address (within branch range)
///
/// Useful for creating trampolines that need to be within ±128MB of the target.
///
/// # Arguments
/// * `target` - The target address
/// * `size` - The required size in bytes
///
/// # Returns
/// * `Result<CodeCave, CodeCaveError>` - The allocated cave or an error
pub fn allocate_cave_near(target: usize, size: usize) -> Result<CodeCave, CodeCaveError> {
    if size == 0 {
        return Err(CodeCaveError::InvalidSize(size));
    }

    const BRANCH_RANGE: usize = 128 * 1024 * 1024; // ±128MB for ARM64 B instruction

    let caves = find_caves_in_image(crate::config::TARGET_IMAGE_NAME, size)?;

    for mut cave in caves {
        let distance = if cave.address > target {
            cave.address - target
        } else {
            target - cave.address
        };

        if distance <= BRANCH_RANGE && cave.size >= size {
            let registry = REGISTRY.lock();
            if registry.get(cave.address).is_some() {
                continue;
            }
            drop(registry);

            cave.allocate();
            REGISTRY.lock().register(cave.clone())?;
            #[cfg(dev_release)]
            logger::info(&format!(
                "Allocated code cave near {:#x} at {:#x} (size: {} bytes)",
                target, cave.address, cave.size
            ));
            return Ok(cave);
        }
    }

    Err(CodeCaveError::NoCaveFound)
}

/// Frees a previously allocated code cave
///
/// # Arguments
/// * `address` - The address of the cave to free
///
/// # Returns
/// * `Result<(), CodeCaveError>` - Result indicating success or failure
pub fn free_cave(address: usize) -> Result<(), CodeCaveError> {
    let mut cave = REGISTRY.lock().unregister(address)?;
    cave.free();
    #[cfg(dev_release)]
    logger::info(&format!("Freed code cave at {:#x}", address));
    Ok(())
}

/// Checks if a cave at the given address is available (not modified)
///
/// # Arguments
/// * `address` - The address to check
/// * `size` - The size to check
///
/// # Returns
/// * `bool` - `true` if the memory contains valid cave content (NOPs or zeros)
pub fn is_cave_available(address: usize, size: usize) -> bool {
    let registry = REGISTRY.lock();

    if registry.get(address).is_some() {
        return false;
    }

    unsafe {
        for offset in (0..size).step_by(4) {
            if let Ok(instr) = crate::memory::rw::read::<u32>(address + offset) {
                if instr != ARM64_NOP && instr != 0 {
                    return false;
                }
            } else {
                return false;
            }
        }
    }

    true
}

/// Lists all currently allocated code caves
///
/// # Returns
/// * `Vec<CodeCave>` - A list of allocated caves
pub fn list_allocated_caves() -> Vec<CodeCave> {
    REGISTRY.lock().list_all()
}

/// Returns statistics about code cave usage
///
/// # Returns
/// * `(usize, usize)` - A tuple containing (count, total_size_in_bytes)
pub fn get_cave_stats() -> (usize, usize) {
    let registry = REGISTRY.lock();
    let caves = registry.list_all();
    let total_size: usize = caves.iter().map(|c| c.size).sum();
    (caves.len(), total_size)
}

/// Clears all allocated code caves from the registry
pub fn clear_all_caves() {
    REGISTRY.lock().clear();
    #[cfg(dev_release)]
    logger::info("Cleared all allocated code caves");
}
