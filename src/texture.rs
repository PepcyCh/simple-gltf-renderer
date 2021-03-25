use crate::graphics::GraphicsState;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn depth_stencil_texture(
        device: &wgpu::Device,
        swap_chain_desc: &wgpu::SwapChainDescriptor,
        label: Option<&str>,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: swap_chain_desc.width,
            height: swap_chain_desc.height,
            depth: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: GraphicsState::DEPTH_STENCIL_FORMAT,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn from_bytes_2d(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        sampler_desc: &wgpu::SamplerDescriptor,
        label: Option<&str>,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            bytes,
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: format.describe().block_size as u32 * width,
                rows_per_image: height,
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(sampler_desc);

        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn white1x1(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        Self::from_bytes_2d(
            device,
            queue,
            &[255, 255, 255, 255],
            1,
            1,
            wgpu::TextureFormat::Rgba8Unorm,
            &wgpu::SamplerDescriptor::default(),
            Some("White 1x1"),
        )
    }
    pub fn black1x1(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        Self::from_bytes_2d(
            device,
            queue,
            &[0, 0, 0, 255],
            1,
            1,
            wgpu::TextureFormat::Rgba8Unorm,
            &wgpu::SamplerDescriptor::default(),
            Some("Black 1x1"),
        )
    }
    pub fn normal1x1(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        Self::from_bytes_2d(
            device,
            queue,
            &[128, 128, 255, 255],
            1,
            1,
            wgpu::TextureFormat::Rgba8Unorm,
            &wgpu::SamplerDescriptor::default(),
            Some("Normal 1x1"),
        )
    }
}
