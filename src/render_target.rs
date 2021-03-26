use crate::texture::Texture;
use std::collections::HashMap;

impl Texture {
    pub fn render_target_cube(
        device: &wgpu::Device,
        width: u32,
        format: wgpu::TextureFormat
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height: width,
            depth: 6
        };
        let mip_level_count = size.max_mips() as u32;
        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label: Some("Render Target Texture Cube"),
                size,
                mip_level_count,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST
                    | wgpu::TextureUsage::RENDER_ATTACHMENT,
            }
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        let mut views = HashMap::new();
        for i in 0..mip_level_count {
            for j in 0..6 {
                let view = texture.create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    base_mip_level: i,
                    level_count: Some(std::num::NonZeroU32::new(1).unwrap()),
                    base_array_layer: j,
                    array_layer_count: Some(std::num::NonZeroU32::new(1).unwrap()),
                    ..Default::default()
                });
                views.insert(format!("level-{} layer-{}", i, j), view);
            }
        }
        views.insert("level-0".to_string(), texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2),
            base_mip_level: 0,
            level_count: Some(std::num::NonZeroU32::new(1).unwrap()),
            ..Default::default()
        }));

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Cube Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            texture,
            view,
            sampler,
            views,
        }
    }
}