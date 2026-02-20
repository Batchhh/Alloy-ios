//! # Hook Integrity Checking
//!
//! This module provides self-checksumming capabilities to detect if installed hooks
//! have been tampered with by anti-cheat or security tools.

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::logger;

use crate::memory::info::protection;

/// FNV-1a offset basis for 64-bit
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
/// FNV-1a prime for 64-bit
const FNV_PRIME: u64 = 0x100000001b3;

/// Errors that can occur during integrity checking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrityError {
    InvalidAddress,
    InvalidLength,
    MonitorAlreadyRunning,
    MonitorNotRunning,
}

/// Represents a checksum for an installed hook
#[derive(Clone, Debug)]
pub struct HookChecksum {
    /// Address of the hook redirect (the 16-byte branch at target)
    pub target: usize,
    /// Expected FNV-1a hash of the hook bytes
    pub expected_hash: u64,
    /// Length of bytes to check
    pub length: usize,
}

impl HookChecksum {
    /// Creates a new checksum for a hook at the given address
    ///
    /// # Arguments
    /// * `target` - The address where the hook redirect is installed
    /// * `len` - Number of bytes to checksum (typically 4 or 16)
    ///
    /// # Safety
    /// Caller must ensure target address is valid and readable for `len` bytes
    #[inline]
    pub unsafe fn new_unchecked(target: usize, len: usize) -> Self {
        let hash = Self::compute_unchecked(target, len);
        Self {
            target,
            expected_hash: hash,
            length: len,
        }
    }

    /// Creates a new checksum with validation
    pub fn new(target: usize, len: usize) -> Result<Self, IntegrityError> {
        if target == 0 || len == 0 {
            return Err(IntegrityError::InvalidAddress);
        }
        if len > 4096 {
            return Err(IntegrityError::InvalidLength);
        }

        // Perform a safe read test
        if !Self::is_readable(target, len) {
            return Err(IntegrityError::InvalidAddress);
        }

        Ok(unsafe { Self::new_unchecked(target, len) })
    }

    /// Checks if memory region is readable
    fn is_readable(addr: usize, len: usize) -> bool {
        match protection::get_region_info(addr) {
            Ok(info) => {
                let is_accessible = info.protection.is_readable();
                // Check if the region fully covers the requested range
                let end_addr = addr + len;
                let region_end = info.address + info.size;
                let in_range = end_addr <= region_end;

                is_accessible && in_range
            }
            Err(_) => false,
        }
    }

    /// Computes FNV-1a hash of bytes at the given address
    ///
    /// # Safety
    /// Caller must ensure target address is valid and readable for `len` bytes
    #[inline]
    unsafe fn compute_unchecked(target: usize, len: usize) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        let ptr = target as *const u8;

        // Use slice for better optimization
        let slice = std::slice::from_raw_parts(ptr, len);
        for &byte in slice {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }

        hash
    }

    /// Verifies that the hook bytes have not been modified
    ///
    /// # Returns
    /// * `bool` - `true` if intact, `false` if tampered
    #[inline]
    pub fn verify(&self) -> bool {
        unsafe {
            let current = Self::compute_unchecked(self.target, self.length);
            current == self.expected_hash
        }
    }

    /// Returns the target address
    #[inline]
    pub fn target(&self) -> usize {
        self.target
    }

    /// Updates the expected hash to current memory state
    pub fn update(&mut self) {
        unsafe {
            self.expected_hash = Self::compute_unchecked(self.target, self.length);
        }
    }
}

/// Global registry of hook checksums
static CHECKSUMS: Lazy<Mutex<HashMap<usize, HookChecksum>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Registers a checksum for a newly installed hook
///
/// # Arguments
/// * `target` - The hook target address
/// * `len` - Number of bytes in the hook redirect
///
/// Note: Automatically starts the background integrity monitor on first registration
pub fn register(target: usize, len: usize) -> Result<(), IntegrityError> {
    let checksum = HookChecksum::new(target, len)?;

    let mut checksums = CHECKSUMS.lock();
    let was_empty = checksums.is_empty();
    checksums.insert(target, checksum);
    drop(checksums); // Release lock early

    if was_empty && !is_monitor_running() {
        let _ = start_monitor(5000, Some(default_tamper_callback));
    }

    Ok(())
}

/// Removes a checksum when a hook is uninstalled
///
/// # Arguments
/// * `target` - The hook target address
pub fn unregister(target: usize) -> bool {
    CHECKSUMS.lock().remove(&target).is_some()
}

/// Verifies a single hook's integrity
///
/// # Arguments
/// * `target` - The hook target address
///
/// # Returns
/// * `Option<bool>` - `Some(true)` if intact, `Some(false)` if tampered, `None` if not registered
pub fn verify(target: usize) -> Option<bool> {
    CHECKSUMS.lock().get(&target).map(|c| c.verify())
}

/// Verifies all registered hooks and returns addresses of tampered ones
///
/// # Returns
/// * `Vec<usize>` - List of addresses where tampering was detected
pub fn verify_all() -> Vec<usize> {
    CHECKSUMS
        .lock()
        .values()
        .filter(|c| !c.verify())
        .map(|c| c.target)
        .collect()
}

/// Returns the number of registered checksums
pub fn count() -> usize {
    CHECKSUMS.lock().len()
}

/// Clears all registered checksums
pub fn clear() {
    CHECKSUMS.lock().clear();
}

/// Result of integrity verification
#[derive(Debug, Clone)]
pub struct IntegrityReport {
    /// Total hooks checked
    pub total: usize,
    /// Number of intact hooks
    pub intact: usize,
    /// Addresses of tampered hooks
    pub tampered: Vec<usize>,
}

impl IntegrityReport {
    /// Returns true if all hooks are intact
    #[inline]
    pub fn is_clean(&self) -> bool {
        self.tampered.is_empty()
    }

    /// Returns the percentage of intact hooks
    pub fn integrity_percentage(&self) -> f64 {
        if self.total == 0 {
            return 100.0;
        }
        (self.intact as f64 / self.total as f64) * 100.0
    }
}

/// Performs a full integrity scan and returns a detailed report
///
/// # Returns
/// * `IntegrityReport` - Detailed results of the scan
pub fn scan() -> IntegrityReport {
    let checksums = CHECKSUMS.lock();
    let total = checksums.len();
    let tampered: Vec<usize> = checksums
        .values()
        .filter(|c| !c.verify())
        .map(|c| c.target)
        .collect();
    let intact = total - tampered.len();

    IntegrityReport {
        total,
        intact,
        tampered,
    }
}

/// Monitor thread state
struct MonitorState {
    running: AtomicBool,
    handle: Mutex<Option<JoinHandle<()>>>,
}

static MONITOR_STATE: Lazy<Arc<MonitorState>> = Lazy::new(|| {
    Arc::new(MonitorState {
        running: AtomicBool::new(false),
        handle: Mutex::new(None),
    })
});

/// Callback type for when tampering is detected
pub type TamperCallback = fn(tampered: &[usize]);

/// Default callback that restores tampered hooks
///
/// When tampering is detected, this callback:
/// 1. Logs a warning (in dev builds)
/// 2. Re-writes the hook redirect bytes to restore functionality
/// 3. Updates the checksum with the new bytes
fn default_tamper_callback(tampered: &[usize]) {
    use super::hook::restore_hook_bytes;

    for &addr in tampered {
        #[cfg(feature = "dev_release")]
        logger::warning(&format!("Hook tampered at {:#x}, restoring...", addr));

        if restore_hook_bytes(addr) {
            // Update checksum in a single lock acquisition
            let mut checksums = CHECKSUMS.lock();
            if let Some(checksum) = checksums.get_mut(&addr) {
                checksum.update();
            }
            drop(checksums);

            #[cfg(feature = "dev_release")]
            logger::info(&format!("Hook restored at {:#x}", addr));
        } else {
            #[cfg(feature = "dev_release")]
            logger::error(&format!("Failed to restore hook at {:#x}", addr));
        }
    }
}

/// Starts a background thread that periodically verifies hook integrity
///
/// # Arguments
/// * `interval_ms` - How often to check (in milliseconds)
/// * `on_tamper` - Optional callback when tampering is detected
///
/// # Returns
/// * `Result<(), IntegrityError>` - Ok if monitor started, Err if already running
pub fn start_monitor(
    interval_ms: u64,
    on_tamper: Option<TamperCallback>,
) -> Result<(), IntegrityError> {
    let state = Arc::clone(&MONITOR_STATE);

    if state.running.swap(true, Ordering::SeqCst) {
        return Err(IntegrityError::MonitorAlreadyRunning);
    }

    let callback = on_tamper.unwrap_or(default_tamper_callback);
    let interval = Duration::from_millis(interval_ms);
    let state_clone = Arc::clone(&state);

    let handle = thread::spawn(move || {
        #[cfg(feature = "dev_release")]
        logger::info("Integrity monitor started");

        while state_clone.running.load(Ordering::Relaxed) {
            thread::sleep(interval);

            if count() == 0 {
                continue;
            }

            let tampered = verify_all();
            if !tampered.is_empty() {
                callback(&tampered);
            }
        }

        #[cfg(feature = "dev_release")]
        logger::info("Integrity monitor stopped");
    });

    *state.handle.lock() = Some(handle);
    Ok(())
}

/// Stops the background integrity monitor
///
/// # Returns
/// * `Result<(), IntegrityError>` - Ok if monitor was running and is now stopped
pub fn stop_monitor() -> Result<(), IntegrityError> {
    let state = Arc::clone(&MONITOR_STATE);

    if !state.running.swap(false, Ordering::SeqCst) {
        return Err(IntegrityError::MonitorNotRunning);
    }

    // Wait for thread to finish
    if let Some(handle) = state.handle.lock().take() {
        let _ = handle.join();
    }

    Ok(())
}

/// Checks if the background monitor is running
#[inline]
pub fn is_monitor_running() -> bool {
    MONITOR_STATE.running.load(Ordering::Relaxed)
}
