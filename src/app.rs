use std::error::Error;

use glam::Vec2;
use wgpu::{*, util::{DeviceExt, BufferInitDescriptor}};
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event_loop::{EventLoop, EventLoopBuilder, EventLoopProxy},
    window::{Window, WindowBuilder},
};

#[cfg(target_family = "wasm")]
use wasm_bindgen::JsValue;

use crate::{
    square::{SquarePipeline, SquareUniforms, SquareInstance, SquareInstanceRaw},
    util::{surface::SurfaceInfo, texture::SimpleTextureView},
};

pub struct CreatedWindow<T: 'static> {
    pub window: Window,
    pub event_loop: EventLoop<T>,
}

pub fn create_window() -> Result<CreatedWindow<AppEvent>, Box<dyn Error>> {
    let event_loop = EventLoopBuilder::with_user_event().build();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(640, 480))
        .build(&event_loop)?;
    Ok(CreatedWindow { window, event_loop })
}

pub enum AppEvent {}

pub struct AppState {
    instance: Instance,
    pub window: Window,
    device: Device,
    queue: Queue,
    surface_info: SurfaceInfo,
    event_loop_proxy: EventLoopProxy<AppEvent>,
    square_pipeline: SquarePipeline,
    square_uniforms: SquareUniforms,
    square_instances: Vec<SquareInstance>,
    square_instance_count: usize,
    square_instance_buffer: Buffer,
}

impl AppState {
    pub async fn setup(
        window: Window,
        primary_proxy: EventLoopProxy<AppEvent>,
    ) -> Result<AppState, Box<dyn Error>> {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        let (surface_info, device, queue) = SurfaceInfo::create(&instance, &window).await?;
        let backend = format!("wgpu backend: {:?}", surface_info.backend);
        #[cfg(not(target_family = "wasm"))]
        println!("{backend}");
        #[cfg(target_family = "wasm")]
        web_sys::console::log_1(&JsValue::from_str(&backend));

        let screen_size = window.inner_size();
        let square_pipeline = SquarePipeline::new(&device, surface_info.format()).await?;
        let square_uniforms = SquareUniforms { screen_size: [screen_size.width, screen_size.height] };
        queue.write_buffer(&square_pipeline.uniform_buffer, 0, bytemuck::cast_slice(&[square_uniforms]));
        let square_instances = vec![SquareInstance {
            pos: Vec2::new(0.0, 0.0),
            hue: 0.125,
        }];
        let square_instance_count = square_instances.len();
        let square_instance_data: Vec<_> = square_instances.iter().copied().map(SquareInstanceRaw::from).collect();
        let square_instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Square instance buffer"),
            contents: bytemuck::cast_slice(&square_instance_data),
            usage: BufferUsages::VERTEX,
        });
        Ok(AppState {
            instance,
            window,
            device,
            queue,
            surface_info,
            square_pipeline,
            square_instances,
            square_instance_count,
            square_instance_buffer,
            square_uniforms,
            event_loop_proxy: primary_proxy,
        })
    }
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        // recreate the window surface
        self.surface_info.resize(&self.device, new_size);
        self.square_uniforms = SquareUniforms { screen_size: [new_size.width, new_size.height] };
        self.queue.write_buffer(&self.square_pipeline.uniform_buffer, 0, bytemuck::cast_slice(&[self.square_uniforms]))
    }
    pub fn render(&self) -> Result<(), Box<dyn Error>> {
        // Get the output texture to render to
        let canvas = self.surface_info.get_current_texture()?;
        let mut commands = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("My commands"),
            });
        {
            let canvas_view = SimpleTextureView::new(&canvas.texture, None);
            let mut render_pass = commands.begin_render_pass(&RenderPassDescriptor {
                label: Some("My render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &canvas_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.125,
                            g: 0.125,
                            b: 0.25,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.surface_info.depth_texture_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            render_pass.set_vertex_buffer(0, self.square_instance_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.square_pipeline.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.square_pipeline.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.set_pipeline(&self.square_pipeline);
            render_pass.set_bind_group(0, &self.square_pipeline.bind_group, &[]);
            render_pass.draw_indexed(0..4, 0, 0..1)
        }
        self.queue.submit([commands.finish()]);
        canvas.present();
        Ok(())
    }
}
