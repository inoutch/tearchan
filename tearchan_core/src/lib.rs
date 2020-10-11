#![feature(get_mut_unchecked)]
#[cfg(any(target_os = "macos", all(target_os = "ios", target_arch = "aarch64")))]
#[macro_use]
extern crate objc;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate downcast_rs;

pub mod controller;
pub mod file;
pub mod game;
pub mod scene;
pub mod ui;
