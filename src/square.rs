use bytemuck::{Pod, Zeroable};
use glam::Vec2;
use wgpu::{Device, RenderPipelineDescriptor, RenderPipeline, PipelineLayoutDescriptor, VertexState, ShaderModule, ShaderModuleDescriptor, VertexBufferLayout, VertexStepMode, BufferAddress, PrimitiveState, DepthStencilState, MultisampleState, FragmentState, ColorTargetState, TextureFormat, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStages, BindGroupDescriptor, BindGroupEntry, BufferDescriptor, BufferUsages, util::{DeviceExt, BufferInitDescriptor}};
use std::{error::Error, borrow::Cow, mem, ops::Deref};

use crate::platform;

pub const SQUARE_SIZE: f32 = 8.0; // pixels

const SQUARE_GEOM: [SquareVertexRaw; 4] = [
    SquareVertexRaw::const_from(SquareVertex {
        relpos: Vec2::new(1.0 / 2., 1.0 / 2.),
        uv: Vec2::new(1.0, 1.0),
    }),
    SquareVertexRaw::const_from(SquareVertex {
        relpos: Vec2::new(-1.0 / 2., 1.0 / 2.),
        uv: Vec2::new(-1.0, 1.0),
    }),
    SquareVertexRaw::const_from(SquareVertex {
        relpos: Vec2::new(1.0 / 2., -1.0 / 2.),
        uv: Vec2::new(1.0, -1.0),
    }),
    SquareVertexRaw::const_from(SquareVertex {
        relpos: Vec2::new(-1.0 / 2., -1.0 / 2.),
        uv: Vec2::new(-1.0, -1.0),
    }),
];
// 1--0
// |\ |
// | \|
// 3--2
pub const SQUARE_INDX: [u16; 4] = [0, 1, 2, 3];

struct SquareVertex {
    relpos: Vec2,
    uv: Vec2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct SquareVertexRaw { pos_uv: [f32; 4] }

impl SquareVertexRaw {
    const fn const_from(value: SquareVertex) -> Self {
        Self { pos_uv: [value.relpos.x, value.relpos.y, value.uv.x, value.uv.y] }
    }
}

impl From<SquareVertex> for SquareVertexRaw {
    fn from(value: SquareVertex) -> Self {
        Self { pos_uv: [value.relpos.x, value.relpos.y, value.uv.x, value.uv.y] }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SquareInstance {
    pub pos: Vec2,
    pub hue: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct SquareInstanceRaw { pos_hue: [f32; 3] }

impl From<SquareInstance> for SquareInstanceRaw {
    fn from(value: SquareInstance) -> Self {
        Self { pos_hue: [value.pos.x, value.pos.y, value.hue] }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct SquareUniforms {
    pub screen_size: [u32; 2]
}

pub struct SquarePipeline {
    shader_module: ShaderModule,
    pipeline: RenderPipeline,
    pipeline_layout: wgpu::PipelineLayout,
    bind_group_layout: wgpu::BindGroupLayout,
    pub uniform_buffer: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl SquarePipeline {
    pub async fn new(device: &Device, cfmt: TextureFormat) -> Result<Self, Box<dyn Error>> {
        let shader_code = Cow::from(
            platform::read_text_asset("assets/square.wgsl").await?
        );
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Square shader module"),
            source: wgpu::ShaderSource::Wgsl(shader_code),
        });
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Square uniforms (layout)"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None
                },
                count: None,
            }],
        });
        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Square uniform buffer"),
            size: mem::size_of::<SquareUniforms>() as BufferAddress,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Square vertex buffer"),
            contents: bytemuck::cast_slice(&SQUARE_GEOM),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Square index buffer"),
            contents: bytemuck::cast_slice(&SQUARE_INDX),
            usage: BufferUsages::INDEX,
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Square uniforms"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pipeline for rendering a textured square (layout)"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Pipeline for rendering a textured square"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: "vertex_main",
                buffers: &[
                    VertexBufferLayout {
                        array_stride: mem::size_of::<SquareInstanceRaw>() as BufferAddress,
                        step_mode: VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x3],
                    },
                    VertexBufferLayout {
                        array_stride: mem::size_of::<SquareVertexRaw>() as BufferAddress,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![1 => Float32x4]
                    }
                ],
            },
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: Some(wgpu::IndexFormat::Uint16),
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: MultisampleState {
                count: 1,
                mask: 0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: "pixel_main",
                targets: &[Some(ColorTargetState::from(cfmt))],
            }),
            multiview: None,
        });
        Ok(SquarePipeline { pipeline, pipeline_layout, shader_module, bind_group_layout, bind_group, uniform_buffer, vertex_buffer, index_buffer })
    }
}

impl Deref for SquarePipeline {
    type Target = RenderPipeline;

    fn deref(&self) -> &Self::Target {
        &self.pipeline
    }
}
