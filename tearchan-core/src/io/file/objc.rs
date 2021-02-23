use objc_foundation::{INSString, NSArray, NSString};
use objc::runtime::Object;

#[allow(improper_ctypes)]
#[allow(dead_code)]
extern "C" {
    pub fn NSSearchPathForDirectoriesInDomains(
        directory: std::os::raw::c_ulong,
        domain_mask: std::os::raw::c_ulong,
        expand_tilde: bool,
    ) -> *mut NSArray<*mut NSString>;
}

pub fn create_resource_path() -> String {
    unsafe {
        let bundle_cls = class!(NSBundle);
        let main_bundle: &mut Object = msg_send![bundle_cls, mainBundle];
        let resource_path: &mut NSString = msg_send![main_bundle, resourcePath]; // NSString
        resource_path.as_str().to_string()
    }
}

pub fn create_writable_path() -> String {
    unsafe {
        let directories = NSSearchPathForDirectoriesInDomains(9, 1, true);
        let first_object: &mut NSString = msg_send![directories, firstObject];
        first_object.as_str().to_string()
    }
}
