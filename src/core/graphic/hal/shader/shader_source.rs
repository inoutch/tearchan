use std::io::Cursor;

pub struct ShaderSource {
    pub spirv_vert_source: Vec<u32>,
    pub spirv_frag_source: Vec<u32>,
}

impl ShaderSource {
    pub fn new(vert_binaries: &[u8], frag_binaries: &[u8]) -> Option<ShaderSource> {
        let spirv_vert_source = match gfx_auxil::read_spirv(Cursor::new(vert_binaries)) {
            Ok(x) => x,
            Err(_) => {
                return None;
            }
        };
        let spirv_frag_source = match gfx_auxil::read_spirv(Cursor::new(frag_binaries)) {
            Ok(x) => x,
            Err(_) => {
                return None;
            }
        };
        Some(ShaderSource {
            spirv_vert_source,
            spirv_frag_source,
        })
    }
}
