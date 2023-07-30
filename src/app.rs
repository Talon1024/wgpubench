use std::error::Error;

use wgpu::*;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event_loop::{EventLoop, EventLoopBuilder, EventLoopProxy},
    window::{Window, WindowBuilder},
};

#[cfg(target_family="wasm")]
use wasm_bindgen::JsValue;

use crate::util::{surface::SurfaceInfo, texture::SimpleTextureView};

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
    window: Window,
    device: Device,
    queue: Queue,
    surface_info: SurfaceInfo,
    event_loop_proxy: EventLoopProxy<AppEvent>,
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
        #[cfg(not(target_family="wasm"))]
        println!("{backend}");
        #[cfg(target_family="wasm")]
        web_sys::console::log_1(&JsValue::from_str(&backend));
        Ok(AppState {
            instance,
            window,
            device,
            queue,
            surface_info,
            event_loop_proxy: primary_proxy,
        })
    }
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        // recreate the window surface
        self.surface_info.resize(&self.device, new_size);
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
                            r: 0.25,
                            g: 0.5,
                            b: 1.0,
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
            render_pass.set_blend_constant(Color {
                r: 0.25,
                g: 0.5,
                b: 1.0,
                a: 1.0,
            });
        }
        self.queue.submit([commands.finish()]);
        canvas.present();
        Ok(())
    }
}
