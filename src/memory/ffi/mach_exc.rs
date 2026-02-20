//! Mach Exception Port FFI Bindings
//!
//! This module provides FFI bindings for Mach exception handling functions
//! that are not available in existing crates like mach2 or libc.

use mach2::port::mach_port_t;
use mach2::vm_types::{mach_vm_address_t, mach_vm_size_t};
use std::ffi::c_void;

/// Type alias for kernel return code
pub type KernReturnT = i32;
/// Mask for exception types
pub type ExceptionMaskT = u32;
/// Behavior for exception handling
pub type ExceptionBehaviorT = i32;
/// Flavor for thread state
pub type ThreadStateFlavor = i32;

extern "C" {
    /// Sets exception ports for a task
    ///
    /// # Arguments
    /// * `task` - The task port (usually `mach_task_self()`)
    /// * `exception_mask` - Mask of exceptions to catch (e.g., `EXC_MASK_BREAKPOINT`)
    /// * `new_port` - The port to send exception messages to
    /// * `behavior` - The behavior of the exception (e.g., `EXCEPTION_DEFAULT`)
    /// * `new_flavor` - The state flavor (e.g., `ARM_THREAD_STATE64`)
    ///
    /// # Returns
    /// * `KernReturnT` - `KERN_SUCCESS` on success
    pub fn task_set_exception_ports(
        task: mach_port_t,
        exception_mask: ExceptionMaskT,
        new_port: mach_port_t,
        behavior: ExceptionBehaviorT,
        new_flavor: ThreadStateFlavor,
    ) -> KernReturnT;

    /// Gets existing exception ports for a task
    ///
    /// # Arguments
    /// * `task` - The task port
    /// * `exception_mask` - Mask of exceptions to query
    /// * `masks` - Output array for masks
    /// * `masks_cnt` - Output count of masks
    /// * `old_handlers` - Output array for existing handler ports
    /// * `old_behaviors` - Output array for existing behaviors
    /// * `old_flavors` - Output array for existing flavors
    ///
    /// # Returns
    /// * `KernReturnT` - `KERN_SUCCESS` on success
    pub fn task_get_exception_ports(
        task: mach_port_t,
        exception_mask: ExceptionMaskT,
        masks: *mut ExceptionMaskT,
        masks_cnt: *mut u32,
        old_handlers: *mut mach_port_t,
        old_behaviors: *mut ExceptionBehaviorT,
        old_flavors: *mut ThreadStateFlavor,
    ) -> KernReturnT;

    /// Sets the execution state of a task/thread
    ///
    /// # Arguments
    /// * `task` - The task or thread port
    /// * `flavor` - The state flavor (e.g., `ARM_DEBUG_STATE64`)
    /// * `state` - Pointer to the state structure
    /// * `count` - Size of the state structure in 32-bit words
    ///
    /// # Returns
    /// * `KernReturnT` - `KERN_SUCCESS` on success
    pub fn task_set_state(
        task: mach_port_t,
        flavor: ThreadStateFlavor,
        state: *const c_void,
        count: u32,
    ) -> KernReturnT;

    /// Remaps a memory region
    ///
    /// # Arguments
    /// * `target_task` - The task port to remap to
    /// * `target_address` - The address to remap to
    /// * `size` - The size of the region to remap
    /// * `mask` - The mask of the region to remap
    /// * `flags` - The flags of the region to remap
    /// * `src_task` - The task port to remap from
    /// * `src_address` - The address to remap from
    /// * `copy` - Whether to copy the region
    /// * `cur_protection` - The current protection of the region
    /// * `max_protection` - The maximum protection of the region
    /// * `inheritance` - The inheritance of the region
    ///
    /// # Returns
    /// * `KernReturnT` - `KERN_SUCCESS` on success
    pub fn mach_vm_remap(
        target_task: mach_port_t,
        target_address: *mut mach_vm_address_t,
        size: mach_vm_size_t,
        mask: u64,
        flags: i32,
        src_task: mach_port_t,
        src_address: mach_vm_address_t,
        copy: i32,
        cur_protection: *mut i32,
        max_protection: *mut i32,
        inheritance: u32,
    ) -> i32;
}
