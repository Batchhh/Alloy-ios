use objc2::rc::Retained;
use objc2_foundation::{NSUserDefaults, NSString};
// use objc2::ClassType;

pub struct Preferences;

impl Preferences {
    fn defaults() -> Retained<NSUserDefaults> {
        NSUserDefaults::standardUserDefaults()
    }

    fn key(key: &str) -> Retained<NSString> {
        NSString::from_str(&format!("modmenu.{}", key))
    }

    pub fn get_bool(key: &str) -> bool {
        let defaults = Self::defaults();
        let key = Self::key(key);
        defaults.boolForKey(&key)
    }

    pub fn set_bool(key: &str, value: bool) {
        let defaults = Self::defaults();
        let key = Self::key(key);
        defaults.setBool_forKey(value, &key);
    }

    pub fn get_float(key: &str) -> f32 {
        let defaults = Self::defaults();
        let key = Self::key(key);
        defaults.floatForKey(&key)
    }

    pub fn set_float(key: &str, value: f32) {
        let defaults = Self::defaults();
        let key = Self::key(key);
        defaults.setFloat_forKey(value, &key);
    }

    pub fn get_string(key: &str) -> String {
        let defaults = Self::defaults();
        let key = Self::key(key);
        let val = defaults.stringForKey(&key);
        val.map(|s| s.to_string()).unwrap_or_default()
    }

    pub fn set_string(key: &str, value: &str) {
        let defaults = Self::defaults();
        let key = Self::key(key);
        let val = NSString::from_str(value);
        unsafe { defaults.setObject_forKey(Some(&val), &key) };
    }
}
