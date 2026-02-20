//! Thread suspension utilities for safe memory patching

use std::ptr;

use mach2::{
    kern_return::KERN_SUCCESS,
    mach_init::mach_thread_self,
    mach_types::{thread_act_array_t, thread_act_t},
    message::mach_msg_type_number_t,
    port::mach_port_t,
    task::task_threads,
    thread_act::{thread_resume, thread_suspend},
    traps::mach_task_self,
    vm::mach_vm_deallocate,
    vm_types::mach_vm_size_t,
};

use thiserror::Error;

#[derive(Error, Debug)]
/// Errors that can occur during thread manipulation
pub enum ThreadError {
    /// Failed to retrieve the list of threads for the current task
    #[error("Failed to get task threads (kern_return: {0})")]
    TaskThreadsFailed(i32),
}

/// Suspends all other threads in the current task to prevent race conditions during memory modification.
///
/// This is critical when applying patches or hooks to ensure that no thread is executing the code
/// being modified.
///
/// # Returns
/// * `Result<Vec<mach_port_t>, ThreadError>` - A list of suspended thread ports (needed to resume them later)
pub unsafe fn suspend_other_threads() -> Result<Vec<mach_port_t>, ThreadError> {
    let mut thread_list: thread_act_array_t = ptr::null_mut();
    let mut thread_count: mach_msg_type_number_t = 0;

    let kret = task_threads(mach_task_self(), &mut thread_list, &mut thread_count);
    if kret != KERN_SUCCESS {
        return Err(ThreadError::TaskThreadsFailed(kret));
    }

    let this_thread = mach_thread_self();
    let mut suspended_threads = Vec::with_capacity(thread_count as usize);

    let threads = std::slice::from_raw_parts(thread_list, thread_count as usize);

    for &thread in threads {
        if thread != this_thread && thread_suspend(thread) == KERN_SUCCESS {
            suspended_threads.push(thread);
        }
    }

    mach_vm_deallocate(
        mach_task_self(),
        thread_list as u64,
        (thread_count as mach_vm_size_t) * (std::mem::size_of::<thread_act_t>() as mach_vm_size_t),
    );

    Ok(suspended_threads)
}

/// Resumes threads that were previously suspended
///
/// # Arguments
/// * `threads` - A slice of thread ports to resume (returned by `suspend_other_threads`)
pub unsafe fn resume_threads(threads: &[mach_port_t]) {
    for &thread in threads {
        thread_resume(thread);
    }
}
