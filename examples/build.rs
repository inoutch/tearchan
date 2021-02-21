use shaderc::ShaderKind;
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

    build_shaders()?;

    Ok(())
}

fn build_shaders() -> Result<(), Box<dyn Error>> {
    let shader_gen_path = "../target/shaders/";
    std::fs::create_dir_all(shader_gen_path)?;

    for entry in std::fs::read_dir("shaders")? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            let in_path = entry.path();
            let shader_type =
                in_path
                    .extension()
                    .and_then(|ext| match ext.to_string_lossy().as_ref() {
                        "vert" => Some(ShaderKind::Vertex),
                        "frag" => Some(ShaderKind::Fragment),
                        _ => None,
                    });
            if let Some(shader_type) = shader_type {
                let source = std::fs::read_to_string(&in_path)?;
                let file_name = in_path.file_name().unwrap().to_string_lossy();

                let mut compiler = shaderc::Compiler::new().ok_or("shaderc not found!")?;
                let compiled_artifact = compiler
                    .compile_into_spirv(&source, shader_type, &file_name, "main", None)
                    .unwrap();
                let compiled_bytes = compiled_artifact.as_binary_u8();

                let out_path = format!("{}{}.spv", shader_gen_path, file_name);

                std::fs::write(&out_path, &compiled_bytes)?;
            }
        }
    }
    Ok(())
}
