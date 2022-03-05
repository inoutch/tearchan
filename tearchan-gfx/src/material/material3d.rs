use crate::material::{Material, MaterialProvider};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendComponent, BlendFactor,
    BlendOperation, BlendState, Buffer, BufferBindingType, BufferSize, ColorTargetState,
    ColorWrites, CompareFunction, DepthBiasState, DepthStencilState, Device, Face, FragmentState,
    MultisampleState, PipelineLayout, PrimitiveState, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, Sampler, ShaderModule, ShaderStages, StencilState, TextureFormat,
    TextureSampleType, TextureView, TextureViewDimension, VertexAttribute, VertexBufferLayout,
    VertexFormat, VertexState, VertexStepMode,
};

pub struct Material3DParams<'a> {
    pub transform_buffer: &'a Buffer,
    pub light_position_buffer: &'a Buffer,
    pub light_ambient_strength_buffer: &'a Buffer,
    pub light_color_buffer: &'a Buffer,
    pub texture_view: &'a TextureView,
    pub sampler: &'a Sampler,
    pub color_format: TextureFormat,
    pub depth_format: TextureFormat,
    pub shader_module: Option<ShaderModule>,
}

pub struct Material3D {
    material: Material<Material3DProvider>,
}

impl Material3D {
    pub fn new(device: &Device, mut params: Material3DParams) -> Self {
        let shader_module = std::mem::take(&mut params.shader_module).unwrap_or_else(|| {
            device.create_shader_module(&wgpu::include_wgsl!("../../shaders/standard_3d.wgsl"))
        });

        Material3D {
            material: Material::new(device, &params, Material3DProvider { shader_module }),
        }
    }

    pub fn bind<'a>(&'a self, rpass: &mut RenderPass<'a>) {
        self.material.bind(rpass);
    }
}

pub struct Material3DProvider {
    shader_module: ShaderModule,
}

impl<'a> MaterialProvider<'a> for Material3DProvider {
    type Params = Material3DParams<'a>;

    fn create_bind_group_layout(&self, device: &Device, _params: &Self::Params) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(64),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler {
                        comparison: false,
                        filtering: true,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    fn create_pipeline(
        &self,
        device: &Device,
        params: &Self::Params,
        pipeline_layout: &PipelineLayout,
    ) -> RenderPipeline {
        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(pipeline_layout),
            primitive: PrimitiveState {
                cull_mode: Some(Face::Back),
                ..Default::default()
            },
            vertex: VertexState {
                module: &self.shader_module,
                entry_point: "vs_main",
                buffers: &create_vertex_buffers(),
            },
            fragment: Some(FragmentState {
                module: &self.shader_module,
                entry_point: "fs_main",
                targets: &[ColorTargetState {
                    format: params.color_format,
                    blend: Some(BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::SrcAlpha,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        },
                        alpha: BlendComponent {
                            src_factor: BlendFactor::One,
                            dst_factor: BlendFactor::One,
                            operation: BlendOperation::Max,
                        },
                    }),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            depth_stencil: Some(DepthStencilState {
                format: params.depth_format,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState::default(),
        })
    }

    fn create_bind_group(
        &self,
        device: &Device,
        params: &Self::Params,
        layout: &BindGroupLayout,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: params.transform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(params.texture_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(params.sampler),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: params.light_ambient_strength_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: params.light_color_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: params.light_position_buffer.as_entire_binding(),
                },
            ],
            label: None,
        })
    }
}

fn create_vertex_buffers<'a>() -> [VertexBufferLayout<'a>; 4] {
    [
        VertexBufferLayout {
            array_stride: 3 * std::mem::size_of::<f32>() as u64, // positions
            step_mode: VertexStepMode::Vertex,
            attributes: &[VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            }],
        },
        VertexBufferLayout {
            array_stride: 2 * std::mem::size_of::<f32>() as u64, // texcoords
            step_mode: VertexStepMode::Vertex,
            attributes: &[VertexAttribute {
                format: VertexFormat::Float32x2,
                offset: 0,
                shader_location: 1,
            }],
        },
        VertexBufferLayout {
            array_stride: 4 * std::mem::size_of::<f32>() as u64, // colors
            step_mode: VertexStepMode::Vertex,
            attributes: &[VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 0,
                shader_location: 2,
            }],
        },
        VertexBufferLayout {
            array_stride: 3 * std::mem::size_of::<f32>() as u64, // normals
            step_mode: VertexStepMode::Vertex,
            attributes: &[VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: 0,
                shader_location: 3,
            }],
        },
    ]
}
