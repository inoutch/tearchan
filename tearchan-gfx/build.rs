use shaderc::ShaderKind;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=shaders");

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
