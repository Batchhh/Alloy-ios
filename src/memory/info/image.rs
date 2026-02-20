//! Dynamic library image lookup utilities

use crate::utils::logger;
use std::ffi::CStr;

use mach2::dyld::{
    _dyld_get_image_header, _dyld_get_image_name, _dyld_get_image_vmaddr_slide, _dyld_image_count,
};
use thiserror::Error;

#[derive(Error, Debug)]
/// Errors that can occur during image lookup
pub enum ImageError {
    /// The specified image was not found
    #[error("Image not found: {0}")]
    NotFound(String),
}

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;

static IMAGE_CACHE: Lazy<RwLock<HashMap<String, usize>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Retrieves the base address of a loaded image by name
///
/// # Arguments
/// * `image_name` - The name of the image (or a substring of it)
///
/// # Returns
/// * `Result<usize, ImageError>` - The base address of the image or an error
pub fn get_image_base(image_name: &str) -> Result<usize, ImageError> {
    {
        let cache = IMAGE_CACHE.read();
        if let Some(&base) = cache.get(image_name) {
            return Ok(base);
        }
    }

    unsafe {
        let count = _dyld_image_count();

        for i in 0..count {
            let name_ptr = _dyld_get_image_name(i);
            if name_ptr.is_null() {
                continue;
            }

            let name = CStr::from_ptr(name_ptr).to_string_lossy();
            if name.contains(image_name) {
                let header = _dyld_get_image_header(i);
                let slide = _dyld_get_image_vmaddr_slide(i);

                #[cfg(dev_release)]
                logger::info(&format!(
                    "Found image: {} (Index: {}, Base: {:p}, Slide: {:#x})",
                    name, i, header, slide
                ));

                let base = header as usize;

                IMAGE_CACHE.write().insert(image_name.to_string(), base);

                return Ok(base);
            }
        }
    }

    #[cfg(dev_release)]
    logger::warning(&format!("Image not found: {}", image_name));
    Err(ImageError::NotFound(image_name.to_string()))
}
