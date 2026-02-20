//! Memory page protection query utilities

use crate::utils::logger;
use mach2::{
    kern_return::KERN_SUCCESS,
    traps::mach_task_self,
    vm::mach_vm_region,
    vm_prot::{VM_PROT_EXECUTE, VM_PROT_READ, VM_PROT_WRITE},
    vm_region::{vm_region_basic_info_64, vm_region_info_t, VM_REGION_BASIC_INFO_64},
    vm_types::{mach_vm_address_t, mach_vm_size_t},
};
use thiserror::Error;

#[derive(Error, Debug)]
/// Errors that can occur during protection queries or modifications
pub enum ProtectionError {
    /// Failed to query memory region information from the kernel
    #[error("Failed to query region at {0:#x}")]
    QueryFailed(usize),
    /// The address is invalid or not mapped
    #[error("Invalid address {0:#x}")]
    InvalidAddress(usize),
    /// Failed to change memory protection
    #[error("Protection failed: {0}")]
    ProtectionFailed(i32),
}

/// Represents memory page protection flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageProtection {
    flags: i32,
}

impl PageProtection {
    /// Creates a new PageProtection from raw VM_PROT flags
    pub fn from_raw(flags: i32) -> Self {
        Self { flags }
    }

    /// Returns the raw VM_PROT flags
    pub fn raw(&self) -> i32 {
        self.flags
    }

    /// Checks if the page is readable
    pub fn is_readable(&self) -> bool {
        (self.flags & VM_PROT_READ) != 0
    }

    /// Checks if the page is writable
    pub fn is_writable(&self) -> bool {
        (self.flags & VM_PROT_WRITE) != 0
    }

    /// Checks if the page is executable
    pub fn is_executable(&self) -> bool {
        (self.flags & VM_PROT_EXECUTE) != 0
    }

    /// Creates a read-only protection
    pub fn read_only() -> Self {
        Self {
            flags: VM_PROT_READ,
        }
    }

    /// Creates a read-write protection
    pub fn read_write() -> Self {
        Self {
            flags: VM_PROT_READ | VM_PROT_WRITE,
        }
    }

    /// Creates a read-execute protection
    pub fn read_execute() -> Self {
        Self {
            flags: VM_PROT_READ | VM_PROT_EXECUTE,
        }
    }
}

/// Represents detailed information about a memory region
#[derive(Debug, Clone)]
pub struct RegionInfo {
    /// Base address of the region
    pub address: usize,
    /// Size of the region in bytes
    pub size: usize,
    /// Current protection flags
    pub protection: PageProtection,
}

/// Queries the current protection flags for a memory address
///
/// # Arguments
/// * `addr` - The address to query
///
/// # Returns
/// * `Result<PageProtection, ProtectionError>` - The protection flags or an error
pub fn get_protection(addr: usize) -> Result<PageProtection, ProtectionError> {
    let info = get_region_info(addr)?;
    Ok(info.protection)
}

/// Queries detailed region information for a memory address
///
/// # Arguments
/// * `addr` - The address to query
///
/// # Returns
/// * `Result<RegionInfo, ProtectionError>` - The region information or an error
pub fn get_region_info(addr: usize) -> Result<RegionInfo, ProtectionError> {
    let region = find_region(addr)?;

    if region.address > addr {
        return Err(ProtectionError::InvalidAddress(addr));
    }

    Ok(region)
}

/// Finds the memory region containing or following the given address
///
/// # Arguments
/// * `addr` - The address to search from
///
/// # Returns
/// * `Result<RegionInfo, ProtectionError>` - The found region or an error
pub fn find_region(addr: usize) -> Result<RegionInfo, ProtectionError> {
    unsafe {
        let task = mach_task_self();
        let mut address = addr as mach_vm_address_t;
        let mut region_size: mach_vm_size_t = 0;
        let mut info = vm_region_basic_info_64::default();
        let mut info_count = VM_REGION_BASIC_INFO_64;
        let mut object_name = 0;

        let kr = mach_vm_region(
            task,
            &mut address,
            &mut region_size,
            VM_REGION_BASIC_INFO_64,
            &mut info as *mut _ as *mut i32,
            &mut info_count as *mut _ as *mut u32,
            &mut object_name,
        );

        if kr != KERN_SUCCESS {
            return Err(ProtectionError::QueryFailed(addr));
        }

        Ok(RegionInfo {
            address: address as usize,
            size: region_size as usize,
            protection: PageProtection::from_raw(info.protection),
        })
    }
}

/// Quick check if an address is readable
pub fn is_readable(addr: usize) -> bool {
    get_protection(addr)
        .map(|p| p.is_readable())
        .unwrap_or(false)
}

/// Quick check if an address is writable
pub fn is_writable(addr: usize) -> bool {
    get_protection(addr)
        .map(|p| p.is_writable())
        .unwrap_or(false)
}

/// Quick check if an address is executable
pub fn is_executable(addr: usize) -> bool {
    get_protection(addr)
        .map(|p| p.is_executable())
        .unwrap_or(false)
}

/// Changes the protection of a memory region
///
/// # Arguments
/// * `addr` - The start address
/// * `size` - The size of the region
/// * `protection` - The new protection flags
///
/// # Returns
/// * `Result<(), ProtectionError>` - Result indicating success or failure
pub fn protect(
    addr: usize,
    size: usize,
    protection: PageProtection,
) -> Result<(), ProtectionError> {
    unsafe {
        let task = mach_task_self();
        let kr = mach2::vm::mach_vm_protect(
            task,
            addr as mach_vm_address_t,
            size as mach_vm_size_t,
            0,
            protection.raw(),
        );

        if kr != KERN_SUCCESS {
            return Err(ProtectionError::ProtectionFailed(kr));
        }
        Ok(())
    }
}

/// Enumerates all readable memory regions in the process
///
/// # Returns
/// * `Result<Vec<RegionInfo>, ProtectionError>` - A list of regions or an error
pub fn get_all_regions() -> Result<Vec<RegionInfo>, ProtectionError> {
    let mut regions = Vec::new();
    let task = unsafe { mach_task_self() };
    let mut address: mach_vm_address_t = 0;

    loop {
        let mut size: mach_vm_size_t = 0;
        let mut info = vm_region_basic_info_64::default();
        let mut info_count = vm_region_basic_info_64::count();
        let mut object_name: u32 = 0;

        let kr = unsafe {
            mach_vm_region(
                task,
                &mut address,
                &mut size,
                VM_REGION_BASIC_INFO_64,
                &mut info as *mut _ as vm_region_info_t,
                &mut info_count,
                &mut object_name,
            )
        };

        if kr != KERN_SUCCESS {
            break;
        }

        if (info.protection & VM_PROT_READ) != 0 {
            regions.push(RegionInfo {
                address: address as usize,
                size: size as usize,
                protection: PageProtection::from_raw(info.protection),
            });
        }

        address += size;
    }

    #[cfg(dev_release)]
    logger::info(&format!("Found {} memory regions", regions.len()));
    Ok(regions)
}
