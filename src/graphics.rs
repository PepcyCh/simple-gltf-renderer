use crate::texture::Texture;
use anyhow::*;
use std::collections::HashMap;

pub struct GraphicsState {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub swap_chain: wgpu::SwapChain,
    pub swap_chain_desc: wgpu::SwapChainDescriptor,
    pub depth_stencil_texture: Texture,
    pub render_pipelines: HashMap<String, wgpu::RenderPipeline>,
    pub bind_group_layouts: HashMap<String, wgpu::BindGroupLayout>,
}

impl GraphicsState {
    pub const DEPTH_STENCIL_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24PlusStencil8;

    pub async fn new(window: &winit::window::Window) -> Result<Self> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .context("Can't request adapter")?;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits {
                        max_bind_groups: 5,
                        ..Default::default()
                    },
                },
                None,
            )
            .await?;
        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);
        let depth_stencil_texture = Texture::depth_stencil_texture(
            &device,
            &swap_chain_desc,
            Some("Default Depth Stencil Texture"),
        );

        let object_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Object Bind Group Layout"),
                entries: &[util::uniform_bind_group_entry(0)],
            });
        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Light Bind Group Layout"),
                entries: &[
                    util::uniform_bind_group_entry(0),
                    // for future use (shadow map)
                    // util::texture_bind_group_entry(1, wgpu::TextureViewDimension::D2),
                    // util::texture_bind_group_entry(2, wgpu::TextureViewDimension::Cube),
                    // util::sampler_bind_group_entry(3),
                ],
            });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[util::uniform_bind_group_entry(0)],
            });
        let scene_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Scene Bind Group Layout"),
                entries: &[
                    util::texture_bind_group_entry(0, wgpu::TextureViewDimension::Cube),
                    util::sampler_bind_group_entry(1),
                    util::texture_bind_group_entry(2, wgpu::TextureViewDimension::Cube),
                    util::sampler_bind_group_entry(3),
                    util::texture_bind_group_entry(4, wgpu::TextureViewDimension::Cube),
                    util::sampler_bind_group_entry(5),
                    util::texture_bind_group_entry(6, wgpu::TextureViewDimension::D2),
                    util::sampler_bind_group_entry(7),
                ],
            });
        let mut bind_group_layouts = HashMap::new();
        bind_group_layouts.insert("_Object".to_string(), object_bind_group_layout);
        bind_group_layouts.insert("_Light".to_string(), light_bind_group_layout);
        bind_group_layouts.insert("_Camera".to_string(), camera_bind_group_layout);
        bind_group_layouts.insert("_Scene".to_string(), scene_bind_group_layout);

        Ok(Self {
            surface,
            device,
            queue,
            swap_chain,
            swap_chain_desc,
            depth_stencil_texture,
            render_pipelines: HashMap::new(),
            bind_group_layouts,
        })
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        self.swap_chain_desc.width = new_width;
        self.swap_chain_desc.height = new_height;
        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_desc);
        self.depth_stencil_texture = Texture::depth_stencil_texture(
            &self.device,
            &self.swap_chain_desc,
            Some("Default Depth Stencil Texture"),
        );
    }
}

pub mod util {
    pub fn uniform_bind_group_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    pub fn texture_bind_group_entry(
        binding: u32,
        view_dimension: wgpu::TextureViewDimension,
    ) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension,
                multisampled: false,
            },
            count: None,
        }
    }

    pub fn sampler_bind_group_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
            ty: wgpu::BindingType::Sampler {
                filtering: true,
                comparison: false,
            },
            count: None,
        }
    }
}
