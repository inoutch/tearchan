use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

#[cfg(target_os = "android")]
pub mod android;
#[cfg(any(target_os = "macos", target_os = "ios"))]
mod objc;
#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
const ASSETS_DIR: &str = "assets";

pub struct FileUtil {
    assets_path: PathBuf,
}

impl FileUtil {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
        let assets_path = {
            use std::env::current_exe;
            let mut asset_path_buf = current_exe()?
                .parent()
                .ok_or(FileUtilError::NoParent)?
                .to_path_buf();
            asset_path_buf.push(ASSETS_DIR);
            asset_path_buf
        };
        #[cfg(target_os = "android")]
        let assets_path = PathBuf::from(android::ASSETS_SCHEME);

        #[cfg(target_arch = "wasm32")]
        let assets_path = PathBuf::from(format!("/{:?}", ASSETS_DIR));

        Ok(FileUtil { assets_path })
    }

    pub fn assets_path(&self) -> &Path {
        self.assets_path.as_path()
    }

    pub fn writable_path(&self) {}

    pub fn set_assets_path<P: AsRef<Path>>(&mut self, assets_path: P) {
        self.assets_path = PathBuf::from(assets_path.as_ref());
    }
}

#[derive(Debug)]
pub enum FileUtilError {
    NoParent,
}

impl Display for FileUtilError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for FileUtilError {}
