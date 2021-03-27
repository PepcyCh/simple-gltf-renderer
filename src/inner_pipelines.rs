use crate::engine::Engine;
use crate::vertex::MeshVertex;

impl Engine {
    pub(crate) fn init_inner_pipelines(&mut self) {
        self.blit_pipeline(&[
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Rgba16Float,
        ]);
        self.skybox_pipeline();
        self.envmap_pipeline();
    }

    fn blit_pipeline(&mut self, formats: &[wgpu::TextureFormat]) {
        let bind_group_layout =
            self.graphics_state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Blit Bind Group Layout"),
                    entries: &[
                        crate::graphics::util::texture_bind_group_entry(
                            0,
                            wgpu::TextureViewDimension::D2,
                        ),
                        crate::graphics::util::sampler_bind_group_entry(1),
                    ],
                });
        let pipeline_layout =
            self.graphics_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Blit Pipeline Layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });
        let vs_module = self
            .graphics_state
            .device
            .create_shader_module(&wgpu::include_spirv!(
                "../res/shaders/inner/screen.vert.spv"
            ));
        let fs_module = self
            .graphics_state
            .device
            .create_shader_module(&wgpu::include_spirv!("../res/shaders/inner/blit.frag.spv"));
        for format in formats {
            self.graphics_state.render_pipelines.insert(
                format!("Blit-{:?}", format),
                self.graphics_state.device.create_render_pipeline(
                    &wgpu::RenderPipelineDescriptor {
                        label: Some(&format!("Blit-{:?} Render Pipeline", format)),
                        layout: Some(&pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &vs_module,
                            entry_point: "main",
                            buffers: &[],
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &fs_module,
                            entry_point: "main",
                            targets: &[wgpu::ColorTargetState {
                                format: *format,
                                alpha_blend: wgpu::BlendState::REPLACE,
                                color_blend: wgpu::BlendState::REPLACE,
                                write_mask: wgpu::ColorWrite::ALL,
                            }],
                        }),
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: wgpu::CullMode::None,
                            polygon_mode: wgpu::PolygonMode::Fill,
                        },
                        depth_stencil: None,
                        multisample: wgpu::MultisampleState {
                            count: 1,
                            mask: !0,
                            alpha_to_coverage_enabled: false,
                        },
                    },
                ),
            );
        }
        self.graphics_state
            .bind_group_layouts
            .insert("_Blit".to_string(), bind_group_layout);
    }

    fn skybox_pipeline(&mut self) {
        let pipeline_layout =
            self.graphics_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Skybox Pipeline Layout"),
                    bind_group_layouts: &[
                        &self.graphics_state.bind_group_layouts["_Camera"],
                        &self.graphics_state.bind_group_layouts["_Scene"],
                    ],
                    push_constant_ranges: &[],
                });
        let vs_module = self
            .graphics_state
            .device
            .create_shader_module(&wgpu::include_spirv!(
                "../res/shaders/inner/skybox.vert.spv"
            ));
        let fs_module = self
            .graphics_state
            .device
            .create_shader_module(&wgpu::include_spirv!(
                "../res/shaders/inner/skybox.frag.spv"
            ));
        self.graphics_state.render_pipelines.insert(
            "Skybox".to_string(),
            self.graphics_state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Skybox Render Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &vs_module,
                        entry_point: "main",
                        buffers: &[MeshVertex::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &fs_module,
                        entry_point: "main",
                        targets: &[wgpu::ColorTargetState {
                            format: self.graphics_state.swap_chain_desc.format,
                            alpha_blend: wgpu::BlendState::REPLACE,
                            color_blend: wgpu::BlendState::REPLACE,
                            write_mask: wgpu::ColorWrite::ALL,
                        }],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: wgpu::CullMode::None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: crate::graphics::GraphicsState::DEPTH_STENCIL_FORMAT,
                        depth_write_enabled: false,
                        depth_compare: wgpu::CompareFunction::LessEqual,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                        clamp_depth: false,
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                }),
        );
    }

    fn envmap_pipeline(&mut self) {
        let bind_group_layout =
            self.graphics_state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("EnvMap Bind Group Layout"),
                    entries: &[
                        crate::graphics::util::texture_bind_group_entry(
                            0,
                            wgpu::TextureViewDimension::Cube,
                        ),
                        crate::graphics::util::sampler_bind_group_entry(1),
                        crate::graphics::util::uniform_bind_group_entry(2),
                    ],
                });
        let pipeline_layout =
            self.graphics_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("EnvMap Pipeline Layout"),
                    bind_group_layouts: &[
                        &self.graphics_state.bind_group_layouts["_Camera"],
                        &bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });
        let vs_module = self
            .graphics_state
            .device
            .create_shader_module(&wgpu::include_spirv!(
                "../res/shaders/inner/cubemap.vert.spv"
            ));
        let irradiance_fs_module =
            self.graphics_state
                .device
                .create_shader_module(&wgpu::include_spirv!(
                    "../res/shaders/inner/irradiance_convolution.frag.spv"
                ));
        let prefilter_fs_module =
            self.graphics_state
                .device
                .create_shader_module(&wgpu::include_spirv!(
                    "../res/shaders/inner/prefilter.frag.spv"
                ));
        self.graphics_state.render_pipelines.insert(
            "EnvMap-Irradiance".to_string(),
            self.graphics_state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("EnvMap-Irradiance Render Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &vs_module,
                        entry_point: "main",
                        buffers: &[MeshVertex::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &irradiance_fs_module,
                        entry_point: "main",
                        targets: &[wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Rgba8UnormSrgb,
                            alpha_blend: wgpu::BlendState::REPLACE,
                            color_blend: wgpu::BlendState::REPLACE,
                            write_mask: wgpu::ColorWrite::ALL,
                        }],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: wgpu::CullMode::None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                }),
        );
        self.graphics_state.render_pipelines.insert(
            "EnvMap-Prefilter".to_string(),
            self.graphics_state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("EnvMap-Prefilter Render Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &vs_module,
                        entry_point: "main",
                        buffers: &[MeshVertex::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &prefilter_fs_module,
                        entry_point: "main",
                        targets: &[wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Rgba8UnormSrgb,
                            alpha_blend: wgpu::BlendState::REPLACE,
                            color_blend: wgpu::BlendState::REPLACE,
                            write_mask: wgpu::ColorWrite::ALL,
                        }],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: wgpu::CullMode::None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                }),
        );
        self.graphics_state
            .bind_group_layouts
            .insert("_EnvMap".to_string(), bind_group_layout);
    }
}
