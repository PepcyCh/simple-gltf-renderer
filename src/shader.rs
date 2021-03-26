use crate::vertex::MeshVertex;
use anyhow::*;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

pub struct Shader {
    pub name: String,
    pub uniform_properties: Vec<(String, UniformProperty)>,
    pub uniform_offsets: HashMap<String, usize>,
    pub uniform_size: usize,
    pub texture_properties: HashMap<String, TextureProperty>,
    pub textures_index: HashMap<String, u32>,
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
    Texture2D(String),
    Texture3D,
    TextureCube,
}

pub struct SubShader {
    tag: String,
    options: SubShaderOption,
    vs_file: String,
    fs_file: String,
    shader_definition: HashMap<String, Option<String>>,
    vs_module: Option<wgpu::ShaderModule>,
    fs_module: Option<wgpu::ShaderModule>,
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
        let mut textures_index = HashMap::new();
        for (i, (name, _)) in texture_properties.iter().enumerate() {
            textures_index.insert(name.clone(), i as u32);
        }
        let mut texture_properties_hm = HashMap::new();
        for (name, tex) in texture_properties {
            texture_properties_hm.insert(name, tex);
        }
        Self {
            name,
            uniform_properties,
            texture_properties: texture_properties_hm,
            textures_index,
            sub_shaders,
            uniform_size: 0,
            uniform_offsets: HashMap::new(),
            bind_group_layout: None,
        }
    }

    pub fn get_uniform_offset(&self, name: &str) -> Option<usize> {
        self.uniform_offsets.get(name).cloned()
    }

    pub fn get_texture_index(&self, name: &str) -> Option<u32> {
        self.textures_index.get(name).cloned()
    }

    pub fn build(&mut self, device: &wgpu::Device) -> Result<()> {
        self.build_sub_shaders(device)?;

        let mut entries = vec![];
        entries.push(crate::graphics::util::uniform_bind_group_entry(0));
        for (name, tex) in &self.texture_properties {
            let view_dimension = match tex {
                TextureProperty::Texture2D(_) => wgpu::TextureViewDimension::D2,
                TextureProperty::Texture3D => wgpu::TextureViewDimension::D3,
                TextureProperty::TextureCube => wgpu::TextureViewDimension::Cube,
            };
            entries.push(crate::graphics::util::texture_bind_group_entry(
                self.textures_index[name] * 2 + 1,
                view_dimension,
            ));
            entries.push(crate::graphics::util::sampler_bind_group_entry(
                self.textures_index[name] * 2 + 2,
            ));
        }

        self.bind_group_layout = Some(device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some(&format!("{} Bind Group Layout", &self.name)),
                entries: &entries,
            },
        ));

        self.build_uniform_offsets();

        Ok(())
    }

    fn build_uniform_offsets(&mut self) {
        self.uniform_offsets.clear();
        let mut total_size = 0;
        for (name, uniform) in &self.uniform_properties {
            let align = uniform.align();
            let diff = total_size % align;
            if diff != 0 {
                total_size += align - diff;
            }
            self.uniform_offsets.insert(name.clone(), total_size);
            total_size += uniform.size();
        }
        let diff = total_size % 16;
        if diff != 0 {
            total_size += 16 - diff;
        }
        self.uniform_size = total_size;
    }

    fn build_sub_shaders(&mut self, device: &wgpu::Device) -> Result<()> {
        for (_, sub) in &mut self.sub_shaders {
            sub.build(device)?;
        }
        Ok(())
    }
}

impl SubShader {
    pub fn new(
        name: String,
        options: SubShaderOption,
        vs_file: String,
        fs_file: String,
        shader_definition: HashMap<String, Option<String>>,
    ) -> Self {
        Self {
            tag: name,
            options,
            vs_file,
            fs_file,
            shader_definition,
            vs_module: None,
            fs_module: None,
        }
    }

    pub fn build(&mut self, device: &wgpu::Device) -> Result<()> {
        self.vs_module = Some(shader_util::compile_to_module(
            self.vs_file.as_str(),
            &self.shader_definition,
            device,
        )?);
        self.fs_module = Some(shader_util::compile_to_module(
            self.fs_file.as_str(),
            &self.shader_definition,
            device,
        )?);
        Ok(())
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
            label: Some(&format!("{}-{} Pipeline Layout", &shader.name, &self.tag)),
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
            label: Some(&format!("{}-{} Render Pipeline", &shader.name, &self.tag)),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &self.vs_module.as_ref().unwrap(),
                entry_point: "main",
                buffers: &[MeshVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &self.fs_module.as_ref().unwrap(),
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
pub struct ShaderParseError {
    parse_error: String,
}

impl std::fmt::Display for ShaderParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Shader parse error: {}", &self.parse_error)
    }
}

impl std::error::Error for ShaderParseError {}

impl TryFrom<&serde_json::Value> for Shader {
    type Error = ShaderParseError;

    fn try_from(value: &serde_json::Value) -> Result<Self, Self::Error> {
        if let Some(path) = value.as_str() {
            let json_file = std::fs::File::open(path);
            if json_file.is_err() {
                return Err(ShaderParseError {
                    parse_error: format!("Can't open shader json file '{}'", path),
                });
            }
            let json_reader = std::io::BufReader::new(json_file.unwrap());
            let json_value = serde_json::from_reader(json_reader);
            if json_value.is_err() {
                return Err(ShaderParseError {
                    parse_error: format!("Can't parse shader json file '{}'", path),
                });
            }
            return Shader::try_from(&json_value.unwrap());
        }

        let name = value["name"].as_str().unwrap().to_string();

        let uniform_properties_arr = value["uniform_properties"].as_array().unwrap();
        let mut uniform_properties = Vec::with_capacity(uniform_properties_arr.len());
        for prop in uniform_properties_arr {
            let prop = prop.as_array().unwrap();
            let name = prop[1].as_str().unwrap().to_string();
            let ty: UniformProperty = prop[0].as_str().unwrap().to_string().try_into()?;
            uniform_properties.push((name, ty));
        }

        let texture_properties_arr = value["texture_properties"].as_array().unwrap();
        let mut texture_properties = Vec::with_capacity(texture_properties_arr.len());
        for prop in texture_properties_arr {
            let prop = prop.as_array().unwrap();
            let name = prop[1].as_str().unwrap().to_string();
            let mut ty: TextureProperty = prop[0].as_str().unwrap().to_string().try_into()?;
            if let TextureProperty::Texture2D(default) = &mut ty {
                *default = prop[2].as_str().unwrap().to_string();
                if !["white", "black", "normal", "gray", "grey"].contains(&default.as_str()) {
                    return Err(ShaderParseError {
                        parse_error: format!("Unknown texture2D default value '{}'", default),
                    });
                }
            }
            texture_properties.push((name, ty));
        }

        let sub_shaders_arr = value["subshaders"].as_array().unwrap();
        let mut sub_shaders = HashMap::new();
        for sub in sub_shaders_arr {
            let tag = sub["tag"].as_str().unwrap().to_string();
            let vs_file = sub["vs"].as_str().unwrap();
            let fs_file = sub["fs"].as_str().unwrap();
            let mut shader_definition = HashMap::new();
            if let Some(definition) = sub.get("definition") {
                let definition = definition.as_object().unwrap();
                for (k, v) in definition {
                    if let Some(v) = v.as_str() {
                        shader_definition.insert(k.to_string(), Some(v.to_string()));
                    } else {
                        shader_definition.insert(k.to_string(), None);
                    }
                }
            }
            let option = sub.try_into()?;
            let sub_shader = SubShader::new(
                tag.clone(),
                option,
                vs_file.to_string(),
                fs_file.to_string(),
                shader_definition,
            );
            sub_shaders.insert(tag, sub_shader);
        }

        Ok(Shader::new(
            name,
            uniform_properties,
            texture_properties,
            sub_shaders,
        ))
    }
}

impl TryFrom<&serde_json::Value> for SubShaderOption {
    type Error = ShaderParseError;

    fn try_from(value: &serde_json::Value) -> Result<Self, Self::Error> {
        let mut option = Self::default();
        if let Some(cull) = value.get("cull") {
            let cull = cull.as_str().unwrap();
            match cull {
                "front" => option.cull_mode = wgpu::CullMode::Front,
                "back" => option.cull_mode = wgpu::CullMode::Back,
                "none" => option.cull_mode = wgpu::CullMode::None,
                _ => {
                    return Err(ShaderParseError {
                        parse_error: format!("Unknown cull value: '{}'", cull),
                    })
                }
            }
        }
        if let Some(front) = value.get("front_face") {
            let front = front.as_str().unwrap();
            match front {
                "ccw" => option.front_face = wgpu::FrontFace::Ccw,
                "cw" => option.front_face = wgpu::FrontFace::Cw,
                _ => {
                    return Err(ShaderParseError {
                        parse_error: format!("Unknown front face value: '{}'", front),
                    })
                }
            }
        }
        if let Some(write_mask) = value.get("write_mask") {
            let mut mask = wgpu::ColorWrite::from_bits(0).unwrap();
            let write_mask = write_mask.as_array().unwrap();
            for ch in write_mask {
                let ch = ch.as_str().unwrap();
                match ch {
                    "R" => mask.insert(wgpu::ColorWrite::RED),
                    "G" => mask.insert(wgpu::ColorWrite::GREEN),
                    "B" => mask.insert(wgpu::ColorWrite::BLUE),
                    "A" => mask.insert(wgpu::ColorWrite::ALPHA),
                    _ => {
                        return Err(ShaderParseError {
                            parse_error: format!("Unknown color write mask value: '{}'", ch),
                        })
                    }
                };
            }
            option.write_mask = mask;
        }
        if let Some(blend) = value.get("blend") {
            let mut color_blend = wgpu::BlendState::default();
            let mut alpha_blend = wgpu::BlendState::default();
            if let Some(op) = blend.get("op") {
                let op = op.as_str().unwrap();
                color_blend.operation = shader_option_util::blend_op_from_str(op)?;
            }
            if let Some(src) = blend.get("src") {
                let src = src.as_str().unwrap();
                color_blend.src_factor = shader_option_util::blend_factor_from_str(src)?;
            }
            if let Some(dst) = blend.get("dst") {
                let dst = dst.as_str().unwrap();
                color_blend.dst_factor = shader_option_util::blend_factor_from_str(dst)?;
            }
            if let Some(op) = blend.get("op_alpha") {
                let op = op.as_str().unwrap();
                alpha_blend.operation = shader_option_util::blend_op_from_str(op)?;
            }
            if let Some(src) = blend.get("src_alpha") {
                let src = src.as_str().unwrap();
                alpha_blend.src_factor = shader_option_util::blend_factor_from_str(src)?;
            }
            if let Some(dst) = blend.get("dst_alpha") {
                let dst = dst.as_str().unwrap();
                alpha_blend.dst_factor = shader_option_util::blend_factor_from_str(dst)?;
            }
            option.color_blend = color_blend;
            option.alpha_blend = alpha_blend;
        }
        if let Some(depth_write) = value.get("depth_write") {
            let depth_write = depth_write.as_bool().unwrap();
            option.depth_write = depth_write;
        }
        if let Some(depth_cmp) = value.get("depth_compare") {
            let depth_cmp = depth_cmp.as_str().unwrap();
            option.depth_compare = shader_option_util::compare_func_from_str(depth_cmp)?;
        }
        if let Some(stencil) = value.get("stencil") {
            let mut stencil_state = wgpu::StencilState::default();
            if let Some(read_mask) = stencil.get("read_mask") {
                stencil_state.read_mask = read_mask.as_u64().unwrap() as u32;
            }
            if let Some(write_mask) = stencil.get("write_mask") {
                stencil_state.write_mask = write_mask.as_u64().unwrap() as u32;
            }
            if let Some(front_state) = stencil.get("front") {
                stencil_state.front =
                    shader_option_util::stencil_face_state_from_json(front_state)?;
            }
            if let Some(back_state) = stencil.get("back") {
                stencil_state.back = shader_option_util::stencil_face_state_from_json(back_state)?;
            }
            option.stencil = stencil_state;
        }
        Ok(option)
    }
}

impl TryFrom<String> for UniformProperty {
    type Error = ShaderParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "float" => Ok(UniformProperty::Float),
            "vec2" => Ok(UniformProperty::Vec2),
            "vec3" => Ok(UniformProperty::Vec3),
            "vec4" => Ok(UniformProperty::Vec4),
            "mat3" => Ok(UniformProperty::Mat3),
            "mat4" => Ok(UniformProperty::Mat4),
            _ => Err(ShaderParseError {
                parse_error: format!("Unknown uniform property '{}'", value),
            }),
        }
    }
}

impl TryFrom<String> for TextureProperty {
    type Error = ShaderParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "2D" => Ok(TextureProperty::Texture2D("".to_string())),
            "3D" => Ok(TextureProperty::Texture3D),
            "Cube" => Ok(TextureProperty::TextureCube),
            _ => Err(ShaderParseError {
                parse_error: format!("Unknown texture property '{}'", value),
            }),
        }
    }
}

mod shader_util {
    use anyhow::*;
    use std::collections::HashMap;

    pub fn compile_to_module<P: AsRef<std::path::Path>>(
        path: P,
        definition: &HashMap<String, Option<String>>,
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

        let mut compile_options =
            shaderc::CompileOptions::new().context("Can't get compile options object")?;
        for (key, value) in definition {
            compile_options
                .add_macro_definition(key.as_str(), value.as_ref().map(|str| str.as_str()));
        }

        let compiler_result = compiler.compile_into_spirv(
            &shader_source,
            shader_kind,
            path_buf.to_str().unwrap(),
            "main",
            Some(&compile_options),
        )?;

        std::fs::write(spv_path, compiler_result.as_binary_u8())?;

        Ok(device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: path_buf.to_str(),
            source: wgpu::util::make_spirv(compiler_result.as_binary_u8()),
            flags: Default::default(),
        }))
    }
}

mod shader_option_util {
    use crate::shader::ShaderParseError;

    pub fn blend_factor_from_str(str: &str) -> Result<wgpu::BlendFactor, ShaderParseError> {
        match str {
            "one" => Ok(wgpu::BlendFactor::One),
            "zero" => Ok(wgpu::BlendFactor::Zero),
            "src_alpha" => Ok(wgpu::BlendFactor::SrcAlpha),
            "src_color" => Ok(wgpu::BlendFactor::SrcColor),
            "one_minus_src_alpha" => Ok(wgpu::BlendFactor::OneMinusSrcAlpha),
            "one_minus_src_color" => Ok(wgpu::BlendFactor::OneMinusSrcColor),
            "dst_alpha" => Ok(wgpu::BlendFactor::DstAlpha),
            "dst_color" => Ok(wgpu::BlendFactor::DstColor),
            "one_minus_dst_alpha" => Ok(wgpu::BlendFactor::OneMinusDstAlpha),
            "one_minus_dst_color" => Ok(wgpu::BlendFactor::OneMinusDstColor),
            _ => Err(ShaderParseError {
                parse_error: format!("Unknown blend factor '{}'", str),
            }),
        }
    }

    pub fn blend_op_from_str(str: &str) -> Result<wgpu::BlendOperation, ShaderParseError> {
        match str {
            "add" => Ok(wgpu::BlendOperation::Add),
            "max" => Ok(wgpu::BlendOperation::Max),
            "min" => Ok(wgpu::BlendOperation::Min),
            "sub" => Ok(wgpu::BlendOperation::Subtract),
            "rsub" => Ok(wgpu::BlendOperation::ReverseSubtract),
            _ => Err(ShaderParseError {
                parse_error: format!("Unknown blend operation '{}'", str),
            }),
        }
    }

    pub fn compare_func_from_str(str: &str) -> Result<wgpu::CompareFunction, ShaderParseError> {
        match str {
            "always" => Ok(wgpu::CompareFunction::Always),
            "never" => Ok(wgpu::CompareFunction::Never),
            "equal" => Ok(wgpu::CompareFunction::Equal),
            "nequal" => Ok(wgpu::CompareFunction::NotEqual),
            "less" => Ok(wgpu::CompareFunction::Less),
            "lequal" => Ok(wgpu::CompareFunction::LessEqual),
            "greater" => Ok(wgpu::CompareFunction::Greater),
            "gequal" => Ok(wgpu::CompareFunction::GreaterEqual),
            _ => Err(ShaderParseError {
                parse_error: format!("Unknown compare function '{}'", str),
            }),
        }
    }

    pub fn stencil_face_state_from_json(
        value: &serde_json::Value,
    ) -> Result<wgpu::StencilFaceState, ShaderParseError> {
        todo!("stencil_face_state_from_json")
    }
}
