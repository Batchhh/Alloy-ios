//! Symbol resolution and caching utilities

use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::ffi::CString;
use thiserror::Error;

#[derive(Error, Debug)]
/// Errors that can occur during symbol resolution
pub enum SymbolError {
    /// The specified symbol was not found
    #[error("Symbol not found: {0}")]
    NotFound(String),
    /// Failed to convert the symbol name to a CString
    #[error("CString error")]
    StringError,
}

static CACHE: Lazy<DashMap<String, usize>> = Lazy::new(|| DashMap::new());

/// Resolves a symbol to its address using dlsym
///
/// # Arguments
/// * `symbol` - The name of the symbol to resolve (e.g., "MGCopyAnswer")
///
/// # Returns
/// * `Result<usize, SymbolError>` - The address of the symbol or an error
pub fn resolve_symbol(symbol: &str) -> Result<usize, SymbolError> {
    if let Some(entry) = CACHE.get(symbol) {
        return Ok(*entry);
    }

    let c_str = CString::new(symbol).map_err(|_| SymbolError::StringError)?;
    unsafe {
        let addr_ptr = libc::dlsym(libc::RTLD_DEFAULT, c_str.as_ptr());
        if addr_ptr.is_null() {
            return Err(SymbolError::NotFound(symbol.into()));
        }
        let addr = addr_ptr as usize;
        CACHE.insert(symbol.into(), addr);
        Ok(addr)
    }
}

/// Manually caches a symbol address
///
/// Use this if you have resolved a symbol via other means and want to store it for future lookups.
///
/// # Arguments
/// * `s` - The symbol name
/// * `a` - The symbol address
pub fn cache_symbol(s: &str, a: usize) {
    CACHE.insert(s.into(), a);
}

/// Clears the symbol cache
pub fn clear_cache() {
    CACHE.clear();
}
