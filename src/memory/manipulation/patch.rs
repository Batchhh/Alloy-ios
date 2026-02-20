//! Stealth memory patching
//!
//! Uses mach_vm_remap to create writable aliases of code pages,
//! avoiding detectable vm_protect calls on the original code segment.

use crate::memory::ffi::mach_exc::mach_vm_remap;
use crate::memory::platform::thread;
use crate::utils::logger;
use jit_assembler::aarch64::Aarch64InstructionBuilder;
use jit_assembler::common::InstructionBuilder;
use mach2::kern_return::KERN_SUCCESS;
use mach2::traps::mach_task_self;
use mach2::vm::{mach_vm_deallocate, mach_vm_protect};
use mach2::vm_prot::{VM_PROT_COPY, VM_PROT_EXECUTE, VM_PROT_READ, VM_PROT_WRITE};
use std::arch::asm;
use std::ffi::c_void;
use std::ptr;
use thiserror::Error;

const CACHE_LINE_SIZE: usize = 64;

// B instruction range: +/- 128MB
const B_RANGE: isize = 128 * 1024 * 1024;

// Mach VM constants not exposed by mach2
const VM_FLAGS_ANYWHERE: i32 = 0x1;
const VM_INHERIT_NONE: u32 = 2;

#[derive(Error, Debug)]
/// Errors that can occur during patching operations
pub enum PatchError {
    /// The provided hex string is invalid
    #[error("Invalid hex: {0}")]
    InvalidHex(#[from] hex::FromHexError),
    #[error("Image not found: {0}")]
    ImageBaseNotFound(#[from] crate::memory::info::image::ImageError),
    /// Failed to change memory protection
    #[error("Protection failed: {0}")]
    ProtectionFailed(i32),
    /// Thread manipulation error
    #[error("Thread error: {0}")]
    ThreadError(#[from] crate::memory::platform::thread::ThreadError),
    /// The provided instruction list is empty
    #[error("Empty instructions")]
    EmptyInstructions,
    /// Code cave error
    #[error("Code cave error: {0}")]
    CaveError(#[from] crate::memory::info::code_cave::CodeCaveError),
    /// Branch out of range
    #[error("Branch target out of range")]
    BranchOutOfRange,
    /// Write verification failed
    #[error("Write verification failed")]
    VerificationFailed,
}

/// Represents an applied memory patch
pub struct Patch {
    /// The address where the patch was applied
    address: usize,
    /// The original bytes that were overwritten
    original_bytes: Vec<u8>,
    /// Optional code cave used by this patch
    cave: Option<crate::memory::info::code_cave::CodeCave>,
}

impl Patch {
    /// Reverts the patch, restoring the original bytes
    pub fn revert(&self) {
        unsafe {
            let suspended = match thread::suspend_other_threads() {
                Ok(s) => s,
                Err(e) => {
                    #[cfg(dev_release)]
                    logger::error(&format!("Revert suspend failed: {}", e));
                    return;
                }
            };

            if let Err(e) = stealth_write(self.address, &self.original_bytes) {
                #[cfg(dev_release)]
                logger::error(&format!("Revert write failed: {}", e));
            }

            if let Some(cave) = &self.cave {
                if let Err(e) = crate::memory::info::code_cave::free_cave(cave.address) {
                    #[cfg(dev_release)]
                    logger::error(&format!("Cave free failed: {}", e));
                }
            }

            thread::resume_threads(&suspended);
            #[cfg(dev_release)]
            logger::debug("Patch reverted");
        }
    }

    /// Returns the address of the patch
    pub fn address(&self) -> usize {
        self.address
    }

    /// Returns the original bytes
    pub fn original_bytes(&self) -> &[u8] {
        &self.original_bytes
    }
}

/// Applies a hex string patch at a relative virtual address (RVA)
///
/// # Arguments
/// * `rva` - The relative virtual address to patch
/// * `hex_str` - The hex string representing the bytes to write
///
/// # Returns
/// * `Result<Patch, PatchError>` - The applied patch or an error
pub fn apply(rva: usize, hex_str: &str) -> Result<Patch, PatchError> {
    let clean: String = hex_str.chars().filter(|c| !c.is_whitespace()).collect();
    let bytes = hex::decode(&clean)?;
    let base = crate::memory::info::image::get_image_base(crate::config::TARGET_IMAGE_NAME)?;
    let address = base + rva;

    unsafe {
        let suspended = thread::suspend_other_threads()?;

        let original_bytes = read_bytes(address, bytes.len());

        let result = stealth_write(address, &bytes);
        if let Err(e) = result {
            thread::resume_threads(&suspended);
            return Err(e);
        }

        if !verify_write(address, &bytes) {
            thread::resume_threads(&suspended);
            return Err(PatchError::VerificationFailed);
        }

        thread::resume_threads(&suspended);
        #[cfg(dev_release)]
        logger::debug("Patch applied");

        Ok(Patch {
            address,
            original_bytes,
            cave: None,
        })
    }
}

/// Applies an assembly patch at a relative virtual address (RVA)
///
/// # Type Parameters
/// * `F` - The closure that builds the assembly instructions
///
/// # Arguments
/// * `rva` - The relative virtual address to patch
/// * `build` - A closure that takes an `Aarch64InstructionBuilder` and appends instructions
///
/// # Returns
/// * `Result<Patch, PatchError>` - The applied patch or an error
pub fn apply_asm<F>(rva: usize, build: F) -> Result<Patch, PatchError>
where
    F: FnOnce(&mut Aarch64InstructionBuilder) -> &mut Aarch64InstructionBuilder,
{
    let mut builder = Aarch64InstructionBuilder::new();
    build(&mut builder);
    let instructions = builder.instructions();
    if instructions.is_empty() {
        return Err(PatchError::EmptyInstructions);
    }
    let bytes: Vec<u8> = instructions
        .iter()
        .flat_map(|instr| instr.0.to_le_bytes())
        .collect();
    let base = crate::memory::info::image::get_image_base(crate::config::TARGET_IMAGE_NAME)?;
    let address = base + rva;

    unsafe {
        let suspended = thread::suspend_other_threads()?;

        let original_bytes = read_bytes(address, bytes.len());

        let result = stealth_write(address, &bytes);
        if let Err(e) = result {
            thread::resume_threads(&suspended);
            return Err(e);
        }

        if !verify_write(address, &bytes) {
            thread::resume_threads(&suspended);
            return Err(PatchError::VerificationFailed);
        }

        thread::resume_threads(&suspended);
        #[cfg(dev_release)]
        logger::debug("ASM patch applied");

        Ok(Patch {
            address,
            original_bytes,
            cave: None,
        })
    }
}

/// Applies an assembly patch using a code cave
///
/// This writes the assembly instructions to a nearby code cave and patches the
/// target address with a branch to the cave.
///
/// # Type Parameters
/// * `F` - The closure that builds the assembly instructions
///
/// # Arguments
/// * `rva` - The relative virtual address to patch
/// * `build` - A closure that takes an `Aarch64InstructionBuilder` and appends instructions
///
/// # Returns
/// * `Result<Patch, PatchError>` - The applied patch or an error
pub fn apply_asm_in_cave<F>(rva: usize, build: F) -> Result<Patch, PatchError>
where
    F: FnOnce(&mut Aarch64InstructionBuilder) -> &mut Aarch64InstructionBuilder,
{
    let mut builder = Aarch64InstructionBuilder::new();
    build(&mut builder);
    let instructions = builder.instructions();
    if instructions.is_empty() {
        return Err(PatchError::EmptyInstructions);
    }

    let bytes: Vec<u8> = instructions
        .iter()
        .flat_map(|instr| instr.0.to_le_bytes())
        .collect();

    let base = crate::memory::info::image::get_image_base(crate::config::TARGET_IMAGE_NAME)?;
    let address = base + rva;

    let cave = crate::memory::info::code_cave::allocate_cave_near(address, bytes.len())?;

    let aligned_address = (cave.address + 3) & !3;

    if aligned_address + bytes.len() > cave.address + cave.size {
        crate::memory::info::code_cave::free_cave(cave.address).ok();
        return Err(PatchError::CaveError(
            crate::memory::info::code_cave::CodeCaveError::Custom(
                "Cave too small for alignment".to_string(),
            ),
        ));
    }

    let offset = (aligned_address as isize) - (address as isize);
    if !(-B_RANGE..B_RANGE).contains(&offset) {
        crate::memory::info::code_cave::free_cave(cave.address).ok();
        return Err(PatchError::BranchOutOfRange);
    }

    // B instruction: 0x14000000 | imm26
    let b_instr = 0x14000000 | (((offset >> 2) as u32) & 0x03FFFFFF);
    let b_bytes = b_instr.to_le_bytes();

    unsafe {
        let suspended = thread::suspend_other_threads()?;

        // Write instructions to the cave
        if let Err(e) = stealth_write(aligned_address, &bytes) {
            thread::resume_threads(&suspended);
            crate::memory::info::code_cave::free_cave(cave.address).ok();
            return Err(e);
        }

        if !verify_write(aligned_address, &bytes) {
            thread::resume_threads(&suspended);
            crate::memory::info::code_cave::free_cave(cave.address).ok();
            return Err(PatchError::VerificationFailed);
        }

        // Save original bytes and write the branch
        let original_bytes = read_bytes(address, 4);

        if let Err(e) = stealth_write(address, &b_bytes) {
            thread::resume_threads(&suspended);
            crate::memory::info::code_cave::free_cave(cave.address).ok();
            return Err(e);
        }

        if !verify_write(address, &b_bytes) {
            thread::resume_threads(&suspended);
            crate::memory::info::code_cave::free_cave(cave.address).ok();
            return Err(PatchError::VerificationFailed);
        }

        thread::resume_threads(&suspended);
        #[cfg(dev_release)]
        logger::debug("Cave ASM patch applied");

        Ok(Patch {
            address,
            original_bytes,
            cave: Some(cave),
        })
    }
}

/// Applies a hex string patch using a code cave
///
/// This writes the hex bytes to a nearby code cave and patches the
/// target address with a branch to the cave.
///
/// # Arguments
/// * `rva` - The relative virtual address to patch
/// * `hex_str` - The hex string representing the bytes to write
///
/// # Returns
/// * `Result<Patch, PatchError>` - The applied patch or an error
pub fn apply_in_cave(rva: usize, hex_str: &str) -> Result<Patch, PatchError> {
    let clean: String = hex_str.chars().filter(|c| !c.is_whitespace()).collect();
    let bytes = hex::decode(&clean)?;

    if bytes.is_empty() {
        return Err(PatchError::EmptyInstructions);
    }

    let base = crate::memory::info::image::get_image_base(crate::config::TARGET_IMAGE_NAME)?;
    let address = base + rva;

    let cave = crate::memory::info::code_cave::allocate_cave_near(address, bytes.len())?;

    let aligned_address = (cave.address + 3) & !3;

    if aligned_address + bytes.len() > cave.address + cave.size {
        crate::memory::info::code_cave::free_cave(cave.address).ok();
        return Err(PatchError::CaveError(
            crate::memory::info::code_cave::CodeCaveError::Custom(
                "Cave too small for alignment".to_string(),
            ),
        ));
    }

    let offset = (aligned_address as isize) - (address as isize);
    if !(-B_RANGE..B_RANGE).contains(&offset) {
        crate::memory::info::code_cave::free_cave(cave.address).ok();
        return Err(PatchError::BranchOutOfRange);
    }

    // B instruction: 0x14000000 | imm26
    let b_instr = 0x14000000 | (((offset >> 2) as u32) & 0x03FFFFFF);
    let b_bytes = b_instr.to_le_bytes();

    unsafe {
        let suspended = thread::suspend_other_threads()?;

        // Write payload to the cave
        if let Err(e) = stealth_write(aligned_address, &bytes) {
            thread::resume_threads(&suspended);
            crate::memory::info::code_cave::free_cave(cave.address).ok();
            return Err(e);
        }

        if !verify_write(aligned_address, &bytes) {
            thread::resume_threads(&suspended);
            crate::memory::info::code_cave::free_cave(cave.address).ok();
            return Err(PatchError::VerificationFailed);
        }

        // Save original bytes and write the branch
        let original_bytes = read_bytes(address, 4);

        if let Err(e) = stealth_write(address, &b_bytes) {
            thread::resume_threads(&suspended);
            crate::memory::info::code_cave::free_cave(cave.address).ok();
            return Err(e);
        }

        if !verify_write(address, &b_bytes) {
            thread::resume_threads(&suspended);
            crate::memory::info::code_cave::free_cave(cave.address).ok();
            return Err(PatchError::VerificationFailed);
        }

        thread::resume_threads(&suspended);
        #[cfg(dev_release)]
        logger::debug("Cave patch applied");

        Ok(Patch {
            address,
            original_bytes,
            cave: Some(cave),
        })
    }
}

/// Applies a patch at an absolute address
///
/// # Arguments
/// * `address` - The absolute address to patch
/// * `bytes` - The bytes to write
///
/// # Returns
/// * `Result<Patch, PatchError>` - The applied patch or an error
pub fn apply_at_address(address: usize, bytes: &[u8]) -> Result<Patch, PatchError> {
    unsafe {
        let suspended = thread::suspend_other_threads()?;

        let original_bytes = read_bytes(address, bytes.len());

        let result = stealth_write(address, bytes);
        if let Err(e) = result {
            thread::resume_threads(&suspended);
            return Err(e);
        }

        if !verify_write(address, bytes) {
            thread::resume_threads(&suspended);
            return Err(PatchError::VerificationFailed);
        }

        thread::resume_threads(&suspended);
        #[cfg(dev_release)]
        logger::debug("Address patch applied");

        Ok(Patch {
            address,
            original_bytes,
            cave: None,
        })
    }
}

unsafe fn read_bytes(address: usize, len: usize) -> Vec<u8> {
    (0..len)
        .map(|i| super::rw::read::<u8>(address + i).unwrap_or(0))
        .collect()
}

/// Writes to code memory using mach_vm_remap to create a writable alias,
/// avoiding detectable vm_protect calls on the original code pages.
/// Falls back to traditional vm_protect if remap is unavailable.
pub(crate) unsafe fn stealth_write(address: usize, data: &[u8]) -> Result<(), PatchError> {
    let page_size = libc::sysconf(libc::_SC_PAGESIZE) as usize;
    let page_mask = !(page_size - 1);
    let page_start = address & page_mask;
    let page_len = ((address + data.len() + page_size - 1) & page_mask) - page_start;
    let offset_in_page = address - page_start;

    let task = mach_task_self();
    let mut remap_addr: u64 = 0;
    let mut cur_prot: i32 = 0;
    let mut max_prot: i32 = 0;

    let kr = mach_vm_remap(
        task,
        &mut remap_addr,
        page_len as u64,
        0,
        VM_FLAGS_ANYWHERE,
        task,
        page_start as u64,
        0, // share, not copy
        &mut cur_prot,
        &mut max_prot,
        VM_INHERIT_NONE,
    );

    if kr != KERN_SUCCESS {
        #[cfg(dev_release)]
        logger::debug("Remap unavailable, fallback");
        return fallback_write(address, data);
    }

    // Check if the remap's max protection allows writing
    if (max_prot & VM_PROT_WRITE) == 0 {
        mach_vm_deallocate(task, remap_addr, page_len as u64);
        #[cfg(dev_release)]
        logger::debug("Remap max prot insufficient, fallback");
        return fallback_write(address, data);
    }

    // Make the REMAP writable (not the original code pages)
    let kr = mach_vm_protect(
        task,
        remap_addr,
        page_len as u64,
        0,
        VM_PROT_READ | VM_PROT_WRITE,
    );

    if kr != KERN_SUCCESS {
        mach_vm_deallocate(task, remap_addr, page_len as u64);
        #[cfg(dev_release)]
        logger::debug("Remap protect failed, fallback");
        return fallback_write(address, data);
    }

    // Write through the writable alias
    let write_addr = remap_addr as usize + offset_in_page;
    ptr::copy_nonoverlapping(data.as_ptr(), write_addr as *mut u8, data.len());

    // Tear down the alias
    mach_vm_deallocate(task, remap_addr, page_len as u64);

    // Flush caches on the ORIGINAL address
    invalidate_icache(address as *mut c_void, data.len());

    Ok(())
}

/// Fallback write using traditional vm_protect (if remap is unavailable)
unsafe fn fallback_write(address: usize, data: &[u8]) -> Result<(), PatchError> {
    use crate::memory::info::protection;

    let page_size = libc::sysconf(libc::_SC_PAGESIZE) as usize;
    let page_mask = !(page_size - 1);
    let page_start = address & page_mask;
    let page_len = ((address + data.len() + page_size - 1) & page_mask) - page_start;

    let original_prot = protection::get_protection(address)
        .map(|p| p.raw())
        .unwrap_or(VM_PROT_READ | VM_PROT_EXECUTE);

    protection::protect(
        page_start,
        page_len,
        protection::PageProtection::from_raw(VM_PROT_READ | VM_PROT_WRITE | VM_PROT_COPY),
    )
    .map_err(|e| match e {
        protection::ProtectionError::ProtectionFailed(k) => PatchError::ProtectionFailed(k),
        _ => PatchError::ProtectionFailed(0),
    })?;

    ptr::copy_nonoverlapping(data.as_ptr(), address as *mut u8, data.len());

    let _ = protection::protect(
        page_start,
        page_len,
        protection::PageProtection::from_raw(original_prot),
    );

    invalidate_icache(address as *mut c_void, data.len());

    Ok(())
}

/// Verifies that written bytes match expected data
unsafe fn verify_write(address: usize, expected: &[u8]) -> bool {
    for (i, &byte) in expected.iter().enumerate() {
        if super::rw::read::<u8>(address + i).unwrap_or(!byte) != byte {
            return false;
        }
    }
    true
}

/// Invalidates the instruction cache for a memory range
///
/// Uses the full ARM64 cache maintenance sequence:
/// dc cvau -> dsb ish -> ic ivau -> dsb ish -> isb
#[inline]
pub unsafe fn invalidate_icache(start: *mut c_void, len: usize) {
    let start_addr = start as usize;
    let end_addr = start_addr + len;
    let mut addr = start_addr & !(CACHE_LINE_SIZE - 1);

    // Clean data cache to Point of Unification
    while addr < end_addr {
        asm!("dc cvau, {x}", x = in(reg) addr, options(nostack, preserves_flags));
        addr += CACHE_LINE_SIZE;
    }
    asm!("dsb ish", options(nostack, preserves_flags));

    // Invalidate instruction cache
    addr = start_addr & !(CACHE_LINE_SIZE - 1);
    while addr < end_addr {
        asm!("ic ivau, {x}", x = in(reg) addr, options(nostack, preserves_flags));
        addr += CACHE_LINE_SIZE;
    }
    asm!("dsb ish", options(nostack, preserves_flags));
    asm!("isb", options(nostack, preserves_flags));
}
