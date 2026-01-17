//! ARM64 inline hooking

use crate::config;
use crate::memory::{image, patch, thread};
use crate::utils::logger;
use mach2::{
    kern_return::KERN_SUCCESS,
    traps::mach_task_self,
    vm::mach_vm_protect,
    vm_prot::{VM_PROT_COPY, VM_PROT_EXECUTE, VM_PROT_READ, VM_PROT_WRITE},
    vm_types::{mach_vm_address_t, mach_vm_size_t},
};
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

struct HookEntry {
    target: usize,
    original: Vec<u8>,
    trampoline: usize,
    stolen_size: usize,
}

static REGISTRY: Lazy<Mutex<HashMap<usize, HookEntry>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Error, Debug)]
pub enum HookError {
    #[error("Hook exists: {0:#x}")]
    AlreadyExists(usize),
    #[error("Image not found: {0}")]
    ImageBaseNotFound(#[from] super::image::ImageError),
    #[error("Alloc failed")]
    AllocationFailed,
    #[error("Protection failed: {0}")]
    ProtectionFailed(i32),
    #[error("Patch failed")]
    PatchFailed,
    #[error("Relocation failed")]
    RelocationFailed,
    #[error("Thread error: {0}")]
    ThreadError(#[from] super::thread::ThreadError),
}


pub struct Hook {
    target: usize,
    trampoline: usize,
}


impl Hook {
    #[inline]
    pub fn trampoline(&self) -> usize {
        self.trampoline
    }
    #[inline]
    pub unsafe fn trampoline_as<F>(&self) -> F
    where
        F: Copy,
    {
        std::mem::transmute_copy(&self.trampoline)
    }
    pub fn remove(self) {
        unsafe {
            remove_at_address(self.target);
        }
    }
}

pub unsafe fn install(rva: usize, replacement: usize) -> Result<Hook, HookError> {
    let base = image::get_image_base(config::TARGET_IMAGE_NAME)?;
    let target = base + rva;
    let trampoline = install_at_address(target, replacement)?;
    Ok(Hook { target, trampoline })
}

pub unsafe fn install_at_address(target: usize, replacement: usize) -> Result<usize, HookError> {
    if REGISTRY.lock().contains_key(&target) {
        logger::warning(&format!("Hook already at {:#x}", target));
        return Err(HookError::AlreadyExists(target));
    }
    let first_instr = super::rw::read::<u32>(target).map_err(|_| HookError::PatchFailed)?;
    if is_b_instruction(first_instr) {
        logger::info("Detected thunk, using short hook");
        return install_thunk_hook(target, replacement, first_instr);
    }
    install_regular_hook(target, replacement)
}

fn is_b_instruction(instr: u32) -> bool {
    (instr >> 26) & 0x3F == 0x05
}

fn decode_b_target(instr: u32, pc: usize) -> usize {
    let imm26 = instr & 0x03FFFFFF;
    let mut offset = (imm26 << 2) as i32;
    if (offset & (1 << 27)) != 0 {
        offset |= !0x0FFFFFFF_u32 as i32;
    }
    (pc as isize).wrapping_add(offset as isize) as usize
}

fn encode_b_instruction(from: usize, to: usize) -> Option<u32> {
    let offset = (to as isize) - (from as isize);
    if offset < -B_RANGE || offset >= B_RANGE {
        return None;
    }
    Some(0x14000000 | (((offset >> 2) as u32) & 0x03FFFFFF))
}

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

    let task = mach_task_self();
    if mach_vm_protect(
        task,
        trampoline_base as mach_vm_address_t,
        TRAMPOLINE_SIZE as mach_vm_size_t,
        0,
        VM_PROT_READ | VM_PROT_EXECUTE,
    ) != KERN_SUCCESS
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
    thread::resume_threads(&suspended);
    logger::info(&format!("Hook: {:#x} → {:#x}", target, replacement));
    Ok(trampoline_base)
}

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

    let task = mach_task_self();
    if mach_vm_protect(
        task,
        trampoline_base as mach_vm_address_t,
        TRAMPOLINE_SIZE as mach_vm_size_t,
        0,
        VM_PROT_READ | VM_PROT_EXECUTE,
    ) != KERN_SUCCESS
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
    thread::resume_threads(&suspended);
    logger::info(&format!("Hook: {:#x} → {:#x}", target, replacement));
    Ok(trampoline_base)
}


pub unsafe fn remove(rva: usize) -> bool {
    match image::get_image_base(config::TARGET_IMAGE_NAME) {
        Ok(base) => remove_at_address(base + rva),
        Err(_) => false,
    }
}

pub unsafe fn remove_at_address(target: usize) -> bool {
    let entry = match REGISTRY.lock().remove(&target) {
        Some(e) => e,
        None => return false,
    };

    let suspended = match thread::suspend_other_threads() {
        Ok(s) => s,
        Err(_) => return false,
    };

    let task = mach_task_self();
    let page_mask = !(PAGE_SIZE - 1);
    let page = entry.target & page_mask;
    let page_len = ((entry.target + entry.stolen_size + PAGE_SIZE - 1) & page_mask) - page;

    if mach_vm_protect(
        task,
        page as mach_vm_address_t,
        page_len as mach_vm_size_t,
        0,
        VM_PROT_READ | VM_PROT_WRITE | VM_PROT_COPY,
    ) == KERN_SUCCESS
    {
        ptr::copy_nonoverlapping(
            entry.original.as_ptr(),
            entry.target as *mut u8,
            entry.stolen_size,
        );
        mach_vm_protect(
            task,
            page as mach_vm_address_t,
            page_len as mach_vm_size_t,
            0,
            VM_PROT_READ | VM_PROT_EXECUTE,
        );
        patch::invalidate_icache(entry.target as *mut c_void, entry.stolen_size);
    }

    libc::munmap(entry.trampoline as *mut c_void, TRAMPOLINE_SIZE);
    thread::resume_threads(&suspended);
    logger::info(&format!("Unhook: {:#x}", target));
    true
}

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

unsafe fn patch_short(target: usize, instr: u32) -> bool {
    let task = mach_task_self();
    let page = target & !(PAGE_SIZE - 1);
    if mach_vm_protect(
        task,
        page as mach_vm_address_t,
        PAGE_SIZE as mach_vm_size_t,
        0,
        VM_PROT_READ | VM_PROT_WRITE | VM_PROT_COPY,
    ) != KERN_SUCCESS
    {
        return false;
    }
    ptr::write(target as *mut u32, instr);
    mach_vm_protect(
        task,
        page as mach_vm_address_t,
        PAGE_SIZE as mach_vm_size_t,
        0,
        VM_PROT_READ | VM_PROT_EXECUTE,
    );
    patch::invalidate_icache(target as *mut c_void, 4);
    true
}

unsafe fn patch_code(target: usize, dest: usize) -> bool {
    let task = mach_task_self();
    let page_mask = !(PAGE_SIZE - 1);
    let page = target & page_mask;
    let page_len = ((target + MAX_STOLEN_BYTES + PAGE_SIZE - 1) & page_mask) - page;
    if mach_vm_protect(
        task,
        page as mach_vm_address_t,
        page_len as mach_vm_size_t,
        0,
        VM_PROT_READ | VM_PROT_WRITE | VM_PROT_COPY,
    ) != KERN_SUCCESS
    {
        return false;
    }
    emit_branch(target, dest);
    mach_vm_protect(
        task,
        page as mach_vm_address_t,
        page_len as mach_vm_size_t,
        0,
        VM_PROT_READ | VM_PROT_EXECUTE,
    );
    patch::invalidate_icache(target as *mut c_void, MAX_STOLEN_BYTES);
    true
}

#[inline]
unsafe fn emit_branch(addr: usize, dest: usize) {
    const BRANCH_CODE: [u8; 8] = [0x51, 0x00, 0x00, 0x58, 0x20, 0x02, 0x1F, 0xD6];
    ptr::copy_nonoverlapping(BRANCH_CODE.as_ptr(), addr as *mut u8, 8);
    *((addr + 8) as *mut usize) = dest;
}

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