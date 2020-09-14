use crate::renderer::RenderId;
use nalgebra_glm::{Mat4, Vec3, Vec4};

pub enum RenderCommandValue {
    Vec3 { v: Vec3 },
    Vec4 { v: Vec4 },
    U32 { v: u32 },
}

pub enum RenderCommandVertices {
    F32 { vertices: Vec<f32> },
    U32 { vertices: Vec<u32> },
}

pub enum RenderCommandTransform {
    Mat4 { m: Mat4 },
}

pub enum RenderCommand {
    Add {
        id: RenderId,
        renderer_type: u32,
        vertices: Vec<RenderCommandVertices>,
        order: Option<i32>,
    },
    Remove {
        id: RenderId,
    },
    Transform {
        id: RenderId,
        attribute: u32,
        transform: RenderCommandTransform,
    },
    Copy {
        id: RenderId,
        vertices: Vec<RenderCommandVertices>,
    },
    CopyForEach {
        id: RenderId,
        value: RenderCommandValue,
    },
}
