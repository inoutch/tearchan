use std::error::Error;
use std::path::PathBuf;
use tearchan_tools::assets::transfer_default_assets;

pub fn main() -> Result<(), Box<dyn Error>> {
    let target_dir = std::env::var("OUT_DIR")?;
    let workspace_dir = env!("CARGO_MANIFEST_DIR");

    let mut assets_src_path = PathBuf::new();
    assets_src_path.push(workspace_dir);
    assets_src_path.push("assets");

    let mut assets_dst_path = PathBuf::new();
    assets_dst_path.push(target_dir);
    assets_dst_path.push("../../../");
    assets_dst_path.push("assets");

    transfer_default_assets(&assets_src_path, &assets_dst_path)?;

    Ok(())
}
