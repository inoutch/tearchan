use crate::core::graphic::hal::backend::FixedApi;
use crate::core::graphic::shader::attribute::Attribute;
use crate::core::graphic::shader::shader_program::ShaderProgram;
use crate::core::graphic::shader::shader_source::ShaderSource;

pub struct Standard2DShaderProgram {
    shader_program: ShaderProgram,
}

impl Standard2DShaderProgram {
    pub fn new(api: &FixedApi) -> Self {
        let shader_source = ShaderSource::new(
            include_bytes!("../../../../target/data/shaders/standard.vert"),
            include_bytes!("../../../../target/data/shaders/standard.frag"),
        )
        .unwrap();

        let attributes = create_2d_attributes();
        let shader = api.create_shader(shader_source, attributes);
        let shader_program = ShaderProgram::new(shader, vec![]);
        Standard2DShaderProgram { shader_program }
    }
}

fn create_2d_attributes() -> Vec<Attribute> {
    vec![
        Attribute {
            attribute_desc: gfx_hal::pso::AttributeDesc {
                // position
                location: 0,
                binding: 0,
                element: gfx_hal::pso::Element {
                    format: gfx_hal::format::Format::Rg32Sfloat,
                    offset: 0,
                },
            },
            stride: 3,
        },
        Attribute {
            attribute_desc: gfx_hal::pso::AttributeDesc {
                // color
                location: 1,
                binding: 1,
                element: gfx_hal::pso::Element {
                    format: gfx_hal::format::Format::Rg32Sfloat,
                    offset: 0,
                },
            },
            stride: 4,
        },
        Attribute {
            attribute_desc: gfx_hal::pso::AttributeDesc {
                // texcoord
                location: 2,
                binding: 2,
                element: gfx_hal::pso::Element {
                    format: gfx_hal::format::Format::Rg32Sfloat,
                    offset: 0,
                },
            },
            stride: 2,
        },
    ]
}
