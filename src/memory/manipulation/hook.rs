//! # ARM64 Inline Hooking
//!
//! This module implements a robust inline hooking engine for ARM64 architecture.
//! It supports:
//! - Standard trampoline-based hooking
//! - Code cave-based hooking (for stealth)
//! - Call tracing and original function invocation
//! - Automatic instruction relocation (fixing PC-relative instructions)
//! - Thread safety during hook installation/removal
//! - Instruction obfuscation (junk instructions, opaque predicates)
//! - Self-checksumming for tamper detection

use super::checksum;
use crate::config;
use crate::memory::info::{code_cave, symbol};
use crate::memory::{image, patch, protection, thread};
use crate::utils::logger;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::ffi::c_void;
use std::ptr;
use thiserror::Error;

const PAGE_SIZE: usize = 0x4000;
const TRAMPOLINE_SIZE: usize = 4096;
const MAX_STOLEN_BYTES: usize = 16;
const B_RANGE: isize = 128 * 1024 * 1024;

/// Caller-saved registers safe to clobber at function entry
const SAFE_REGS: [u32; 9] = [9, 10, 11, 12, 13, 14, 15, 16, 17];

/// Maximum junk instructions to insert (randomized 1-4)
const MAX_JUNK_INSTRS: usize = 4;

/// Internal entry for a registered hook
struct HookEntry {
    /// Target address of the hook
    target: usize,
    /// Original bytes stolen from the target
    original: Vec<u8>,
    /// Address of the allocated trampoline
    trampoline: usize,
    /// Size of the stolen bytes (usually 16 or 4 bytes)
    stolen_size: usize,
}

static REGISTRY: Lazy<Mutex<HashMap<usize, HookEntry>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Error, Debug)]
/// Errors that can occur during hook operations
pub enum HookError {
    /// A hook already exists at the specified address
    #[error("Hook exists: {0:#x}")]
    AlreadyExists(usize),
    /// Failed to find the target image base
    #[error("Image not found: {0}")]
    ImageBaseNotFound(#[from] crate::memory::info::image::ImageError),
    /// Failed to allocate memory for the trampoline
    #[error("Alloc failed")]
    AllocationFailed,
    /// Failed to change memory protection
    #[error("Protection failed: {0}")]
    ProtectionFailed(i32),
    /// Failed to patch the target memory
    #[error("Patch failed")]
    PatchFailed,
    /// Failed to relocate instructions
    #[error("Relocation failed")]
    RelocationFailed,
    /// Thread manipulation error
    #[error("Thread error: {0}")]
    ThreadError(#[from] crate::memory::platform::thread::ThreadError),
    /// Failed to resolve symbol
    #[error("Symbol error: {0}")]
    SymbolError(#[from] crate::memory::info::symbol::SymbolError),
}

/// Represents an installed hook
pub struct Hook {
    /// The address where the hook was installed
    target: usize,
    /// The address of the trampoline (original function wrapper)
    trampoline: usize,
}

impl Hook {
    #[inline]
    /// Returns the address of the trampoline
    ///
    /// # Returns
    /// * `usize` - The memory address of the trampoline
    #[inline]
    pub fn trampoline(&self) -> usize {
        self.trampoline
    }
    #[inline]
    /// Returns the trampoline as a function pointer of the specified type
    ///
    /// # Type Parameters
    /// * `F` - The function pointer type
    ///
    /// # Returns
    /// * `F` - The trampoline cast to the function pointer type
    pub unsafe fn trampoline_as<F>(&self) -> F
    where
        F: Copy,
    {
        std::mem::transmute_copy(&self.trampoline)
    }
    /// Removes the hook, restoring the original code
    pub fn remove(self) {
        unsafe {
            remove_at_address(self.target);
        }
    }

    #[inline]
    /// Calls the original function (via trampoline)
    ///
    /// # Type Parameters
    /// * `T` - The function pointer type for the trampoline
    /// * `F` - The callback closure type
    /// * `R` - The return type
    ///
    /// # Arguments
    /// * `callback` - A closure that takes the trampoline and returns a result
    ///
    /// # Returns
    /// * `R` - The result of the callback
    #[inline]
    pub unsafe fn call_original<T, F, R>(&self, callback: F) -> R
    where
        T: Copy,
        F: FnOnce(T) -> R,
    {
        let orig: T = std::mem::transmute_copy(&self.trampoline);
        callback(orig)
    }

    /// Verifies that the hook has not been tampered with
    ///
    /// # Returns
    /// * `bool` - `true` if the hook is intact, `false` if it has been modified
    #[inline]
    pub fn verify_integrity(&self) -> bool {
        checksum::verify(self.target).unwrap_or(false)
    }

    /// Returns the target address where the hook is installed
    #[inline]
    pub fn target(&self) -> usize {
        self.target
    }
}

/// Installs an inline hook at a relative virtual address (RVA)
///
/// # Arguments
/// * `rva` - The relative virtual address to hook
/// * `replacement` - The address of the replacement function
///
/// # Returns
/// * `Result<Hook, HookError>` - The installed hook or an error
pub unsafe fn install(rva: usize, replacement: usize) -> Result<Hook, HookError> {
    let base = image::get_image_base(config::TARGET_IMAGE_NAME)?;
    let target = base + rva;
    let trampoline = install_at_address(target, replacement)?;
    Ok(Hook { target, trampoline })
}

/// Installs an inline hook on a symbol resolved by name
///
/// # Arguments
/// * `symbol_name` - The name of the symbol to hook (e.g., "_objc_msgSend")
/// * `replacement` - The address of the replacement function
///
/// # Returns
/// * `Result<Hook, HookError>` - The installed hook or an error
pub unsafe fn hook_symbol(symbol_name: &str, replacement: usize) -> Result<Hook, HookError> {
    let target = symbol::resolve_symbol(symbol_name)?;
    let trampoline = install_at_address(target, replacement)?;
    Ok(Hook { target, trampoline })
}

/// Installs an inline hook at an absolute address
///
/// # Arguments
/// * `target` - The absolute address to hook
/// * `replacement` - The address of the replacement function
///
/// # Returns
/// * `Result<usize, HookError>` - The address of the trampoline or an error
pub unsafe fn install_at_address(target: usize, replacement: usize) -> Result<usize, HookError> {
    if REGISTRY.lock().contains_key(&target) {
        #[cfg(dev_release)]
        logger::warning("Hook already exists at target");
        return Err(HookError::AlreadyExists(target));
    }

    let first_instr = super::rw::read::<u32>(target).map_err(|_| HookError::PatchFailed)?;
    if is_b_instruction(first_instr) {
        #[cfg(dev_release)]
        logger::debug("Detected thunk, using short hook");
        return install_thunk_hook(target, replacement, first_instr);
    }
    install_regular_hook(target, replacement)
}

/// Checks if instruction is a generic branch
fn is_b_instruction(instr: u32) -> bool {
    (instr >> 26) & 0x3F == 0x05
}

/// Decodes destination of a branch instruction
fn decode_b_target(instr: u32, pc: usize) -> usize {
    let imm26 = instr & 0x03FFFFFF;
    let mut offset = (imm26 << 2) as i32;
    if (offset & (1 << 27)) != 0 {
        offset |= !0x0FFFFFFF_u32 as i32;
    }
    (pc as isize).wrapping_add(offset as isize) as usize
}

/// Encodes a branch instruction to target
fn encode_b_instruction(from: usize, to: usize) -> Option<u32> {
    let offset = (to as isize) - (from as isize);
    if !(-B_RANGE..B_RANGE).contains(&offset) {
        return None;
    }
    Some(0x14000000 | (((offset >> 2) as u32) & 0x03FFFFFF))
}

/// Generates a polymorphic 16-byte absolute branch to `dest`.
/// Randomly selects between literal-pool and immediate-move encoding,
/// and randomly rotates the scratch register per call.
fn gen_branch_bytes(dest: usize) -> [u8; 16] {
    let rand = unsafe { libc::arc4random() };
    let reg = SAFE_REGS[(rand as usize) % SAFE_REGS.len()];
    if (rand >> 16) & 1 == 0 {
        gen_variant_ldr(dest, reg)
    } else {
        gen_variant_mov(dest, reg)
    }
}

/// Variant A: LDR Xn, #8 ; BR Xn ; .quad dest (literal pool)
fn gen_variant_ldr(dest: usize, reg: u32) -> [u8; 16] {
    let mut buf = [0u8; 16];
    let ldr = 0x58000040u32 | reg; // LDR Xn, #8
    let br = 0xD61F0000u32 | (reg << 5); // BR Xn
    buf[0..4].copy_from_slice(&ldr.to_le_bytes());
    buf[4..8].copy_from_slice(&br.to_le_bytes());
    buf[8..16].copy_from_slice(&dest.to_le_bytes());
    buf
}

/// Variant B: MOVZ/MOVK Xn, #imm16 (x3) ; BR Xn (immediate moves, no embedded pointer)
fn gen_variant_mov(dest: usize, reg: u32) -> [u8; 16] {
    let mut buf = [0u8; 16];
    let d = dest as u64;
    let movz = 0xD2800000u32 | (((d & 0xFFFF) as u32) << 5) | reg;
    let movk16 = 0xF2A00000u32 | ((((d >> 16) & 0xFFFF) as u32) << 5) | reg;
    let movk32 = 0xF2C00000u32 | ((((d >> 32) & 0xFFFF) as u32) << 5) | reg;
    let br = 0xD61F0000u32 | (reg << 5);
    buf[0..4].copy_from_slice(&movz.to_le_bytes());
    buf[4..8].copy_from_slice(&movk16.to_le_bytes());
    buf[8..12].copy_from_slice(&movk32.to_le_bytes());
    buf[12..16].copy_from_slice(&br.to_le_bytes());
    buf
}

/// Generates a random junk instruction that has no semantic effect
/// Uses caller-saved registers to avoid corrupting state
fn gen_junk_instruction() -> u32 {
    let rand = unsafe { libc::arc4random() };
    let reg = SAFE_REGS[(rand as usize) % SAFE_REGS.len()];

    match (rand >> 8) % 6 {
        // NOP
        0 => 0xD503201F,
        // MOV Xn, Xn (self-move via ORR)
        1 => 0xAA0003E0 | reg | (reg << 16),
        // ADD Xn, Xn, #0
        2 => 0x91000000 | reg | (reg << 5),
        // SUB Xn, Xn, #0
        3 => 0xD1000000 | reg | (reg << 5),
        // EOR Xn, Xn, Xn (XOR with self = 0, then we don't use result)
        // Using AND Xn, Xn, Xn instead (preserves value)
        4 => 0x8A000000 | reg | (reg << 5) | (reg << 16),
        // ORR Xn, Xn, Xn (preserves value)
        _ => 0xAA000000 | reg | (reg << 5) | (reg << 16),
    }
}

/// Generates 1-4 random junk instructions
/// Returns the bytes and the count of instructions generated
fn gen_junk_sled() -> (Vec<u8>, usize) {
    let rand = unsafe { libc::arc4random() };
    let count = 1 + ((rand as usize) % MAX_JUNK_INSTRS);
    let mut bytes = Vec::with_capacity(count * 4);

    for _ in 0..count {
        bytes.extend_from_slice(&gen_junk_instruction().to_le_bytes());
    }

    (bytes, count)
}

/// Generates an opaque predicate that always evaluates to true
/// Format: CMP XZR, XZR ; B.NE +8 (skip next instruction)
/// The condition is never taken, so execution falls through
fn gen_opaque_always_true() -> [u8; 8] {
    let mut buf = [0u8; 8];
    // CMP XZR, XZR (SUBS XZR, XZR, XZR) - always sets Z flag
    let cmp = 0xEB1F03FF_u32;
    // B.NE +8 (skip 2 instructions) - never taken since Z is set
    let bne = 0x54000041_u32;
    buf[0..4].copy_from_slice(&cmp.to_le_bytes());
    buf[4..8].copy_from_slice(&bne.to_le_bytes());
    buf
}

/// Generates an opaque predicate that always evaluates to false
/// Format: CMP XZR, XZR ; B.EQ +8 (always jumps forward)
fn gen_opaque_always_false() -> [u8; 8] {
    let mut buf = [0u8; 8];
    // CMP XZR, XZR - always sets Z flag
    let cmp = 0xEB1F03FF_u32;
    // B.EQ +8 (skip 2 instructions) - always taken since Z is set
    let beq = 0x54000040_u32;
    buf[0..4].copy_from_slice(&cmp.to_le_bytes());
    buf[4..8].copy_from_slice(&beq.to_le_bytes());
    buf
}

/// Generates an obfuscated branch sequence with junk instructions and opaque predicates
/// Returns the full byte sequence for the obfuscated branch
///
/// Structure:
/// 1. Random junk sled (1-4 NOPs/self-moves)
/// 2. Optional opaque predicate (50% chance)
/// 3. Actual branch instruction sequence
fn gen_obfuscated_branch(dest: usize) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(64);
    let rand = unsafe { libc::arc4random() };

    let (junk, _) = gen_junk_sled();
    bytes.extend_from_slice(&junk);

    if (rand >> 16) & 1 == 1 {
        bytes.extend_from_slice(&gen_opaque_always_true());
    }

    let branch = gen_branch_bytes(dest);
    bytes.extend_from_slice(&branch);

    bytes
}

/// Writes an obfuscated absolute branch sequence to writable memory (trampolines)
/// Returns the number of bytes written
#[inline]
unsafe fn emit_obfuscated_branch(addr: usize, dest: usize) -> usize {
    let bytes = gen_obfuscated_branch(dest);
    let len = bytes.len();
    ptr::copy_nonoverlapping(bytes.as_ptr(), addr as *mut u8, len);
    len
}

/// Installs a hook using existing branch/thunk
unsafe fn install_thunk_hook(
    target: usize,
    replacement: usize,
    first_instr: u32,
) -> Result<usize, HookError> {
    let suspended = thread::suspend_other_threads()?;
    let original_target = decode_b_target(first_instr, target);
    let trampoline = alloc_trampoline_near(target).ok_or(HookError::AllocationFailed)?;
    let trampoline_base = trampoline as usize;

    if encode_b_instruction(target, trampoline_base).is_none() {
        libc::munmap(trampoline, TRAMPOLINE_SIZE);
        thread::resume_threads(&suspended);
        return install_regular_hook(target, replacement);
    }

    let can_direct = encode_b_instruction(target, replacement).is_some();
    emit_branch(trampoline_base, original_target);
    if !can_direct {
        emit_branch(trampoline_base + 16, replacement);
    }

    if protection::protect(
        trampoline_base,
        TRAMPOLINE_SIZE,
        protection::PageProtection::read_execute(),
    )
    .is_err()
    {
        libc::munmap(trampoline, TRAMPOLINE_SIZE);
        thread::resume_threads(&suspended);
        return Err(HookError::ProtectionFailed(0));
    }
    patch::invalidate_icache(trampoline, TRAMPOLINE_SIZE);

    let mut original = vec![0u8; 4];
    ptr::copy_nonoverlapping(target as *const u8, original.as_mut_ptr(), 4);

    let b_instr = if can_direct {
        encode_b_instruction(target, replacement).unwrap()
    } else {
        encode_b_instruction(target, trampoline_base + 16).unwrap()
    };

    if !patch_short(target, b_instr) {
        libc::munmap(trampoline, TRAMPOLINE_SIZE);
        thread::resume_threads(&suspended);
        return Err(HookError::PatchFailed);
    }

    REGISTRY.lock().insert(
        target,
        HookEntry {
            target,
            original,
            trampoline: trampoline_base,
            stolen_size: 4,
        },
    );
    checksum::register(target, 4);
    thread::resume_threads(&suspended);
    #[cfg(dev_release)]
    logger::debug("Thunk hook installed");
    Ok(trampoline_base)
}

/// Installs a standard inline hook with trampoline
unsafe fn install_regular_hook(target: usize, replacement: usize) -> Result<usize, HookError> {
    let suspended = thread::suspend_other_threads()?;
    let trampoline = alloc_trampoline().ok_or(HookError::AllocationFailed)?;
    if trampoline.is_null() {
        thread::resume_threads(&suspended);
        return Err(HookError::AllocationFailed);
    }
    let trampoline_base = trampoline as usize;

    let mut original = vec![0u8; MAX_STOLEN_BYTES];
    ptr::copy_nonoverlapping(target as *const u8, original.as_mut_ptr(), MAX_STOLEN_BYTES);

    let mut trampoline_offset = 0;
    for i in 0..4 {
        let instr = super::rw::read::<u32>(target + i * 4).unwrap_or(0);
        if let Some(size) =
            relocate_instruction(instr, target + i * 4, trampoline_base + trampoline_offset)
        {
            trampoline_offset += size;
        } else {
            libc::munmap(trampoline, TRAMPOLINE_SIZE);
            thread::resume_threads(&suspended);
            return Err(HookError::RelocationFailed);
        }
    }

    emit_branch(
        trampoline_base + trampoline_offset,
        target + MAX_STOLEN_BYTES,
    );

    if protection::protect(
        trampoline_base,
        TRAMPOLINE_SIZE,
        protection::PageProtection::read_execute(),
    )
    .is_err()
    {
        libc::munmap(trampoline, TRAMPOLINE_SIZE);
        thread::resume_threads(&suspended);
        return Err(HookError::ProtectionFailed(0));
    }
    patch::invalidate_icache(trampoline, TRAMPOLINE_SIZE);

    if !patch_code(target, replacement) {
        libc::munmap(trampoline, TRAMPOLINE_SIZE);
        thread::resume_threads(&suspended);
        return Err(HookError::PatchFailed);
    }

    REGISTRY.lock().insert(
        target,
        HookEntry {
            target,
            original,
            trampoline: trampoline_base,
            stolen_size: MAX_STOLEN_BYTES,
        },
    );
    checksum::register(target, MAX_STOLEN_BYTES);
    thread::resume_threads(&suspended);
    #[cfg(dev_release)]
    logger::debug("Hook installed");
    Ok(trampoline_base)
}

/// Removes a hook at a relative virtual address (RVA)
///
/// # Arguments
/// * `rva` - The relative virtual address where the hook is installed
///
/// # Returns
/// * `bool` - `true` if the hook was successfully removed, `false` otherwise
pub unsafe fn remove(rva: usize) -> bool {
    match image::get_image_base(config::TARGET_IMAGE_NAME) {
        Ok(base) => remove_at_address(base + rva),
        Err(_) => false,
    }
}

/// Removes a hook at an absolute address
///
/// # Arguments
/// * `target` - The absolute address where the hook is installed
///
/// # Returns
/// * `bool` - `true` if the hook was successfully removed, `false` otherwise
pub unsafe fn remove_at_address(target: usize) -> bool {
    let entry = match REGISTRY.lock().remove(&target) {
        Some(e) => e,
        None => return false,
    };

    let suspended = match thread::suspend_other_threads() {
        Ok(s) => s,
        Err(_) => return false,
    };

    let _ = patch::stealth_write(entry.target, &entry.original[..entry.stolen_size]);

    libc::munmap(entry.trampoline as *mut c_void, TRAMPOLINE_SIZE);
    checksum::unregister(target);
    thread::resume_threads(&suspended);
    #[cfg(dev_release)]
    logger::debug("Hook removed");
    true
}

/// Restores the hook redirect bytes at the target address
///
/// This is called by the integrity monitor when tampering is detected.
/// It re-writes the hook redirect to restore functionality.
///
/// # Arguments
/// * `target` - The hook target address
///
/// # Returns
/// * `bool` - `true` if restored successfully, `false` otherwise
pub fn restore_hook_bytes(target: usize) -> bool {
    let registry = REGISTRY.lock();
    let entry = match registry.get(&target) {
        Some(e) => e,
        None => return false,
    };

    let trampoline = entry.trampoline;
    let stolen_size = entry.stolen_size;
    drop(registry);

    unsafe {
        if stolen_size == 4 {
            let offset = (trampoline as isize) - (target as isize);
            if (-B_RANGE..B_RANGE).contains(&offset) {
                let b_instr = 0x14000000 | (((offset >> 2) as u32) & 0x03FFFFFF);
                return patch::stealth_write(target, &b_instr.to_le_bytes()).is_ok();
            }
        } else {
            let bytes = gen_branch_bytes(trampoline + 16);
            return patch::stealth_write(target, &bytes).is_ok();
        }
    }

    false
}

/// Allocates executable memory for trampoline
#[inline]
unsafe fn alloc_trampoline() -> Option<*mut c_void> {
    let ptr = libc::mmap(
        ptr::null_mut(),
        TRAMPOLINE_SIZE,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_PRIVATE | libc::MAP_ANON,
        -1,
        0,
    );
    if ptr == libc::MAP_FAILED {
        None
    } else {
        Some(ptr)
    }
}

/// Allocates trampoline within relative branch range
#[inline]
unsafe fn alloc_trampoline_near(target: usize) -> Option<*mut c_void> {
    let search_range = B_RANGE as usize - TRAMPOLINE_SIZE;
    for offset in (0..search_range).step_by(0x100000) {
        let hint = target.saturating_sub(offset);
        let hint_aligned = (hint & !(PAGE_SIZE - 1)) as *mut c_void;
        let ptr = libc::mmap(
            hint_aligned,
            TRAMPOLINE_SIZE,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANON,
            -1,
            0,
        );
        if ptr != libc::MAP_FAILED && (ptr as isize - target as isize).abs() < B_RANGE {
            return Some(ptr);
        }
        if ptr != libc::MAP_FAILED {
            libc::munmap(ptr, TRAMPOLINE_SIZE);
        }
    }
    for offset in (0..search_range).step_by(0x100000) {
        let hint = target.saturating_add(offset);
        let hint_aligned = (hint & !(PAGE_SIZE - 1)) as *mut c_void;
        let ptr = libc::mmap(
            hint_aligned,
            TRAMPOLINE_SIZE,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANON,
            -1,
            0,
        );
        if ptr != libc::MAP_FAILED && (ptr as isize - target as isize).abs() < B_RANGE {
            return Some(ptr);
        }
        if ptr != libc::MAP_FAILED {
            libc::munmap(ptr, TRAMPOLINE_SIZE);
        }
    }
    alloc_trampoline()
}

/// Patches a single instruction via stealth write
unsafe fn patch_short(target: usize, instr: u32) -> bool {
    patch::stealth_write(target, &instr.to_le_bytes()).is_ok()
}

/// Patches code with polymorphic redirect branch via stealth write
unsafe fn patch_code(target: usize, dest: usize) -> bool {
    let bytes = gen_branch_bytes(dest);
    patch::stealth_write(target, &bytes).is_ok()
}

/// Writes polymorphic absolute branch sequence (to writable memory like mmap'd trampolines)
#[inline]
unsafe fn emit_branch(addr: usize, dest: usize) {
    let bytes = gen_branch_bytes(dest);
    ptr::copy_nonoverlapping(bytes.as_ptr(), addr as *mut u8, 16);
}

/// Relocates PC-relative instructions to trampoline
unsafe fn relocate_instruction(instr: u32, pc: usize, tramp: usize) -> Option<usize> {
    let op24 = (instr >> 24) & 0x9F;
    let op26 = (instr >> 26) & 0x3F;
    let rd = (instr & 0x1F) as u8;

    let is_adr = op24 == 0x10;
    if is_adr || op24 == 0x90 {
        let immlo = (instr >> 29) & 0x3;
        let immhi = (instr >> 5) & 0x7FFFF;
        let mut imm = (immhi << 2) | immlo;
        if (imm & (1 << 20)) != 0 {
            imm |= !0xFFFFF;
        }
        let target_val = if is_adr {
            (pc as isize).wrapping_add(imm as isize) as usize
        } else {
            let pc_page = pc & !0xFFF;
            (pc_page as isize).wrapping_add((imm as isize) << 12) as usize
        };
        ptr::write(tramp as *mut u32, 0x58000040 | (rd as u32));
        ptr::write((tramp + 4) as *mut u32, 0x14000003);
        ptr::write((tramp + 8) as *mut usize, target_val);
        return Some(16);
    }

    let op_check = instr & 0x3B000000;
    if op_check == 0x18000000 || op_check == 0x58000000 || op_check == 0x98000000 {
        let imm19 = (instr >> 5) & 0x7FFFF;
        let mut offset = imm19 << 2;
        if (offset & (1 << 20)) != 0 {
            offset |= !0xFFFFF;
        }
        let target_addr = (pc as isize).wrapping_add(offset as isize) as usize;
        let ldr_reg_opcode = if op_check == 0x18000000 {
            0xB9400000 | (rd as u32)
        } else if op_check == 0x58000000 {
            0xF9400000 | (rd as u32)
        } else {
            0xB9800000 | (rd as u32)
        };
        ptr::write(tramp as *mut u32, 0x58000071);
        ptr::write((tramp + 4) as *mut u32, ldr_reg_opcode | (17 << 5));
        ptr::write((tramp + 8) as *mut u32, 0x14000003);
        ptr::write((tramp + 12) as *mut usize, target_addr);
        return Some(20);
    }

    if op26 == 0x05 || op26 == 0x25 {
        let imm26 = instr & 0x03FFFFFF;
        let mut offset = imm26 << 2;
        if (offset & (1 << 27)) != 0 {
            offset |= !0x0FFFFFFF;
        }
        let target_addr = (pc as isize).wrapping_add(offset as isize) as usize;
        if op26 == 0x25 {
            ptr::write(tramp as *mut u32, 0x100000BE);
            emit_branch(tramp + 4, target_addr);
            return Some(20);
        } else {
            emit_branch(tramp, target_addr);
            return Some(16);
        }
    }

    let op_byte = (instr >> 24) & 0xFF;
    let is_b_cond = op_byte == 0x54;
    let is_cbz_cbnz = matches!(op_byte, 0x34 | 0xB4 | 0x35 | 0xB5);
    let is_tbz_tbnz = matches!(op_byte, 0x36 | 0xB6 | 0x37 | 0xB7);

    if is_b_cond || is_cbz_cbnz || is_tbz_tbnz {
        let target_addr = if is_b_cond || is_cbz_cbnz {
            let imm19 = (instr >> 5) & 0x7FFFF;
            let offset = if (imm19 & (1 << 18)) != 0 {
                ((imm19 | 0xFFF80000) as i32) as isize
            } else {
                imm19 as isize
            };
            (pc as isize).wrapping_add(offset * 4) as usize
        } else {
            let imm14 = (instr >> 5) & 0x3FFF;
            let offset = if (imm14 & (1 << 13)) != 0 {
                ((imm14 | 0xFFFFC000) as i32) as isize
            } else {
                imm14 as isize
            };
            (pc as isize).wrapping_add(offset * 4) as usize
        };
        let inverted = if is_b_cond {
            ((instr & 0xFF00000F) ^ 1) | (5 << 5)
        } else if is_cbz_cbnz {
            ((instr & 0xFF00001F) ^ (1 << 24)) | (5 << 5)
        } else {
            ((instr & 0xFFF8001F) ^ (1 << 24)) | (5 << 5)
        };
        ptr::write(tramp as *mut u32, inverted);
        ptr::write((tramp + 4) as *mut u32, 0x58000051);
        ptr::write((tramp + 8) as *mut u32, 0xD61F0220);
        ptr::write((tramp + 12) as *mut usize, target_addr);
        return Some(20);
    }

    ptr::write(tramp as *mut u32, instr);
    Some(4)
}

/// Installs a hook using a code cave for the trampoline (harder to detect)
///
/// # Arguments
/// * `rva` - The relative virtual address to hook
/// * `replacement` - The address of the replacement function
///
/// # Returns
/// * `Result<Hook, HookError>` - The installed hook or an error
pub unsafe fn install_in_cave(rva: usize, replacement: usize) -> Result<Hook, HookError> {
    let base = image::get_image_base(config::TARGET_IMAGE_NAME)?;
    let target = base + rva;
    install_in_cave_at_address(target, replacement)
}

/// Installs a hook using a code cave at an absolute address
///
/// # Arguments
/// * `target` - The absolute address to hook
/// * `replacement` - The address of the replacement function
///
/// # Returns
/// * `Result<Hook, HookError>` - The installed hook or an error
pub unsafe fn install_in_cave_at_address(
    target: usize,
    replacement: usize,
) -> Result<Hook, HookError> {
    if REGISTRY.lock().contains_key(&target) {
        return Err(HookError::AlreadyExists(target));
    }

    let cave =
        code_cave::allocate_cave_near(target, 128).map_err(|_| HookError::AllocationFailed)?;

    let trampoline_base = (cave.address + 3) & !3;
    let suspended = thread::suspend_other_threads()?;

    let mut original = vec![0u8; MAX_STOLEN_BYTES];
    ptr::copy_nonoverlapping(target as *const u8, original.as_mut_ptr(), MAX_STOLEN_BYTES);

    let mut tramp_buf = vec![0u8; 128];
    let buf_base = tramp_buf.as_mut_ptr() as usize;

    let mut cave_offset = 0;
    for i in 0..4 {
        let instr = super::rw::read::<u32>(target + i * 4).unwrap_or(0);
        if let Some(size) = relocate_instruction(instr, target + i * 4, buf_base + cave_offset) {
            cave_offset += size;
        } else {
            code_cave::free_cave(cave.address).ok();
            thread::resume_threads(&suspended);
            return Err(HookError::RelocationFailed);
        }
    }

    emit_branch(buf_base + cave_offset, target + MAX_STOLEN_BYTES);
    cave_offset += 16;

    if patch::stealth_write(trampoline_base, &tramp_buf[..cave_offset]).is_err() {
        code_cave::free_cave(cave.address).ok();
        thread::resume_threads(&suspended);
        return Err(HookError::PatchFailed);
    }

    if !patch_code(target, replacement) {
        code_cave::free_cave(cave.address).ok();
        thread::resume_threads(&suspended);
        return Err(HookError::PatchFailed);
    }

    REGISTRY.lock().insert(
        target,
        HookEntry {
            target,
            original,
            trampoline: trampoline_base,
            stolen_size: MAX_STOLEN_BYTES,
        },
    );
    checksum::register(target, MAX_STOLEN_BYTES);

    thread::resume_threads(&suspended);
    #[cfg(dev_release)]
    logger::debug("Cave hook installed");

    Ok(Hook {
        target,
        trampoline: trampoline_base,
    })
}

/// Returns the number of active hooks
///
/// # Returns
/// * `usize` - The count of active hooks
pub fn hook_count() -> usize {
    REGISTRY.lock().len()
}

/// Lists all active hook target addresses
///
/// # Returns
/// * `Vec<usize>` - A list of addresses where hooks are installed
pub fn list_hooks() -> Vec<usize> {
    REGISTRY.lock().keys().copied().collect()
}

/// Checks if an address has an active hook
///
/// # Arguments
/// * `target` - The address to check
///
/// # Returns
/// * `bool` - `true` if a hook exists at the address
pub fn is_hooked(target: usize) -> bool {
    REGISTRY.lock().contains_key(&target)
}
