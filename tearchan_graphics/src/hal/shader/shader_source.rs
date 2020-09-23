use std::io::Cursor;

pub struct ShaderSource {
    pub spirv_vert_source: Vec<u32>,
    pub spirv_frag_source: Vec<u32>,
}

impl ShaderSource {
    pub fn new(vert_binaries: &[u8], frag_binaries: &[u8]) -> std::io::Result<ShaderSource> {
        let spirv_vert_source = gfx_auxil::read_spirv(Cursor::new(vert_binaries))?;
        let spirv_frag_source = gfx_auxil::read_spirv(Cursor::new(frag_binaries))?;
        Ok(ShaderSource {
            spirv_vert_source,
            spirv_frag_source,
        })
    }
}
