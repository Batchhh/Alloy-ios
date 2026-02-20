//! Shellcode loader with symbol resolution and position-independent code support

use crate::memory::{code_cave, info::symbol, rw};
use crate::utils::logger;
use std::arch::asm;
use std::collections::HashMap;
use std::ffi::c_void;
use thiserror::Error;

const CACHE_LINE_SIZE: usize = 64;

#[derive(Error, Debug)]
/// Errors that can occur during shellcode loading or execution
pub enum LoaderError {
    /// Failed to allocate memory for the shellcode (e.g., no suitable code cave found)
    #[error("Code cave allocation failed: {0}")]
    AllocationFailed(#[from] code_cave::CodeCaveError),
    /// A required symbol for relocation was not found
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),
    /// Failed to write shellcode to memory
    #[error("Write failed: {0}")]
    WriteFailed(#[from] rw::RwError),
    /// The input bytes or allocation size is invalid
    #[error("Invalid shellcode size: {0}")]
    InvalidSize(usize),
    /// Failed to apply a relocation at the specified offset
    #[error("Relocation failed at offset {0}")]
    RelocationFailed(usize),
}

/// Represents loaded executable shellcode in memory
pub struct LoadedShellcode {
    /// The base address of the loaded code
    pub address: usize,
    /// The size of the loaded code
    pub size: usize,
    /// Whether to automatically free the memory on drop
    auto_free: bool,
}

impl LoadedShellcode {
    /// Executes the shellcode as a function with no arguments, returning usize
    ///
    /// # Safety
    /// Caller must ensure the shellcode is valid, matches the signature, and respects ABI conventions.
    pub unsafe fn execute(&self) -> usize {
        let func: extern "C" fn() -> usize = std::mem::transmute(self.address);
        func()
    }

    /// Executes the shellcode with a custom function signature
    ///
    /// # Safety
    /// Caller must ensure the signature `F` matches the shellcode's implementation.
    pub unsafe fn execute_as<F, R>(&self, callback: impl FnOnce(F) -> R) -> R
    where
        F: Copy,
    {
        let func_ptr = self.address as *const ();
        let func: F = *(&func_ptr as *const *const () as *const F);
        callback(func)
    }

    /// Gets a function pointer to the shellcode
    ///
    /// # Safety
    /// Caller must ensure the signature `F` matches the shellcode's implementation.
    pub unsafe fn as_function<F>(&self) -> F
    where
        F: Copy,
    {
        let func_ptr = self.address as *const ();
        *(&func_ptr as *const *const () as *const F)
    }

    /// Manually free the shellcode memory
    pub fn free(self) {
        if self.address != 0 {
            let _ = code_cave::free_cave(self.address);
        }
    }
}

impl Drop for LoadedShellcode {
    fn drop(&mut self) {
        if self.auto_free && self.address != 0 {
            let _ = code_cave::free_cave(self.address);
        }
    }
}

#[derive(Debug, Clone)]
/// Represents a symbol that needs to be resolved and patched into the shellcode
pub struct SymbolRelocation {
    /// The offset within the shellcode where the address should be written
    pub offset: usize,
    /// The name of the symbol to resolve
    pub symbol_name: String,
}

/// Builder for configuring and loading shellcode
pub struct ShellcodeBuilder {
    code: Vec<u8>,
    relocations: Vec<SymbolRelocation>,
    auto_free: bool,
    target_address: Option<usize>,
}

impl ShellcodeBuilder {
    /// Creates a new shellcode builder from raw bytes
    ///
    /// # Arguments
    /// * `shellcode` - The raw machine code bytes
    pub fn new(shellcode: &[u8]) -> Self {
        Self {
            code: shellcode.to_vec(),
            relocations: Vec::new(),
            auto_free: true,
            target_address: None,
        }
    }

    /// Creates a shellcode builder from ARM64 instructions (u32 array)
    ///
    /// # Arguments
    /// * `instructions` - Slice of 32-bit ARM64 instructions
    pub fn from_instructions(instructions: &[u32]) -> Self {
        let bytes: Vec<u8> = instructions
            .iter()
            .flat_map(|&instr| instr.to_le_bytes())
            .collect();
        Self::new(&bytes)
    }

    /// Adds a symbol relocation for dynamic linking
    ///
    /// The loader will resolve the symbol address and write it at the specified offset.
    ///
    /// # Arguments
    /// * `offset` - Byte offset in the shellcode
    /// * `symbol_name` - Name of the symbol to resolve
    pub fn with_symbol(mut self, offset: usize, symbol_name: &str) -> Self {
        self.relocations.push(SymbolRelocation {
            offset,
            symbol_name: symbol_name.to_string(),
        });
        self
    }

    /// Disables automatic cleanup (shellcode won't be freed on drop)
    ///
    /// Use this if you want the shellcode to persist for the lifetime of the process.
    pub fn no_auto_free(mut self) -> Self {
        self.auto_free = false;
        self
    }

    /// Tries to load shellcode near a specific address (within branch range)
    ///
    /// Use this if your shellcode contains relative branches to nearby code.
    pub fn near_address(mut self, target: usize) -> Self {
        self.target_address = Some(target);
        self
    }

    /// Loads the shellcode into memory with all configured options
    ///
    /// This will:
    /// 1. Allocate a code cave (optionally near a target)
    /// 2. Resolve and apply symbol relocations
    /// 3. Write the code to memory
    /// 4. Flush the instruction cache
    ///
    /// # Returns
    /// * `Result<LoadedShellcode, LoaderError>` - The loaded shellcode handle or an error
    pub fn load(self) -> Result<LoadedShellcode, LoaderError> {
        if self.code.is_empty() {
            return Err(LoaderError::InvalidSize(0));
        }

        let mut cave = if let Some(target) = self.target_address {
            code_cave::allocate_cave_near(target, self.code.len())?
        } else {
            code_cave::allocate_cave(self.code.len())?
        };

        let alignment = cave.address % 4;
        if alignment != 0 {
            let adjust = 4 - alignment;
            cave.address += adjust;
            cave.size = cave.size.saturating_sub(adjust);

            if cave.size < self.code.len() {
                return Err(LoaderError::InvalidSize(cave.size));
            }
        }

        let mut code = self.code.clone();

        let mut symbol_cache: HashMap<String, usize> = HashMap::new();
        for reloc in &self.relocations {
            let symbol_addr = if let Some(&cached) = symbol_cache.get(&reloc.symbol_name) {
                cached
            } else {
                let addr = symbol::resolve_symbol(&reloc.symbol_name)
                    .map_err(|_| LoaderError::SymbolNotFound(reloc.symbol_name.clone()))?;

                symbol_cache.insert(reloc.symbol_name.clone(), addr);
                addr
            };

            if reloc.offset + 8 > code.len() {
                return Err(LoaderError::RelocationFailed(reloc.offset));
            }

            let addr_bytes = (symbol_addr as u64).to_le_bytes();
            code[reloc.offset..reloc.offset + 8].copy_from_slice(&addr_bytes);
        }

        unsafe {
            rw::write_bytes(cave.address, &code)?;
        }

        // Verify shellcode
        unsafe {
            for (i, &expected) in code.iter().enumerate() {
                match rw::read::<u8>(cave.address + i) {
                    Ok(actual) => {
                        if actual != expected {
                            return Err(LoaderError::WriteFailed(rw::RwError::NullPointer));
                        }
                    }
                    Err(e) => {
                        return Err(LoaderError::WriteFailed(e));
                    }
                }
            }
        }

        if !crate::memory::protection::is_executable(cave.address) {
            return Err(LoaderError::InvalidSize(0));
        }

        // Flush instruction cache
        unsafe {
            invalidate_icache(cave.address as *mut c_void, code.len());
        }

        #[cfg(dev_release)]
        logger::info(&format!(
            "Shellcode loaded at {:#x} ({} bytes)",
            cave.address,
            code.len()
        ));

        Ok(LoadedShellcode {
            address: cave.address,
            size: code.len(),
            auto_free: self.auto_free,
        })
    }
}

/// Trait for types that can be converted to shellcode bytes
pub trait ShellcodeSource {
    fn into_bytes(self) -> Vec<u8>;
}

impl ShellcodeSource for &[u8] {
    fn into_bytes(self) -> Vec<u8> {
        self.to_vec()
    }
}

impl ShellcodeSource for &[u32] {
    fn into_bytes(self) -> Vec<u8> {
        self.iter().flat_map(|&instr| instr.to_le_bytes()).collect()
    }
}

impl<const N: usize> ShellcodeSource for &[u32; N] {
    fn into_bytes(self) -> Vec<u8> {
        self.iter().flat_map(|&instr| instr.to_le_bytes()).collect()
    }
}

impl<const N: usize> ShellcodeSource for &[u8; N] {
    fn into_bytes(self) -> Vec<u8> {
        self.to_vec()
    }
}

/// Unified helper to load shellcode from bytes or ARM64 instructions
///
/// # Arguments
/// * `source` - The shellcode source (bytes, instruction array, etc.)
///
/// # Returns
/// * `Result<LoadedShellcode, LoaderError>` - The loaded shellcode handle or an error
pub fn load(source: impl ShellcodeSource) -> Result<LoadedShellcode, LoaderError> {
    ShellcodeBuilder::new(&source.into_bytes()).load()
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
