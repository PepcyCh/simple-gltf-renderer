use crate::graphics::GraphicsState;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub size: wgpu::Extent3d,
    pub dimension: wgpu::TextureDimension,
    pub format: wgpu::TextureFormat,
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
        let dimension = wgpu::TextureDimension::D2;
        let format = GraphicsState::DEPTH_STENCIL_FORMAT;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension,
            format,
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
            size,
            dimension,
            format,
        }
    }

    pub fn from_bytes_2d(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        mipmap: bool,
        sampler_desc: &wgpu::SamplerDescriptor,
        label: Option<&str>,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };
        let dimension = wgpu::TextureDimension::D2;
        let texture = Self::wgpu_texture_from_bytes(
            device,
            queue,
            bytes,
            size,
            format,
            dimension,
            if mipmap { size.max_mips() as u32 } else { 1 },
            label,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(sampler_desc);

        Self {
            texture,
            view,
            sampler,
            size,
            dimension,
            format,
        }
    }

    pub fn from_bytes_cube(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        width: u32,
        format: wgpu::TextureFormat,
        mipmap: bool,
        sampler_desc: &wgpu::SamplerDescriptor,
        label: Option<&str>,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height: width,
            depth: 6,
        };
        let dimension = wgpu::TextureDimension::D2;
        let layer_size = wgpu::Extent3d { depth: 1, ..size };
        let texture = Self::wgpu_texture_from_bytes(
            device,
            queue,
            bytes,
            size,
            format,
            dimension,
            if mipmap {
                layer_size.max_mips() as u32
            } else {
                1
            },
            label,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        let sampler = device.create_sampler(sampler_desc);

        Self {
            texture,
            view,
            sampler,
            size,
            dimension,
            format,
        }
    }

    pub fn from_bytes_3d(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        width: u32,
        height: u32,
        depth: u32,
        format: wgpu::TextureFormat,
        mipmap: bool,
        sampler_desc: &wgpu::SamplerDescriptor,
        label: Option<&str>,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth,
        };
        let dimension = wgpu::TextureDimension::D3;
        let texture = Self::wgpu_texture_from_bytes(
            device,
            queue,
            bytes,
            size,
            format,
            dimension,
            if mipmap { size.max_mips() as u32 } else { 1 },
            label,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(sampler_desc);

        Self {
            texture,
            view,
            sampler,
            size,
            dimension,
            format,
        }
    }

    fn wgpu_texture_from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        dimension: wgpu::TextureDimension,
        mip_level_count: u32,
        label: Option<&str>,
    ) -> wgpu::Texture {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count,
            sample_count: 1,
            dimension,
            format,
            // TODO (a problem about mipmap generating)
            // I need this 'RENDER_ATTACHMENT' so that I can generate mipmap from fragment shader...
            // maybe using image crate to generate it is a better choice
            // but what about textures that are render targets ?
            // what about using compute shader ?
            usage: wgpu::TextureUsage::SAMPLED
                | wgpu::TextureUsage::COPY_DST
                | wgpu::TextureUsage::RENDER_ATTACHMENT,
        });
        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: Default::default(),
            },
            bytes,
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: size.width * format.describe().block_size as u32,
                rows_per_image: size.height,
            },
            size,
        );
        texture
    }

    pub fn render_target_cube(
        device: &wgpu::Device,
        width: u32,
        format: wgpu::TextureFormat,
        mipmap: bool,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height: width,
            depth: 6,
        };
        let layer_size = wgpu::Extent3d { depth: 1, ..size };
        let dimension = wgpu::TextureDimension::D2;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Render Target Texture Cube"),
            size,
            mip_level_count: if mipmap {
                layer_size.max_mips() as u32
            } else {
                1
            },
            sample_count: 1,
            dimension,
            format,
            usage: wgpu::TextureUsage::SAMPLED
                | wgpu::TextureUsage::COPY_DST
                | wgpu::TextureUsage::RENDER_ATTACHMENT,
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

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
            size,
            dimension,
            format,
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
            false,
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
            false,
            &wgpu::SamplerDescriptor::default(),
            Some("Black 1x1"),
        )
    }
    pub fn gray1x1(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        Self::from_bytes_2d(
            device,
            queue,
            &[128, 128, 128, 255],
            1,
            1,
            wgpu::TextureFormat::Rgba8Unorm,
            false,
            &wgpu::SamplerDescriptor::default(),
            Some("Gray 1x1"),
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
            false,
            &wgpu::SamplerDescriptor::default(),
            Some("Normal 1x1"),
        )
    }
    #[rustfmt::skip]
    pub fn default_cube(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        Self::from_bytes_cube(
            device,
            queue,
            &[
                255, 0, 0, 255,
                0, 255, 255, 255,
                0, 255, 0, 255,
                255, 0, 255, 255,
                0, 0, 255, 255,
                255, 255, 0, 255,
            ],
            1,
            wgpu::TextureFormat::Rgba8Unorm,
            false,
            &wgpu::SamplerDescriptor::default(),
            Some("Default Cube"),
        )
    }
    pub fn black1x1x1(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        Self::from_bytes_3d(
            device,
            queue,
            &[0, 0, 0, 255],
            1,
            1,
            1,
            wgpu::TextureFormat::Rgba8Unorm,
            false,
            &wgpu::SamplerDescriptor::default(),
            Some("Black 1x1x1"),
        )
    }
}
