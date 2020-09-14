#[cfg(any(target_os = "macos", all(target_os = "ios", target_arch = "aarch64")))]
#[macro_use]
extern crate objc;

pub mod file;
