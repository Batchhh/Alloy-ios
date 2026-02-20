//! # Hardware Breakpoint Hooking
//!
//! This module implements hooking using ARM64 hardware breakpoints (watchpoints/breakpoints).
//! This method is "stealthier" than inline patches as it doesn't modify the executable code,
//! but it is limited by the number of hardware debug registers (max 6).
//!
//! It uses Mach exception handling to catch the breakpoint exception and redirect execution.

use crate::memory::ffi::mach_exc::{
    task_get_exception_ports, task_set_exception_ports, task_set_state,
};
use crate::utils::logger;
use libc::{pthread_create, sysctlbyname};
use mach2::kern_return::KERN_SUCCESS;
use mach2::mach_init::mach_thread_self;
use mach2::mach_port::{mach_port_allocate, mach_port_deallocate, mach_port_insert_right};
use mach2::message::{mach_msg, mach_msg_header_t, mach_msg_type_name_t};
use mach2::port::{mach_port_t, MACH_PORT_NULL, MACH_PORT_RIGHT_RECEIVE};
use mach2::task::task_threads;
use mach2::thread_act::thread_set_state;
use mach2::traps::mach_task_self;
use mach2::vm::mach_vm_deallocate;

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::ffi::c_void;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use thiserror::Error;

use crate::memory::ffi::mach_exc::{
    ExceptionBehaviorT, ExceptionMaskT, KernReturnT, ThreadStateFlavor,
};

type ExceptionTypeT = i32;

const KERN_FAILURE: KernReturnT = 5;
const MACH_MSG_TYPE_MAKE_SEND: mach_msg_type_name_t = 20;
const EXC_MASK_BREAKPOINT: ExceptionMaskT = 1 << 6;
const EXCEPTION_STATE: ExceptionBehaviorT = 2;
const MACH_EXCEPTION_CODES: ExceptionBehaviorT = 0x80000000_u32 as i32;
const ARM_THREAD_STATE64: ThreadStateFlavor = 6;
const ARM_DEBUG_STATE64: ThreadStateFlavor = 15;
const ARM_DEBUG_STATE64_COUNT: u32 = 130;
const ARM_THREAD_STATE64_COUNT: u32 = 68;
const MAX_HOOKS: usize = 16;
const BCR_ENABLE: u64 = 0x1E5;
const MACH_RCV_MSG: i32 = 0x00000002;
const MACH_SEND_MSG: i32 = 0x00000001;
const MACH_MSG_TIMEOUT_NONE: u32 = 0;

#[repr(C)]
struct NdrRecord {
    mig_vers: u8,
    if_vers: u8,
    reserved1: u8,
    mig_encoding: u8,
    int_rep: u8,
    char_rep: u8,
    float_rep: u8,
    reserved2: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ArmThreadState64 {
    x: [u64; 29],
    fp: u64,
    lr: u64,
    sp: u64,
    pc: u64,
    cpsr: u32,
    flags: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ArmDebugState64 {
    bvr: [u64; 16],
    bcr: [u64; 16],
    wvr: [u64; 16],
    wcr: [u64; 16],
    mdscr_el1: u64,
}

impl Default for ArmDebugState64 {
    fn default() -> Self {
        Self {
            bvr: [0; 16],
            bcr: [0; 16],
            wvr: [0; 16],
            wcr: [0; 16],
            mdscr_el1: 0,
        }
    }
}

#[repr(C)]
struct ExcRaiseStateRequest {
    head: mach_msg_header_t,
    ndr: NdrRecord,
    exception: ExceptionTypeT,
    code_cnt: u32,
    code: [i64; 2],
    flavor: i32,
    old_state_cnt: u32,
    old_state: [u32; 614],
}

#[repr(C)]
struct ExcRaiseStateReply {
    head: mach_msg_header_t,
    ndr: NdrRecord,
    ret_code: KernReturnT,
    flavor: i32,
    new_state_cnt: u32,
    new_state: [u32; 614],
}

#[derive(Error, Debug)]
/// Errors that can occur during breakpoint operations
pub enum BrkHookError {
    /// The maximum number of hooks has been reached
    #[error("Too many hooks (max {MAX_HOOKS})")]
    TooManyHooks,
    /// A hook already exists at the specified address
    #[error("Hook already exists at {0:#x}")]
    AlreadyExists(usize),
    /// The hardware limit for breakpoints has been exceeded
    #[error("Exceeds hardware breakpoints: {0}")]
    ExceedsHwBreakpoints(i32),
    /// Failed to set the thread state (debug registers)
    #[error("Failed to set debug state")]
    SetStateFailed,
    /// The specified hook was not found
    #[error("Hook not found at {0:#x}")]
    NotFound(usize),
    /// Failed to initialize the exception handler
    #[error("Initialization failed")]
    InitFailed,
}

struct HookEntry {
    old: usize,
    new: usize,
}

struct HookManager {
    server_port: mach_port_t,
    orig_handler_port: mach_port_t,
    hooks: [Option<HookEntry>; MAX_HOOKS],
    active_count: usize,
    hw_breakpoints: i32,
    initialized: bool,
}

impl HookManager {
    /// Creates a new empty hook manager
    const fn new() -> Self {
        const NONE: Option<HookEntry> = None;
        Self {
            server_port: MACH_PORT_NULL,
            orig_handler_port: MACH_PORT_NULL,
            hooks: [NONE; MAX_HOOKS],
            active_count: 0,
            hw_breakpoints: 6, // hardware limitiation is 6, we can't increase this!
            initialized: false,
        }
    }

    /// Finds a replacement address for a given target PC
    fn find_hook(&self, pc: usize) -> Option<usize> {
        self.hooks
            .iter()
            .flatten()
            .find_map(|h| (h.old == pc).then_some(h.new))
    }

    /// Registers a new hook in the manager
    fn add_hook(&mut self, old: usize, new: usize) -> Result<usize, BrkHookError> {
        if self.hooks.iter().flatten().any(|h| h.old == old) {
            return Err(BrkHookError::AlreadyExists(old));
        }
        for (i, slot) in self.hooks.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(HookEntry { old, new });
                self.active_count += 1;
                return Ok(i);
            }
        }
        Err(BrkHookError::TooManyHooks)
    }

    /// Removes a hook from the manager
    fn remove_hook(&mut self, old: usize) -> Result<(), BrkHookError> {
        for slot in self.hooks.iter_mut() {
            if let Some(hook) = slot {
                if hook.old == old {
                    *slot = None;
                    self.active_count -= 1;
                    return Ok(());
                }
            }
        }
        Err(BrkHookError::NotFound(old))
    }
}

static MANAGER: Lazy<Mutex<HookManager>> = Lazy::new(|| Mutex::new(HookManager::new()));
static HANDLER_RUNNING: AtomicBool = AtomicBool::new(false);

/// Thread that handles Mach exceptions for breakpoints
extern "C" fn exception_handler_thread(_: *mut c_void) -> *mut c_void {
    HANDLER_RUNNING.store(true, Ordering::SeqCst);

    let server_port = MANAGER.lock().server_port;
    #[repr(C)]
    struct ExcMessage {
        head: mach_msg_header_t,
        body: [u8; 8192],
    }

    loop {
        let mut request: ExcMessage = unsafe { std::mem::zeroed() };
        let mut reply: ExcMessage = unsafe { std::mem::zeroed() };

        let kr = unsafe {
            mach_msg(
                &mut request.head,
                MACH_RCV_MSG,
                0,
                std::mem::size_of::<ExcMessage>() as u32,
                server_port,
                MACH_MSG_TIMEOUT_NONE,
                MACH_PORT_NULL,
            )
        };

        if kr != KERN_SUCCESS {
            continue;
        }

        if request.head.msgh_id == 2406 {
            let req_ptr = &request as *const _ as *const ExcRaiseStateRequest;
            let req = unsafe { &*req_ptr };
            let old_state_ptr = req.old_state.as_ptr() as *const ArmThreadState64;
            let old_state = unsafe { &*old_state_ptr };
            let pc = old_state.pc as usize;

            let replacement = { MANAGER.lock().find_hook(pc) };

            let reply_ptr = &mut reply as *mut _ as *mut ExcRaiseStateReply;
            let rep = unsafe { &mut *reply_ptr };

            rep.head.msgh_bits =
                (request.head.msgh_bits & 0xFF) | ((request.head.msgh_bits >> 8) & 0xFF) << 8;
            rep.head.msgh_remote_port = request.head.msgh_remote_port;
            rep.head.msgh_local_port = MACH_PORT_NULL;
            rep.head.msgh_id = request.head.msgh_id + 100;

            if let Some(new_pc) = replacement {
                let new_state_ptr = rep.new_state.as_mut_ptr() as *mut ArmThreadState64;
                unsafe {
                    ptr::copy_nonoverlapping(old_state, new_state_ptr, 1);
                    (*new_state_ptr).pc = new_pc as u64;
                }
                rep.new_state_cnt = ARM_THREAD_STATE64_COUNT;
                rep.flavor = ARM_THREAD_STATE64;
                rep.ret_code = KERN_SUCCESS;
                rep.ndr = NdrRecord {
                    mig_vers: 0,
                    if_vers: 0,
                    reserved1: 0,
                    mig_encoding: 0,
                    int_rep: 1,
                    char_rep: 0,
                    float_rep: 0,
                    reserved2: 0,
                };
                rep.head.msgh_size = 24 + 8 + 4 + 4 + 4 + (ARM_THREAD_STATE64_COUNT * 4);
            } else {
                rep.ret_code = KERN_FAILURE;
                rep.new_state_cnt = 0;
                rep.ndr = NdrRecord {
                    mig_vers: 0,
                    if_vers: 0,
                    reserved1: 0,
                    mig_encoding: 0,
                    int_rep: 1,
                    char_rep: 0,
                    float_rep: 0,
                    reserved2: 0,
                };
                rep.head.msgh_size = 44;
            }

            unsafe {
                mach_msg(
                    &mut reply.head,
                    MACH_SEND_MSG,
                    rep.head.msgh_size,
                    0,
                    MACH_PORT_NULL,
                    MACH_MSG_TIMEOUT_NONE,
                    MACH_PORT_NULL,
                );
            }
        }
    }
}

/// Initializes the global exception handler
unsafe fn init_exception_handler() -> Result<(), BrkHookError> {
    let mut manager = MANAGER.lock();
    if manager.initialized {
        return Ok(());
    }

    let task = mach_task_self();
    let mut bp_count: i32 = 6;
    let mut size = std::mem::size_of::<i32>();
    let sysctl_name = b"hw.optional.breakpoint\0";
    sysctlbyname(
        sysctl_name.as_ptr() as *const i8,
        &mut bp_count as *mut _ as *mut c_void,
        &mut size,
        ptr::null_mut(),
        0,
    );
    manager.hw_breakpoints = bp_count;

    let mut masks = [0u32; 32];
    let mut mask_cnt: u32 = 32;
    let mut old_handlers = [MACH_PORT_NULL; 32];
    let mut old_behaviors = [0i32; 32];
    let mut old_flavors = [0i32; 32];

    if task_get_exception_ports(
        task,
        EXC_MASK_BREAKPOINT,
        masks.as_mut_ptr(),
        &mut mask_cnt,
        old_handlers.as_mut_ptr(),
        old_behaviors.as_mut_ptr(),
        old_flavors.as_mut_ptr(),
    ) == KERN_SUCCESS
        && mask_cnt > 0
    {
        manager.orig_handler_port = old_handlers[0];
    }

    let mut server_port: mach_port_t = MACH_PORT_NULL;
    if mach_port_allocate(task, MACH_PORT_RIGHT_RECEIVE, &mut server_port) != KERN_SUCCESS {
        return Err(BrkHookError::InitFailed);
    }

    if mach_port_insert_right(task, server_port, server_port, MACH_MSG_TYPE_MAKE_SEND)
        != KERN_SUCCESS
    {
        return Err(BrkHookError::InitFailed);
    }

    if task_set_exception_ports(
        task,
        EXC_MASK_BREAKPOINT,
        server_port,
        EXCEPTION_STATE | MACH_EXCEPTION_CODES,
        ARM_THREAD_STATE64,
    ) != KERN_SUCCESS
    {
        return Err(BrkHookError::InitFailed);
    }

    manager.server_port = server_port;
    manager.initialized = true;

    if !HANDLER_RUNNING.load(Ordering::SeqCst) {
        let mut thread_handle: usize = 0;
        pthread_create(
            &mut thread_handle,
            ptr::null(),
            exception_handler_thread,
            ptr::null_mut(),
        );
    }

    #[cfg(dev_release)]
    logger::info(&format!(
        "Breakpoint hooking initialized ({} HW breakpoints)",
        bp_count
    ));
    Ok(())
}

/// Applies debug state (breakpoints) to all threads
unsafe fn apply_debug_state(manager: &HookManager) -> Result<(), BrkHookError> {
    let task = mach_task_self();
    let mut state = ArmDebugState64::default();

    let mut bp_idx = 0;
    for hook in manager.hooks.iter().flatten() {
        if bp_idx >= manager.hw_breakpoints as usize {
            break;
        }
        state.bvr[bp_idx] = hook.old as u64;
        state.bcr[bp_idx] = BCR_ENABLE;
        bp_idx += 1;
    }

    if task_set_state(
        task,
        ARM_DEBUG_STATE64,
        &state as *const _ as *const c_void,
        ARM_DEBUG_STATE64_COUNT,
    ) != KERN_SUCCESS
    {
        return Err(BrkHookError::SetStateFailed);
    }

    let mut threads: *mut mach_port_t = ptr::null_mut();
    let mut thread_count: u32 = 0;
    if task_threads(task, &mut threads, &mut thread_count) == KERN_SUCCESS {
        for i in 0..thread_count as isize {
            let thread = *threads.offset(i);
            thread_set_state(
                thread,
                ARM_DEBUG_STATE64,
                &state as *const _ as *mut u32,
                ARM_DEBUG_STATE64_COUNT,
            );
            mach_port_deallocate(task, thread);
        }
        mach_vm_deallocate(
            task,
            threads as u64,
            (thread_count as u64) * std::mem::size_of::<mach_port_t>() as u64,
        );
    }

    Ok(())
}

/// Installs a hardware breakpoint hook at a relative virtual address (RVA)
///
/// # Arguments
/// * `rva` - The relative virtual address to hook
/// * `replacement` - The address of the replacement function
///
/// # Returns
/// * `Result<Breakpoint, BrkHookError>` - The installed breakpoint or an error
pub unsafe fn install(rva: usize, replacement: usize) -> Result<Breakpoint, BrkHookError> {
    let base = crate::memory::image::get_image_base(crate::config::TARGET_IMAGE_NAME)
        .map_err(|_| BrkHookError::InitFailed)?;
    install_at_address(base + rva, replacement)
}

/// Represents an installed hardware breakpoint
pub struct Breakpoint {
    /// The address being watched
    target: usize,
}

impl Breakpoint {
    /// Removes the breakpoint
    ///
    /// # Returns
    /// * `Result<(), BrkHookError>` - Result indicating success or failure
    pub fn remove(self) -> Result<(), BrkHookError> {
        unsafe { remove_at_address(self.target) }
    }

    /// Returns the target address of the breakpoint
    #[inline]
    pub fn target(&self) -> usize {
        self.target
    }

    /// Calls the original function by temporarily disabling the breakpoint
    ///
    /// # Type Parameters
    /// * `T` - The function pointer type for the original function
    /// * `F` - The callback closure type
    /// * `R` - The return type
    ///
    /// # Arguments
    /// * `callback` - A closure that takes the original function pointer and returns a result
    ///
    /// # Returns
    /// * `R` - The result of the callback
    #[inline]
    pub unsafe fn call_original<T, F, R>(&self, callback: F) -> R
    where
        T: Copy,
        F: FnOnce(T) -> R,
    {
        let _ = suspend_self();
        let orig: T = std::mem::transmute_copy(&self.target);
        let res = callback(orig);
        let _ = resume_self();
        res
    }
}

/// Installs a hardware breakpoint hook at an absolute address
///
/// # Arguments
/// * `target` - The absolute address to hook
/// * `replacement` - The address of the replacement function
///
/// # Returns
/// * `Result<Breakpoint, BrkHookError>` - The installed breakpoint or an error
pub unsafe fn install_at_address(
    target: usize,
    replacement: usize,
) -> Result<Breakpoint, BrkHookError> {
    init_exception_handler()?;

    let mut manager = MANAGER.lock();

    if manager.active_count >= manager.hw_breakpoints as usize {
        return Err(BrkHookError::ExceedsHwBreakpoints(manager.hw_breakpoints));
    }

    manager.add_hook(target, replacement)?;
    apply_debug_state(&manager)?;

    #[cfg(dev_release)]
    logger::info(&format!("BrkHook: {:#x} → {:#x}", target, replacement));

    Ok(Breakpoint { target })
}

/// Removes a hardware breakpoint hook at an absolute address
///
/// # Arguments
/// * `target` - The absolute address where the hook is installed
///
/// # Returns
/// * `Result<(), BrkHookError>` - Result indicating success or failure
pub unsafe fn remove_at_address(target: usize) -> Result<(), BrkHookError> {
    let mut manager = MANAGER.lock();
    manager.remove_hook(target)?;
    apply_debug_state(&manager)?;
    Ok(())
}

/// Returns the number of active breakpoints
///
/// # Returns
/// * `usize` - The count of active breakpoints
pub fn active_count() -> usize {
    MANAGER.lock().active_count
}

/// Returns the maximum number of hardware breakpoints supported by the device
///
/// # Returns
/// * `i32` - The maximum number of breakpoints
pub fn max_breakpoints() -> i32 {
    MANAGER.lock().hw_breakpoints
}

/// Suspends the current thread (disables breakpoints for this thread)
///
/// Use this before calling the original function to avoid infinite recursion loops.
///
/// # Returns
/// * `Result<(), BrkHookError>` - Result indicating success or failure
pub unsafe fn suspend_self() -> Result<(), BrkHookError> {
    let thread = mach_thread_self();
    let state = ArmDebugState64::default();

    if thread_set_state(
        thread,
        ARM_DEBUG_STATE64,
        &state as *const _ as *mut u32,
        ARM_DEBUG_STATE64_COUNT,
    ) != KERN_SUCCESS
    {
        return Err(BrkHookError::SetStateFailed);
    }

    mach_port_deallocate(mach_task_self(), thread);
    Ok(())
}

/// Resumes the current thread (re-enables breakpoints for this thread)
///
/// Use this after calling the original function to re-arm the breakpoints.
///
/// # Returns
/// * `Result<(), BrkHookError>` - Result indicating success or failure
pub unsafe fn resume_self() -> Result<(), BrkHookError> {
    let thread = mach_thread_self();
    let mut state = ArmDebugState64::default();

    {
        let manager = MANAGER.lock();
        let mut bp_idx = 0;
        for hook in manager.hooks.iter().flatten() {
            if bp_idx >= manager.hw_breakpoints as usize {
                break;
            }
            state.bvr[bp_idx] = hook.old as u64;
            state.bcr[bp_idx] = BCR_ENABLE;
            bp_idx += 1;
        }
    }

    if thread_set_state(
        thread,
        ARM_DEBUG_STATE64,
        &state as *const _ as *mut u32,
        ARM_DEBUG_STATE64_COUNT,
    ) != KERN_SUCCESS
    {
        mach_port_deallocate(mach_task_self(), thread);
        return Err(BrkHookError::SetStateFailed);
    }

    mach_port_deallocate(mach_task_self(), thread);
    Ok(())
}
