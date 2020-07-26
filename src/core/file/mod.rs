use std::env::current_exe;
use std::path::PathBuf;

pub struct File {
    resource_path: String,
    writable_path: String,
}

impl File {
    pub fn new(resource_path: Option<String>, writable_path: Option<String>) -> Self {
        File {
            resource_path: resource_path.unwrap_or_else(create_default_resource_path),
            writable_path: writable_path.unwrap_or_else(create_default_writable_path),
        }
    }

    pub fn resource_path(&self, path: &str) -> Option<String> {
        let mut ret = PathBuf::new();
        ret.push(&self.resource_path);
        ret.push(path);
        ret.to_str().map(|x| x.to_string())
    }

    pub fn writable_path(&self, path: &str) -> Option<String> {
        let mut ret = PathBuf::new();
        ret.push(&self.writable_path);
        ret.push(path);
        ret.to_str().map(|x| x.to_string())
    }
}

fn create_default_resource_path() -> String {
    let mut path = PathBuf::new();
    let bin_path = current_exe()
        .expect("failed to get current exec")
        .parent()
        .expect("failed to get parent path")
        .to_str()
        .expect("")
        .to_string();

    path.push(bin_path);
    path.push("assets");
    path.to_str().unwrap().to_string()
}

fn create_default_writable_path() -> String {
    let mut path = PathBuf::new();
    if cfg!(all(target_os = "ios", target_arch = "aarch64")) {
        path.push(ios::create_writable_path());
    }
    path.to_str().unwrap().to_string()
}

// #[cfg(all(target_os = "ios", target_arch = "aarch64"))]
#[cfg(any(target_os = "macos", all(target_os = "ios", target_arch = "aarch64")))]
mod ios {
    use objc_foundation::{INSString, NSArray, NSString};

    #[allow(improper_ctypes)]
    #[allow(dead_code)]
    extern "C" {
        pub fn NSSearchPathForDirectoriesInDomains(
            directory: std::os::raw::c_ulong,
            domain_mask: std::os::raw::c_ulong,
            expand_tilde: bool,
        ) -> *mut NSArray<*mut NSString>;
    }

    /*fn create_resource_path() -> String {
        unsafe {
            let bundle_cls = class!(NSBundle);
            let main_bundle: &mut Object = msg_send![bundle_cls, mainBundle];
            let resource_path: &mut NSString = msg_send![main_bundle, resourcePath]; // NSString
            resource_path.as_str().to_string()
        }
    }*/

    pub fn create_writable_path() -> String {
        unsafe {
            let directories = NSSearchPathForDirectoriesInDomains(9, 1, true);
            let first_object: &mut NSString = msg_send![directories, firstObject];
            first_object.as_str().to_string()
        }
    }
}
