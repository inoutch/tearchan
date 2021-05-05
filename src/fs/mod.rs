use futures::AsyncReadExt;
use once_cell::sync::Lazy;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::{RwLock, RwLockReadGuard};
use tearchan_core::io::file::FileUtil;
use thiserror::Error;

static FILE_UTIL: Lazy<RwLock<FileUtil>> = Lazy::new(|| RwLock::new(FileUtil::new().unwrap()));

pub fn file_util() -> RwLockReadGuard<'static, FileUtil> {
    FILE_UTIL.read().unwrap()
}

pub fn configure_file_util<P: AsRef<Path>>(assets_path: P) {
    let mut file_util = FILE_UTIL.write().expect("Using file_util");
    file_util.set_assets_path(assets_path);
}

pub async fn read_bytes_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, Box<dyn Error>> {
    #[cfg(target_os = "android")]
    {
        use tearchan_core::io::file::android::{FileReadFuture, ASSETS_SCHEME};
        use tearchan_util::string::get_path_without_prefix;
        if let Some(path) = get_path_without_prefix(path.as_ref(), ASSETS_SCHEME) {
            return Ok(FileReadFuture::read_bytes_from_file(Path::new(&path)).await?);
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        use tearchan_core::io::net::web::get_request_as_binaries;
        let path_str = path.as_ref().to_str().unwrap();
        let bytes = get_request_as_binaries(path_str)
            .await
            .map_err(|_| FileReadError::FileNotExists)?;
        Ok(bytes)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut bytes = vec![];
        let mut file = async_std::fs::File::open(path.as_ref()).await?;
        file.read_to_end(&mut bytes).await?;
        Ok(bytes)
    }
}

#[derive(Error, Debug)]
pub enum FileReadError {
    #[error("the file is not exits")]
    FileNotExists,
}

pub fn write_bytes_to_file<P: AsRef<Path>>(path: P, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(path)?;
    file.write_all(bytes)?;
    file.flush()?;
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::fs::file_util;

    #[test]
    fn test_multi_thread_file_util() {
        let path1 = file_util().assets_path().to_str().map(|x| x.to_string());
        let path2 =
            std::thread::spawn(|| file_util().assets_path().to_str().map(|x| x.to_string()))
                .join()
                .unwrap();
        assert_eq!(path1, path2);
    }
}
