use std::{error::Error, ops::Deref};
use wgpu::*;
use winit::{dpi::PhysicalSize, window::Window};

pub struct SurfaceInfo {
    pub surface: Surface,
    pub backend: Backend,
    pub depth_texture: Texture,
    pub depth_texture_view: TextureView,
}

impl SurfaceInfo {
    pub async fn create(
        instance: &Instance,
        window: &Window,
    ) -> Result<(Self, Device, Queue), Box<dyn Error>> {
        // In order for the adapter to be able to render to the surface, the
        // adapter needs a surface to be compatible with.
        let surface = unsafe { instance.create_surface(window) }?;
        let PhysicalSize { width, height } = window.inner_size();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .ok_or(String::from("No suitable GPU found"))?;
        // I like to show the user which backend is being used once they start
        // the app.
        let backend = adapter.get_info().backend;
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("My GPU"),
                    features: Features::empty(),
                    limits: Limits::downlevel_webgl2_defaults(),
                },
                None,
            )
            .await?;
        // wasm and native support different surface texture formats
        #[cfg(target_family="wasm")]
        let surface_format = TextureFormat::Rgba8UnormSrgb;
        #[cfg(not(target_family="wasm"))]
        let surface_format = TextureFormat::Bgra8UnormSrgb;
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: vec![surface_format],
        };
        surface.configure(&device, &config);
        // The depth texture is the same size as the surface
        let depth_texture = device.create_texture(&TextureDescriptor {
            label: Some("My depth texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[TextureFormat::Depth32Float],
        });
        let depth_texture_view = depth_texture.create_view(&TextureViewDescriptor {
            label: Some("View for my depth texture"),
            format: Some(TextureFormat::Depth32Float),
            dimension: Some(TextureViewDimension::D2),
            aspect: TextureAspect::DepthOnly,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
        Ok((
            Self {
                surface,
                backend,
                depth_texture,
                depth_texture_view,
            },
            device,
            queue,
        ))
    }
    pub fn resize(&mut self, device: &Device, new_size: PhysicalSize<u32>) {
        let PhysicalSize { width, height } = new_size;
        self.depth_texture = device.create_texture(&TextureDescriptor {
            label: Some("My depth texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[TextureFormat::Depth32Float],
        });
        self.depth_texture_view = self.depth_texture.create_view(&TextureViewDescriptor {
            label: Some("View for my depth texture"),
            format: Some(TextureFormat::Depth32Float),
            dimension: Some(TextureViewDimension::D2),
            aspect: TextureAspect::DepthOnly,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
    }
}

impl Deref for SurfaceInfo {
    type Target = Surface;

    fn deref(&self) -> &Self::Target {
        &self.surface
    }
}
