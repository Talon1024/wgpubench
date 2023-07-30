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
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        })
    }
}
