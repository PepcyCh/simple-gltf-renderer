use crate::engine::Engine;
use crate::texture::Texture;

pub struct EnvMap {
    pub cubemap: Texture,
    pub irradiance: Texture,
    pub prefiltered: Texture,
    pub bind_group: wgpu::BindGroup,
}

// TODO - LUT in bind group

impl EnvMap {
    pub fn default(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        brdf_lut: &Texture,
    ) -> Self {
        let cubemap = Texture::default_cube(device, queue);
        let irradiance = Texture::default_cube(device, queue);
        let prefiltered = Texture::default_cube(device, queue);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("EnvMap Bind Group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&cubemap.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&cubemap.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&irradiance.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&irradiance.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&prefiltered.view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&prefiltered.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&brdf_lut.view),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(&brdf_lut.sampler),
                },
            ],
        });

        Self {
            cubemap,
            irradiance,
            prefiltered,
            bind_group,
        }
    }
}

impl Engine {
    pub fn create_env_map(
        &self,
        bytes: &[u8],
        width: u32,
        format: wgpu::TextureFormat,
        label: Option<&str>,
        brdf_lut: &Texture,
    ) -> EnvMap {
        let cubemap = Texture::from_bytes_cube(
            &self.graphics_state.device,
            &self.graphics_state.queue,
            bytes,
            width,
            format,
            true,
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            },
            label,
        );
        let irradiance =
            Texture::render_target_cube(&self.graphics_state.device, width, format, true);
        let prefiltered =
            Texture::render_target_cube(&self.graphics_state.device, width, format, true);

        let bind_group = self
            .graphics_state
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("EnvMap Bind Group"),
                layout: &self.graphics_state.bind_group_layouts["_Scene"],
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&cubemap.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&cubemap.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&irradiance.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&irradiance.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&prefiltered.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::Sampler(&prefiltered.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::TextureView(&brdf_lut.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::Sampler(&brdf_lut.sampler),
                    },
                ],
            });

        let pre_calc_uniform_buffer =
            self.graphics_state
                .device
                .create_buffer(&wgpu::BufferDescriptor {
                    label: Some("EnvMap Pre-Calc Uniform Buffer"),
                    size: 4,
                    usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
                    mapped_at_creation: false,
                });
        let pre_calc_bind_group =
            self.graphics_state
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("EnvMap Pre-Calc Bind Group"),
                    layout: &self.graphics_state.bind_group_layouts["_EnvMap"],
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&cubemap.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&cubemap.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: pre_calc_uniform_buffer.as_entire_binding(),
                        },
                    ],
                });
        self.generate_mipmap(&cubemap);

        let mut encoder =
            self.graphics_state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder - EnvMap - Irradiance"),
                });
        for i in 0..6 {
            let view = irradiance
                .texture
                .create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    base_array_layer: i,
                    array_layer_count: std::num::NonZeroU32::new(1),
                    ..Default::default()
                });
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass - EnvMap - Irradiance"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.graphics_state.render_pipelines["EnvMap-Irradiance"]);
            render_pass.set_bind_group(1, &pre_calc_bind_group, &[]);
            render_pass.set_bind_group(0, self.skybox_camera.get_bind_group(i as usize), &[]);
            render_pass.set_vertex_buffer(
                0,
                self.skybox_cube.vertex_buffer.as_ref().unwrap().slice(..),
            );
            render_pass.set_index_buffer(
                self.skybox_cube.index_buffer.as_ref().unwrap().slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.draw_indexed(0..self.skybox_cube.index_count(), 0, 0..1);
        }
        self.graphics_state
            .queue
            .submit(std::iter::once(encoder.finish()));
        self.generate_mipmap(&irradiance);

        let mut encoder =
            self.graphics_state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder - EnvMap - Prefilter"),
                });
        let mipmap_level_count = {
            let layer_size = wgpu::Extent3d {
                depth: 1,
                ..cubemap.size
            };
            layer_size.max_mips() as u32
        };
        for j in 0..mipmap_level_count {
            let roughness = (j as f32 / 6.0).min(1.0);
            self.graphics_state.queue.write_buffer(
                &pre_calc_uniform_buffer,
                0,
                bytemuck::cast_slice(&[roughness]),
            );
            for i in 0..6 {
                let view = prefiltered
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor {
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        base_array_layer: i,
                        array_layer_count: std::num::NonZeroU32::new(1),
                        base_mip_level: j,
                        level_count: std::num::NonZeroU32::new(1),
                        ..Default::default()
                    });
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass - EnvMap - Prefilter"),
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });
                render_pass.set_pipeline(&self.graphics_state.render_pipelines["EnvMap-Prefilter"]);
                render_pass.set_bind_group(1, &pre_calc_bind_group, &[]);
                render_pass.set_bind_group(0, self.skybox_camera.get_bind_group(i as usize), &[]);
                render_pass.set_vertex_buffer(
                    0,
                    self.skybox_cube.vertex_buffer.as_ref().unwrap().slice(..),
                );
                render_pass.set_index_buffer(
                    self.skybox_cube.index_buffer.as_ref().unwrap().slice(..),
                    wgpu::IndexFormat::Uint32,
                );
                render_pass.draw_indexed(0..self.skybox_cube.index_count(), 0, 0..1);
            }
        }
        self.graphics_state
            .queue
            .submit(std::iter::once(encoder.finish()));

        EnvMap {
            cubemap,
            irradiance,
            prefiltered,
            bind_group,
        }
    }
}
