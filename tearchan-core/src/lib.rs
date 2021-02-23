#[cfg(any(target_os = "ios"))]
#[macro_use]
extern crate objc;

pub mod io;
pub mod sync;
