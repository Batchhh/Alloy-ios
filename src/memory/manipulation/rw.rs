//! Memory Read/Write Utilities

use std::ptr;

use crate::utils::logger;
use thiserror::Error;

#[derive(Error, Debug)]
/// Errors that can occur during memory read/write operations
pub enum RwError {
    /// The target address is null
    #[error("Null pointer")]
    NullPointer,
    /// Failed to find the target image base
    #[error("Image not found: {0}")]
    ImageBaseNotFound(#[from] crate::memory::info::image::ImageError),
    /// Failed to change memory protection
    #[error("Protection failed: {0}")]
    ProtectionFailed(i32),
    /// Thread manipulation error
    #[error("Thread error: {0}")]
    ThreadError(#[from] crate::memory::platform::thread::ThreadError),
}

/// Reads a value of type T from the specified address
///
/// # Type Parameters
/// * `T` - The type of value to read (must implement `Copy`)
///
/// # Arguments
/// * `address` - The absolute address to read from
///
/// # Returns
/// * `Result<T, RwError>` - The read value or an error
pub unsafe fn read<T: Copy>(address: usize) -> Result<T, RwError> {
    if address == 0 {
        return Err(RwError::NullPointer);
    }
    Ok(ptr::read(address as *const T))
}

/// Reads a value of type T from a relative virtual address (RVA)
///
/// # Type Parameters
/// * `T` - The type of value to read (must implement `Copy`)
///
/// # Arguments
/// * `rva` - The relative virtual address to read from
///
/// # Returns
/// * `Result<T, RwError>` - The read value or an error
pub unsafe fn read_at_rva<T: Copy>(rva: usize) -> Result<T, RwError> {
    let base = crate::memory::info::image::get_image_base(crate::config::TARGET_IMAGE_NAME)?;
    read::<T>(base + rva)
}

/// Reads a pointer chain starting from a base address
///
/// # Arguments
/// * `base` - The base address
/// * `offsets` - A list of offsets to follow
///
/// # Returns
/// * `Result<usize, RwError>` - The final address or an error
pub unsafe fn read_pointer_chain(base: usize, offsets: &[usize]) -> Result<usize, RwError> {
    if base == 0 {
        return Err(RwError::NullPointer);
    }

    let mut current = base;
    for (i, &offset) in offsets.iter().enumerate() {
        if i < offsets.len() - 1 {
            let ptr = (current + offset) as *const usize;
            if ptr.is_null() {
                return Err(RwError::NullPointer);
            }
            current = ptr::read(ptr);
            if current == 0 {
                return Err(RwError::NullPointer);
            }
        } else {
            current += offset;
        }
    }
    Ok(current)
}

/// Writes a value of type T to the specified address
///
/// # Type Parameters
/// * `T` - The type of value to write (must implement `Copy`)
///
/// # Arguments
/// * `address` - The absolute address to write to
/// * `value` - The value to write
///
/// # Returns
/// * `Result<(), RwError>` - A result indicating success or failure
pub unsafe fn write<T: Copy>(address: usize, value: T) -> Result<(), RwError> {
    if address == 0 {
        return Err(RwError::NullPointer);
    }
    ptr::write(address as *mut T, value);
    Ok(())
}

/// Writes a value to code/executable memory via stealth write and invalidates icache.
///
/// # Type Parameters
/// * `T` - The type of value to write (must implement `Copy`)
///
/// # Arguments
/// * `address` - The absolute address to write to
/// * `value` - The value to write
///
/// # Returns
/// * `Result<(), RwError>` - A result indicating success or failure
pub unsafe fn write_code<T: Copy>(address: usize, value: T) -> Result<(), RwError> {
    if address == 0 {
        return Err(RwError::NullPointer);
    }
    let size = std::mem::size_of::<T>();
    let data = std::slice::from_raw_parts(&value as *const T as *const u8, size);
    write_bytes(address, data)
}

/// Writes a value of type T to a relative virtual address (RVA)
///
/// # Type Parameters
/// * `T` - The type of value to write (must implement `Copy`)
///
/// # Arguments
/// * `rva` - The relative virtual address to write to
/// * `value` - The value to write
///
/// # Returns
/// * `Result<(), RwError>` - A result indicating success or failure
pub unsafe fn write_at_rva<T: Copy>(rva: usize, value: T) -> Result<(), RwError> {
    let base = crate::memory::info::image::get_image_base(crate::config::TARGET_IMAGE_NAME)?;
    write::<T>(base + rva, value)
}

/// Writes a slice of bytes to the specified address via stealth write,
/// handling icache invalidation and thread safety.
///
/// Uses mach_vm_remap to avoid detectable vm_protect calls on code pages.
///
/// # Arguments
/// * `address` - The absolute address to write to
/// * `data` - The bytes to write
///
/// # Returns
/// * `Result<(), RwError>` - A result indicating success or failure
pub unsafe fn write_bytes(address: usize, data: &[u8]) -> Result<(), RwError> {
    let suspended = crate::memory::platform::thread::suspend_other_threads()?;

    let result = super::patch::stealth_write(address, data).map_err(|e| match e {
        super::patch::PatchError::ProtectionFailed(kr) => RwError::ProtectionFailed(kr),
        _ => RwError::ProtectionFailed(0),
    });

    crate::memory::platform::thread::resume_threads(&suspended);

    if result.is_ok() {
        #[cfg(dev_release)]
        logger::debug("Bytes written");
    }

    result
}
