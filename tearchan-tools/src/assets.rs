use std::error::Error;
use std::fmt::Debug;
use std::option::Option::Some;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn transfer_default_assets<P1, P2>(src: &P1, dst: &P2) -> Result<(), Box<dyn Error>>
where
    P1: AsRef<Path>,
    P2: AsRef<Path> + Debug,
{
    transfer_assets(src, dst, "_")
}

pub fn transfer_assets<P1, P2>(
    src: &P1,
    dst: &P2,
    ignore_prefix: &str,
) -> Result<(), Box<dyn Error>>
where
    P1: AsRef<Path>,
    P2: AsRef<Path> + Debug,
{
    let _ = std::fs::create_dir(dst);
    let dst = PathBuf::from(dst.as_ref()).canonicalize()?;
    let src = PathBuf::from(src.as_ref()).canonicalize()?;

    for entry in WalkDir::new(src.as_path())
        .into_iter()
        .filter_map(|x| x.ok())
        .filter(|x| {
            x.file_name()
                .to_str()
                .map_or(false, |x| !x.starts_with(ignore_prefix))
        })
    {
        let mut dst_full_path = PathBuf::new();
        dst_full_path.push(dst.as_path());
        dst_full_path.push(match pathdiff::diff_paths(entry.path(), src.as_path()) {
            None => continue,
            Some(x) => x,
        });
        if entry.file_type().is_dir() {
            let _ = std::fs::create_dir(dst_full_path);
        } else if entry.file_type().is_file() {
            std::fs::copy(entry.path(), dst_full_path)?;
        }
    }
    Ok(())
}
