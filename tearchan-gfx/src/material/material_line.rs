use crate::material::{Material, MaterialProvider};
use std::ops::{Deref, DerefMut};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendComponent, BlendFactor, BlendOperation, BlendState,
    Buffer, BufferBindingType, BufferSize, ColorTargetState, ColorWrites, CompareFunction,
    DepthBiasState, DepthStencilState, Device, FragmentState, MultisampleState, PipelineLayout,
    PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderModule,
    ShaderStages, StencilState, TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexState, VertexStepMode,
};

pub struct MaterialLineParams<'a> {
    pub transform_buffer: &'a Buffer,
    pub color_format: TextureFormat,
    pub depth_format: Option<TextureFormat>,
    pub shader_module: Option<ShaderModule>,
}

pub struct MaterialLine {
    material: Material<MaterialLineProvider>,
}

impl MaterialLine {
    pub fn new(device: &Device, mut params: MaterialLineParams) -> Self {
        let shader_module = std::mem::take(&mut params.shader_module).unwrap_or_else(|| {
            device.create_shader_module(&wgpu::include_wgsl!("../../shaders/standard_line.wgsl"))
        });
        MaterialLine {
            material: Material::new(device, &params, MaterialLineProvider { shader_module }),
        }
    }
}

impl Deref for MaterialLine {
    type Target = Material<MaterialLineProvider>;

    fn deref(&self) -> &Self::Target {
        &self.material
    }
}

impl DerefMut for MaterialLine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.material
    }
}

pub struct MaterialLineProvider {
    shader_module: ShaderModule,
}

impl<'a> MaterialProvider<'a> for MaterialLineProvider {
    type Params = MaterialLineParams<'a>;

    fn create_bind_group_layout(&self, device: &Device, _params: &Self::Params) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(64),
                },
                count: None,
            }],
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
                topology: PrimitiveTopology::LineList,
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
            depth_stencil: params.depth_format.map(|format| DepthStencilState {
                format,
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
            entries: &[BindGroupEntry {
                binding: 0,
                resource: params.transform_buffer.as_entire_binding(),
            }],
            label: None,
        })
    }
}

fn create_vertex_buffers<'a>() -> [VertexBufferLayout<'a>; 2] {
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
            array_stride: 4 * std::mem::size_of::<f32>() as u64, // colors
            step_mode: VertexStepMode::Vertex,
            attributes: &[VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 0,
                shader_location: 1,
            }],
        },
    ]
}
