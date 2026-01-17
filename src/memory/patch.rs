//! Memory patching

use crate::utils::logger;
use jit_assembler::aarch64::Aarch64InstructionBuilder;
use jit_assembler::common::InstructionBuilder;
use std::arch::asm;
use std::ffi::c_void;
use thiserror::Error;

const CACHE_LINE_SIZE: usize = 64;

#[derive(Error, Debug)]

pub enum PatchError {
    #[error("Invalid hex: {0}")]
    InvalidHex(#[from] hex::FromHexError),
    #[error("Image not found: {0}")]
    ImageBaseNotFound(#[from] super::image::ImageError),
    #[error("Protection failed: {0}")]
    ProtectionFailed(i32),
    #[error("Thread error: {0}")]
    ThreadError(#[from] super::thread::ThreadError),
    #[error("Empty instructions")]
    EmptyInstructions,
}

pub struct Patch {
    
    address: usize,
    
    original_bytes: Vec<u8>,
}

impl Patch {
    
    pub fn revert(&self) {
        unsafe {
            if let Err(e) = write_bytes(self.address, &self.original_bytes) {
                logger::error(&format!("Revert failed: {}", e));
            }
        }
    }
}


pub fn apply(rva: usize, hex_str: &str) -> Result<Patch, PatchError> {
    let clean: String = hex_str.chars().filter(|c| !c.is_whitespace()).collect();
    let bytes = hex::decode(&clean)?;
    let base = super::image::get_image_base(crate::config::TARGET_IMAGE_NAME)?;
    let address = base + rva;
    let original_bytes = unsafe { read_bytes(address, bytes.len()) };
    unsafe {
        write_bytes(address, &bytes)?;
    }
    Ok(Patch {
        address,
        original_bytes,
    })
}


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
    let base = super::image::get_image_base(crate::config::TARGET_IMAGE_NAME)?;
    let address = base + rva;
    let original_bytes = unsafe { read_bytes(address, bytes.len()) };
    unsafe {
        write_bytes(address, &bytes)?;
    }
    Ok(Patch {
        address,
        original_bytes,
    })
}

unsafe fn read_bytes(address: usize, len: usize) -> Vec<u8> {
    (0..len)
        .map(|i| super::rw::read::<u8>(address + i).unwrap_or(0))
        .collect()
}

unsafe fn write_bytes(address: usize, data: &[u8]) -> Result<(), PatchError> {
    for (i, &byte) in data.iter().enumerate() {
        super::rw::write(address + i, byte).map_err(|_| PatchError::ProtectionFailed(0))?;
    }
    Ok(())
}

#[inline]
pub unsafe fn invalidate_icache(start: *mut c_void, len: usize) {
    let start_addr = start as usize;
    let end_addr = start_addr + len;
    let mut addr = start_addr & !(CACHE_LINE_SIZE - 1);
    while addr < end_addr {
        asm!("ic ivau, {x}", x = in(reg) addr, options(nostack, preserves_flags));
        addr += CACHE_LINE_SIZE;
    }
    asm!("dsb ish", options(nostack, preserves_flags));
    asm!("isb", options(nostack, preserves_flags));
}
