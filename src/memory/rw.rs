//! Memory Read/Write Utilities

use std::ffi::c_void;
use std::ptr;

use mach2::{
    kern_return::KERN_SUCCESS,
    traps::mach_task_self,
    vm::mach_vm_protect,
    vm_prot::{VM_PROT_COPY, VM_PROT_EXECUTE, VM_PROT_READ, VM_PROT_WRITE},
    vm_types::{mach_vm_address_t, mach_vm_size_t},
};

use crate::utils::logger;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RwError {
    #[error("Null pointer")]
    NullPointer,
    #[error("Image not found: {0}")]
    ImageBaseNotFound(#[from] super::image::ImageError),
    #[error("Protection failed: {0}")]
    ProtectionFailed(i32),
    #[error("Thread error: {0}")]
    ThreadError(#[from] super::thread::ThreadError),
}


pub unsafe fn read<T: Copy>(address: usize) -> Result<T, RwError> {
    if address == 0 {
        return Err(RwError::NullPointer);
    }
    Ok(ptr::read(address as *const T))
}


pub unsafe fn read_at_rva<T: Copy>(rva: usize) -> Result<T, RwError> {
    let base = super::image::get_image_base(crate::config::TARGET_IMAGE_NAME)?;
    read::<T>(base + rva)
}


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


pub unsafe fn write<T: Copy>(address: usize, value: T) -> Result<(), RwError> {
    if address == 0 {
        return Err(RwError::NullPointer);
    }
    ptr::write(address as *mut T, value);
    Ok(())
}

/// Write a value to code/executable memory. Changes protection and invalidates icache.

pub unsafe fn write_code<T: Copy>(address: usize, value: T) -> Result<(), RwError> {
    if address == 0 {
        return Err(RwError::NullPointer);
    }
    let size = std::mem::size_of::<T>();
    let data = std::slice::from_raw_parts(&value as *const T as *const u8, size);
    write_bytes_internal(address, data)
}


pub unsafe fn write_at_rva<T: Copy>(rva: usize, value: T) -> Result<(), RwError> {
    let base = super::image::get_image_base(crate::config::TARGET_IMAGE_NAME)?;
    write::<T>(base + rva, value)
}

unsafe fn write_bytes_internal(address: usize, data: &[u8]) -> Result<(), RwError> {
    let page_size = libc::sysconf(libc::_SC_PAGESIZE) as usize;
    let page_mask = !(page_size - 1);
    let page_start = address & page_mask;
    let page_len = ((address + data.len() + page_size - 1) & page_mask) - page_start;

    let suspended = super::thread::suspend_other_threads()?;
    let task = mach_task_self();

    let kr = mach_vm_protect(
        task,
        page_start as mach_vm_address_t,
        page_len as mach_vm_size_t,
        0,
        VM_PROT_READ | VM_PROT_WRITE | VM_PROT_COPY,
    );

    if kr != KERN_SUCCESS {
        logger::error("vm_protect RW failed");
        super::thread::resume_threads(&suspended);
        return Err(RwError::ProtectionFailed(kr));
    }

    ptr::copy_nonoverlapping(data.as_ptr(), address as *mut u8, data.len());

    mach_vm_protect(
        task,
        page_start as mach_vm_address_t,
        page_len as mach_vm_size_t,
        0,
        VM_PROT_READ | VM_PROT_EXECUTE,
    );

    super::patch::invalidate_icache(address as *mut c_void, data.len());
    logger::info(&format!("Wrote {} bytes at {:#x}", data.len(), address));

    super::thread::resume_threads(&suspended);
    Ok(())
}
