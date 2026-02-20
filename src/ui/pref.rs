//! Preferences

use objc2::rc::Retained;
use objc2_foundation::{NSString, NSUserDefaults};

/// Manages persistent user preferences using `NSUserDefaults`
pub struct Preferences;

impl Preferences {
    fn defaults() -> Retained<NSUserDefaults> {
        NSUserDefaults::standardUserDefaults()
    }

    fn key(key: &str) -> Retained<NSString> {
        NSString::from_str(&format!("modmenu.{}", key))
    }

    /// Gets a boolean preference
    ///
    /// # Arguments
    /// * `key` - The preference key (without prefix)
    ///
    /// # Returns
    /// * `bool` - The stored value, or false if not set
    pub fn get_bool(key: &str) -> bool {
        let defaults = Self::defaults();
        let key = Self::key(key);
        defaults.boolForKey(&key)
    }

    /// Sets a boolean preference
    ///
    /// # Arguments
    /// * `key` - The preference key
    /// * `value` - The value to store
    pub fn set_bool(key: &str, value: bool) {
        let defaults = Self::defaults();
        let key = Self::key(key);
        defaults.setBool_forKey(value, &key);
    }

    /// Gets a float preference
    ///
    /// # Arguments
    /// * `key` - The preference key
    ///
    /// # Returns
    /// * `f32` - The stored value, or 0.0 if not set
    pub fn get_float(key: &str) -> f32 {
        let defaults = Self::defaults();
        let key = Self::key(key);
        defaults.floatForKey(&key)
    }

    /// Sets a float preference
    ///
    /// # Arguments
    /// * `key` - The preference key
    /// * `value` - The value to store
    pub fn set_float(key: &str, value: f32) {
        let defaults = Self::defaults();
        let key = Self::key(key);
        defaults.setFloat_forKey(value, &key);
    }

    /// Gets a string preference
    ///
    /// # Arguments
    /// * `key` - The preference key
    ///
    /// # Returns
    /// * `String` - The stored string, or empty string if not set
    pub fn get_string(key: &str) -> String {
        let defaults = Self::defaults();
        let key = Self::key(key);
        let val = defaults.stringForKey(&key);
        val.map(|s| s.to_string()).unwrap_or_default()
    }

    /// Sets a string preference
    ///
    /// # Arguments
    /// * `key` - The preference key
    /// * `value` - The string value to store
    pub fn set_string(key: &str, value: &str) {
        let defaults = Self::defaults();
        let key = Self::key(key);
        let val = NSString::from_str(value);
        unsafe { defaults.setObject_forKey(Some(&val), &key) };
    }

    /// Gets an integer preference
    ///
    /// # Arguments
    /// * `key` - The preference key
    ///
    /// # Returns
    /// * `i32` - The stored value, or 0 if not set
    pub fn get_int(key: &str) -> i32 {
        let defaults = Self::defaults();
        let key = Self::key(key);
        defaults.integerForKey(&key) as i32
    }

    /// Sets an integer preference
    ///
    /// # Arguments
    /// * `key` - The preference key
    /// * `value` - The value to store
    pub fn set_int(key: &str, value: i32) {
        let defaults = Self::defaults();
        let key = Self::key(key);
        defaults.setInteger_forKey(value as isize, &key);
    }
}
