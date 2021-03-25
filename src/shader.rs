use crate::vertex::MeshVertex;
use anyhow::*;
use std::collections::HashMap;
use std::convert::TryFrom;

pub struct Shader {
    pub name: String,
    pub uniform_properties: Vec<(String, UniformProperty)>,
    uniform_offsets: Vec<usize>,
    pub uniform_size: usize,
    pub texture_properties: Vec<(String, TextureProperty)>,
    pub sub_shaders: HashMap<String, SubShader>,
    pub bind_group_layout: Option<wgpu::BindGroupLayout>,
}

#[derive(Eq, PartialEq)]
pub enum UniformProperty {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Mat3,
    Mat4,
}

#[derive(Eq, PartialEq)]
pub enum TextureProperty {
    Texture2D,
    Texture3D,
    TextureCube,
}

pub struct SubShader {
    name: String,
    options: SubShaderOption,
    vs_module: wgpu::ShaderModule,
    fs_modules: wgpu::ShaderModule,
}

pub struct SubShaderOption {
    cull_mode: wgpu::CullMode,
    front_face: wgpu::FrontFace,
    write_mask: wgpu::ColorWrite,
    color_blend: wgpu::BlendState,
    alpha_blend: wgpu::BlendState,
    depth_write: bool,
    depth_compare: wgpu::CompareFunction,
    stencil: wgpu::StencilState,
}

impl Default for SubShaderOption {
    fn default() -> Self {
        Self {
            cull_mode: wgpu::CullMode::Back,
            front_face: wgpu::FrontFace::Ccw,
            write_mask: wgpu::ColorWrite::ALL,
            color_blend: wgpu::BlendState::REPLACE,
            alpha_blend: wgpu::BlendState::REPLACE,
            depth_write: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
        }
    }
}

impl Shader {
    pub fn new(
        name: String,
        uniform_properties: Vec<(String, UniformProperty)>,
        texture_properties: Vec<(String, TextureProperty)>,
        sub_shaders: HashMap<String, SubShader>,
    ) -> Self {
        Self {
            name,
            uniform_properties,
            texture_properties,
            sub_shaders,
            uniform_size: 0,
            uniform_offsets: vec![],
            bind_group_layout: None,
        }
    }

    pub fn get_uniform_offset(&self, name: &str) -> Option<usize> {
        self.uniform_properties
            .iter()
            .position(|(self_name, _)| name == self_name)
            .map(|ind| self.uniform_offsets[ind])
    }

    pub fn build(&mut self, device: &wgpu::Device) {
        let mut entries = vec![];
        entries.push(crate::graphics::util::uniform_bind_group_entry(0));
        let mut curr_binding = 1;
        for (_, tex) in &self.texture_properties {
            let view_dimension = match tex {
                TextureProperty::Texture2D => wgpu::TextureViewDimension::D2,
                TextureProperty::Texture3D => wgpu::TextureViewDimension::D3,
                TextureProperty::TextureCube => wgpu::TextureViewDimension::Cube,
            };
            entries.push(crate::graphics::util::texture_bind_group_entry(
                curr_binding,
                view_dimension,
            ));
            curr_binding += 1;
            entries.push(crate::graphics::util::sampler_bind_group_entry(
                curr_binding,
            ));
            curr_binding += 1;
        }

        self.bind_group_layout = Some(device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some(&format!("{} Bind Group Layout", &self.name)),
                entries: &entries,
            },
        ));

        self.build_uniform_offsets();
    }

    fn build_uniform_offsets(&mut self) {
        self.uniform_offsets.clear();
        let mut total_size = 0;
        for (_, uniform) in &self.uniform_properties {
            let align = uniform.align();
            let diff = total_size % align;
            if diff != 0 {
                total_size += align - diff;
            }
            self.uniform_offsets.push(total_size);
            total_size += uniform.size();
        }
        let diff = total_size % 16;
        if diff != 0 {
            total_size += 16 - diff;
        }
        self.uniform_size = total_size;
    }
}

impl SubShader {
    pub fn new(
        name: String,
        options: SubShaderOption,
        vs_module: wgpu::ShaderModule,
        fs_modules: wgpu::ShaderModule,
    ) -> Self {
        Self {
            name,
            options,
            vs_module,
            fs_modules,
        }
    }

    pub fn render_pipeline(
        &self,
        shader: &Shader,
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
        depth_stencil_format: wgpu::TextureFormat,
        object_bind_group_layout: &wgpu::BindGroupLayout,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        light_bind_group_layout: &wgpu::BindGroupLayout,
        scene_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{}-{} Pipeline Layout", &shader.name, &self.name)),
            bind_group_layouts: &[
                &shader.bind_group_layout.as_ref().unwrap(),
                object_bind_group_layout,
                camera_bind_group_layout,
                light_bind_group_layout,
                // scene_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("{}-{} Render Pipeline", &shader.name, &self.name)),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &self.vs_module,
                entry_point: "main",
                buffers: &[MeshVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &self.fs_modules,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: color_format,
                    alpha_blend: self.options.alpha_blend.clone(),
                    color_blend: self.options.color_blend.clone(),
                    write_mask: self.options.write_mask,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: self.options.front_face,
                cull_mode: self.options.cull_mode,
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_stencil_format,
                depth_write_enabled: self.options.depth_write,
                depth_compare: self.options.depth_compare,
                stencil: self.options.stencil.clone(),
                bias: wgpu::DepthBiasState::default(),
                clamp_depth: false,
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        })
    }
}

impl UniformProperty {
    pub fn size(&self) -> usize {
        match self {
            UniformProperty::Float => 4,
            UniformProperty::Vec2 => 8,
            UniformProperty::Vec3 => 12,
            UniformProperty::Vec4 => 16,
            UniformProperty::Mat3 => 48,
            UniformProperty::Mat4 => 64,
        }
    }

    pub fn align(&self) -> usize {
        match self {
            UniformProperty::Float => 4,
            UniformProperty::Vec2 => 8,
            UniformProperty::Vec3 => 16,
            UniformProperty::Vec4 => 16,
            UniformProperty::Mat3 => 16,
            UniformProperty::Mat4 => 16,
        }
    }
}

#[derive(Debug)]
pub struct PropertyConvertError {
    unknown_input: String,
}

impl std::fmt::Display for PropertyConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown property type: {}", &self.unknown_input)
    }
}
impl std::error::Error for PropertyConvertError {}

impl TryFrom<String> for UniformProperty {
    type Error = PropertyConvertError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "float" => Ok(UniformProperty::Float),
            "vec2" => Ok(UniformProperty::Vec2),
            "vec3" => Ok(UniformProperty::Vec3),
            "vec4" => Ok(UniformProperty::Vec4),
            "mat3" => Ok(UniformProperty::Mat3),
            "mat4" => Ok(UniformProperty::Mat4),
            _ => Err(PropertyConvertError {
                unknown_input: value,
            }),
        }
    }
}

impl TryFrom<String> for TextureProperty {
    type Error = PropertyConvertError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "2D" => Ok(TextureProperty::Texture2D),
            "3D" => Ok(TextureProperty::Texture3D),
            "Cube" => Ok(TextureProperty::TextureCube),
            _ => Err(PropertyConvertError {
                unknown_input: value,
            }),
        }
    }
}

pub mod util {
    use anyhow::*;

    pub fn compile_to_module<P: AsRef<std::path::Path>>(
        path: P,
        device: &wgpu::Device,
    ) -> Result<wgpu::ShaderModule> {
        let path_buf = path.as_ref().to_path_buf();
        let shader_source = std::fs::read_to_string(path)?;
        let orig_extension = path_buf
            .extension()
            .context("No extension")?
            .to_str()
            .context("Invalid extension")?;
        let spv_path = path_buf.with_extension(format!("{}.spv", orig_extension));
        let shader_kind = match orig_extension {
            "vert" => Some(shaderc::ShaderKind::Vertex),
            "frag" => Some(shaderc::ShaderKind::Fragment),
            "comp" => Some(shaderc::ShaderKind::Compute),
            _ => None,
        }
        .context("Unknown shader kind")?;
        let mut compiler = shaderc::Compiler::new().context("Can't get compiler")?;
        let compiler_result = compiler.compile_into_spirv(
            &shader_source,
            shader_kind,
            path_buf.to_str().unwrap(),
            "main",
            None,
        )?;
        std::fs::write(spv_path, compiler_result.as_binary_u8())?;
        Ok(device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: path_buf.to_str(),
            source: wgpu::util::make_spirv(compiler_result.as_binary_u8()),
            flags: Default::default(),
        }))
    }
}
