use image::{DynamicImage, imageops::FilterType, ImageBuffer, Rgba};
use wgpu::{TextureDescriptor, Extent3d, TextureFormat, TextureUsages, ImageCopyTexture, Origin3d, TextureAspect, ImageDataLayout};
use std::{error::Error, borrow::{Cow, Borrow}};
use crate::platform;

pub struct SimpleTextureView;
impl SimpleTextureView {
    pub fn new(texture: &wgpu::Texture, label: Option<&'static str>) -> wgpu::TextureView {
        let format = texture.format();
        let dimension = match texture.dimension() {
            wgpu::TextureDimension::D1 => wgpu::TextureViewDimension::D1,
            wgpu::TextureDimension::D2 => wgpu::TextureViewDimension::D2,
            wgpu::TextureDimension::D3 => wgpu::TextureViewDimension::D3,
        };
        texture.create_view(&wgpu::TextureViewDescriptor {
            label,
            format: Some(format),
            dimension: Some(dimension),
            aspect: TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        })
    }
}

pub const MIP_LEVELS: u32 = 4;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub sampler: wgpu::Sampler,
    pub view: wgpu::TextureView,
    format: TextureFormat,
}

macro_rules! conversion {
    ($conversion_function: path) => {
        &|i| DynamicImage::from($conversion_function(i))
    };
}

trait IntoRgba16Float {
    fn into_rgba16f(image: DynamicImage) -> ImageBuffer<Rgba<u16>, Vec<u16>>;
}

impl IntoRgba16Float for DynamicImage {
    fn into_rgba16f(image: DynamicImage) -> ImageBuffer<Rgba<u16>, Vec<u16>> {
        let width = image.width();
        let height = image.height();
        let buf = image.into_rgba32f().into_vec().into_iter()
            .map(half::f16::from_f32)
            .map(half::f16::to_bits)
            .collect();
        let buf: ImageBuffer<Rgba<u16>, Vec<u16>> = ImageBuffer::from_vec(width, height, buf).unwrap();
        buf
    }
}

impl Texture {
    pub async fn load_asset(device: &wgpu::Device, queue: &wgpu::Queue, path: &'static str, label: Option<&str>) -> Result<Self, Box<dyn Error>> {
        let asset_data = platform::read_asset(path).await?;
        let image = image::load_from_memory(&asset_data)?;
        Texture::from_image(device, queue, image, label)
    }
    pub fn from_image(device: &wgpu::Device, queue: &wgpu::Queue, image: DynamicImage, label: Option<&str>) -> Result<Self, Box<dyn Error>> {
        let width = image.width();
        let height = image.height();
        // For now, the image has to be Rgba8UnormSrgb or Rgba16Float.
        // wgpu doesn't implement float32-filterable.
        // The `half` crate is used to convert images to Rgba16Float.
        let (format, conversion): (Result<TextureFormat, String>, Option<&dyn Fn(DynamicImage) -> DynamicImage>) = match &image {
            DynamicImage::ImageLuma8(_) => (Ok(TextureFormat::Rgba8UnormSrgb), Some(conversion!(DynamicImage::into_rgba8))),
            DynamicImage::ImageLumaA8(_) => (Ok(TextureFormat::Rgba8UnormSrgb), Some(conversion!(DynamicImage::into_rgba8))),
            DynamicImage::ImageRgb8(_) => (Ok(TextureFormat::Rgba8UnormSrgb), Some(conversion!(DynamicImage::into_rgba8))),
            DynamicImage::ImageRgba8(_) => (Ok(TextureFormat::Rgba8UnormSrgb), None),
            DynamicImage::ImageLuma16(_) => (Ok(TextureFormat::Rgba16Float), Some(conversion!(DynamicImage::into_rgba16f))),
            DynamicImage::ImageLumaA16(_) => (Ok(TextureFormat::Rgba16Float), Some(conversion!(DynamicImage::into_rgba16f))),
            DynamicImage::ImageRgb16(_) => (Ok(TextureFormat::Rgba16Float), Some(conversion!(DynamicImage::into_rgba16f))),
            DynamicImage::ImageRgba16(_) => (Ok(TextureFormat::Rgba16Float), Some(conversion!(DynamicImage::into_rgba16f))),
            DynamicImage::ImageRgb32F(_) => (Ok(TextureFormat::Rgba16Float), Some(conversion!(DynamicImage::into_rgba16f))),
            DynamicImage::ImageRgba32F(_) => (Ok(TextureFormat::Rgba16Float), Some(conversion!(DynamicImage::into_rgba16f))),
            f => (Err(format!("Unknown/unsupported format {f:?}")), None),
        };
        let format = format?;
        let image = match conversion {
            Some(f) => f(image),
            None => image,
        };
        let mip_level_count = MIP_LEVELS.max(1);
        let texture = device.create_texture(&TextureDescriptor {
            label,
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
            view_formats: &[format],
        });
        for mip_level in 0..mip_level_count {
            let nwidth = width >> mip_level;
            let nheight = height >> mip_level;
            let bytes_per_channel = match format {
                TextureFormat::Rgba8UnormSrgb => 1,
                TextureFormat::Rgba16Float => 2,
                TextureFormat::Rgba32Float => 4,
                _ => unreachable!()
            };
            let channels = match format {
                TextureFormat::Rgba8UnormSrgb => 4,
                TextureFormat::Rgba16Float => 4,
                TextureFormat::Rgba32Float => 4,
                _ => unreachable!()
            };
            let bytes_per_row = nwidth * channels * bytes_per_channel;
            let data = if mip_level > 0 {
                let filter = FilterType::CatmullRom;
                let image = image.resize(nwidth, nheight, filter);
                Cow::Owned(image.into_bytes())
            } else {
                Cow::from(image.as_bytes())
            };
            let data_layout = ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: None,
            };
            let copy = ImageCopyTexture {
                texture: &texture,
                mip_level,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            };
            let size = Extent3d {
                width: nwidth,
                height: nheight,
                depth_or_array_layers: 1,
            };
            queue.write_texture(copy, data.borrow(), data_layout, size);
        }
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 4.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label,
            format: Some(format),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(mip_level_count),
            base_array_layer: 0,
            array_layer_count: None,
        });
        Ok(Self {
            texture,
            sampler,
            view,
            format
        })
    }
}
