use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

#[cfg(target_os = "android")]
pub mod android;
#[cfg(any(target_os = "ios"))]
mod objc;
#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
const ASSETS_DIR: &str = "assets";

const WRITABLE_DIR: &str = "data";

pub struct FileUtil {
    assets_path: PathBuf,
    writable_path: PathBuf,
}

impl FileUtil {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let (assets_path, writable_path) = Self::create_paths()?;

        let _ = std::fs::create_dir(&writable_path);

        Ok(FileUtil {
            assets_path,
            writable_path,
        })
    }

    pub fn assets_path(&self) -> &Path {
        self.assets_path.as_path()
    }

    pub fn writable_path(&self) -> &Path {
        self.writable_path.as_path()
    }

    pub fn set_assets_path<P: AsRef<Path>>(&mut self, assets_path: P) {
        self.assets_path = PathBuf::from(assets_path.as_ref());
    }

    fn create_paths() -> Result<(PathBuf, PathBuf), Box<dyn Error>> {
        #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
        {
            use std::env::current_exe;
            let exec_path = current_exe()?
                .parent()
                .ok_or(FileUtilError::NoParent)?
                .to_path_buf();
            let assets_path = {
                let mut asset_path_buf = exec_path.clone();
                asset_path_buf.push(ASSETS_DIR);
                asset_path_buf
            };
            #[cfg(target_os = "ios")]
            let writable_path = {
                let mut writable_path = objc::create_writable_path();
                writable_path.push(WRITABLE_DIR);
                writable_path
            };
            #[cfg(not(target_os = "ios"))]
            let writable_path = {
                let mut writable_path = exec_path;
                writable_path.push(WRITABLE_DIR);
                writable_path
            };
            Ok((assets_path, writable_path))
        }

        #[cfg(target_os = "android")]
        {
            let assets_path = PathBuf::from(android::ASSETS_SCHEME);
            let writable_path = {
                let mut writable_path = PathBuf::from(android::get_writable_path());
                writable_path.push(WRITABLE_DIR);
                writable_path
            };
            Ok((assets_path, writable_path))
        }

        #[cfg(target_arch = "wasm32")]
        {
            let assets_path = PathBuf::from(format!("/{:?}", ASSETS_DIR));
            let writable_path = PathBuf::from(web::WRITABLE_SCHEME);
            Ok((assets_path, writable_path))
        }
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
