use anyhow::*;
use byte_slice_cast::AsSliceOf;
use cgmath::SquareMatrix;
use gltf::json::texture::{MagFilter, MinFilter, WrappingMode};

use crate::engine::Engine;
use crate::mesh::Mesh;
use crate::shader::TextureProperty;
use crate::texture::Texture;
use crate::vertex::MeshVertex;
use gltf::image::Format;

struct GltfScene {
    gltf_document: gltf::Document,
    buffers: Vec<gltf::buffer::Data>,
    images: Vec<gltf::image::Data>,
}

impl GltfScene {
    fn import<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let (gltf_document, buffers, images) = gltf::import(path)?;
        Ok(Self {
            gltf_document,
            buffers,
            images,
        })
    }

    fn data_of_accessor<'a>(&'a self, accessor: &gltf::Accessor<'a>) -> Result<&'a [u8]> {
        let buffer_view = accessor.view().context("Accessor has no buffer view")?;
        let buffer = buffer_view.buffer();
        let buffer_data = &self.buffers[buffer.index()];
        let buffer_view_data =
            &buffer_data[buffer_view.offset()..buffer_view.offset() + buffer_view.length()];
        let accessor_data = &buffer_view_data
            [accessor.offset()..accessor.offset() + accessor.count() * accessor.size()];
        Ok(accessor_data)
    }
}

impl Engine {
    pub fn load_gltf<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        let gltf_scene = GltfScene::import(path)?;

        self.meshes.reserve(gltf_scene.gltf_document.meshes().len());
        for s in gltf_scene.gltf_document.scenes() {
            for node in s.nodes() {
                self.parse_gltf_node(&node, &gltf_scene, cgmath::Matrix4::identity())?;
            }
        }

        Ok(())
    }

    fn parse_gltf_node(
        &mut self,
        node: &gltf::Node,
        gltf_scene: &GltfScene,
        transform: cgmath::Matrix4<f32>,
    ) -> Result<()> {
        let curr_trans: cgmath::Matrix4<f32> = node.transform().matrix().into();
        let transform = transform * curr_trans;

        if let Some(mesh) = node.mesh() {
            for prim in mesh.primitives() {
                // vertices
                let position_accessor = prim
                    .get(&gltf::mesh::Semantic::Positions)
                    .context("Primitives has no semantic POSITION")?;

                let vertex_count = position_accessor.count();
                let mut vertices = vec![MeshVertex::default(); vertex_count];

                let position_data = gltf_scene.data_of_accessor(&position_accessor)?;
                let position_data = position_data.as_slice_of::<f32>().unwrap();
                for i in 0..vertex_count {
                    vertices[i].position[0] = position_data[3 * i];
                    vertices[i].position[1] = position_data[3 * i + 1];
                    vertices[i].position[2] = position_data[3 * i + 2];
                }
                // texcoords (may be normalized u8 or u16)
                prim.get(&gltf::mesh::Semantic::TexCoords(0))
                    .map(|accessor| {
                        if accessor.data_type() != gltf::accessor::DataType::F32 {
                            return;
                        }
                        if let Ok(data) = gltf_scene.data_of_accessor(&accessor) {
                            let data = data.as_slice_of::<f32>().unwrap();
                            for i in 0..vertex_count {
                                vertices[i].texcoords[0] = data[2 * i];
                                vertices[i].texcoords[1] = data[2 * i + 1];
                            }
                        }
                    });
                // normal
                prim.get(&gltf::mesh::Semantic::Normals).map(|accessor| {
                    if let Ok(data) = gltf_scene.data_of_accessor(&accessor) {
                        let data = data.as_slice_of::<f32>().unwrap();
                        for i in 0..vertex_count {
                            vertices[i].normal[0] = data[3 * i];
                            vertices[i].normal[1] = data[3 * i + 1];
                            vertices[i].normal[2] = data[3 * i + 2];
                        }
                    }
                });
                // tangent
                prim.get(&gltf::mesh::Semantic::Tangents).map(|accessor| {
                    if let Ok(data) = gltf_scene.data_of_accessor(&accessor) {
                        let data = data.as_slice_of::<f32>().unwrap();
                        for i in 0..vertex_count {
                            vertices[i].tangent[0] = data[4 * i];
                            vertices[i].tangent[1] = data[4 * i + 1];
                            vertices[i].tangent[2] = data[4 * i + 2];
                            vertices[i].tangent[3] = data[4 * i + 3];
                        }
                    }
                });
                // color (may be normalized u8 or u16)
                prim.get(&gltf::mesh::Semantic::Colors(0)).map(|accessor| {
                    if accessor.data_type() != gltf::accessor::DataType::F32 {
                        return;
                    }
                    if let Ok(data) = gltf_scene.data_of_accessor(&accessor) {
                        let data = data.as_slice_of::<f32>().unwrap();
                        if accessor.dimensions() == gltf::accessor::Dimensions::Vec3 {
                            for i in 0..vertex_count {
                                vertices[i].color[0] = data[3 * i];
                                vertices[i].color[1] = data[3 * i + 1];
                                vertices[i].color[2] = data[3 * i + 2];
                            }
                        } else if accessor.dimensions() == gltf::accessor::Dimensions::Vec4 {
                            for i in 0..vertex_count {
                                vertices[i].color[0] = data[4 * i];
                                vertices[i].color[1] = data[4 * i + 1];
                                vertices[i].color[2] = data[4 * i + 2];
                                vertices[i].color[3] = data[4 * i + 3];
                            }
                        }
                    }
                });

                // indicess
                let index_accessor = prim.indices().context("Primitives has no indices")?;
                let index_count = index_accessor.count();
                let mut indices = vec![0; index_count];
                let index_data = gltf_scene.data_of_accessor(&index_accessor)?;
                if index_accessor.data_type() == gltf::accessor::DataType::U32 {
                    let index_data = index_data.as_slice_of::<u32>()?;
                    for i in 0..index_count {
                        indices[i] = index_data[i];
                    }
                } else if index_accessor.data_type() == gltf::accessor::DataType::U16 {
                    let index_data = index_data.as_slice_of::<u16>()?;
                    for i in 0..index_count {
                        indices[i] = index_data[i] as u32;
                    }
                }

                // material
                let gltf_material = prim.material();
                let gltf_material_name = gltf_material.name().unwrap();
                if let Some(material) = self.materials.get_mut(gltf_material_name) {
                    let shader = self.shaders.get(&material.shader).unwrap(); // TODO - unwrap
                    let pbr_mr = gltf_material.pbr_metallic_roughness();
                    material.set_vec4("base_color".to_string(), pbr_mr.base_color_factor(), shader);
                    let is_fully_transparent = pbr_mr.base_color_factor()[3].abs() < 0.001;

                    if let Some(info) = pbr_mr.base_color_texture() {
                        material.set_texture(
                            "base_color_tex".to_string(),
                            gltf_texture_to_wgpu_texture(
                                &self.graphics_state.device,
                                &self.graphics_state.queue,
                                &info.texture(),
                                true,
                                gltf_scene,
                            ),
                            &TextureProperty::Texture2D,
                            shader,
                        );
                    } else {
                        material.set_texture(
                            "base_color_tex".to_string(),
                            Texture::white1x1(
                                &self.graphics_state.device,
                                &self.graphics_state.queue,
                            ),
                            &TextureProperty::Texture2D,
                            shader,
                        );
                    }
                    material.set_vec3(
                        "emissive_factor".to_string(),
                        gltf_material.emissive_factor(),
                        shader,
                    );
                    material.set_float(
                        "metallic_factor".to_string(),
                        pbr_mr.metallic_factor(),
                        shader,
                    );
                    material.set_float(
                        "roughness_factor".to_string(),
                        pbr_mr.roughness_factor(),
                        shader,
                    );
                    if let Some(info) = pbr_mr.metallic_roughness_texture() {
                        material.set_texture(
                            "metallic_roughness_tex".to_string(),
                            gltf_texture_to_wgpu_texture(
                                &self.graphics_state.device,
                                &self.graphics_state.queue,
                                &info.texture(),
                                false,
                                gltf_scene,
                            ),
                            &TextureProperty::Texture2D,
                            shader,
                        );
                    } else {
                        material.set_texture(
                            "metallic_roughness_tex".to_string(),
                            Texture::black1x1(
                                &self.graphics_state.device,
                                &self.graphics_state.queue,
                            ),
                            &TextureProperty::Texture2D,
                            shader,
                        );
                    }
                    if let Some(info) = gltf_material.emissive_texture() {
                        material.set_texture(
                            "emissive_tex".to_string(),
                            gltf_texture_to_wgpu_texture(
                                &self.graphics_state.device,
                                &self.graphics_state.queue,
                                &info.texture(),
                                true,
                                gltf_scene,
                            ),
                            &TextureProperty::Texture2D,
                            shader,
                        );
                    } else {
                        material.set_texture(
                            "emissive_tex".to_string(),
                            Texture::black1x1(
                                &self.graphics_state.device,
                                &self.graphics_state.queue,
                            ),
                            &TextureProperty::Texture2D,
                            shader,
                        );
                    }
                    if let Some(info) = gltf_material.normal_texture() {
                        material.set_texture(
                            "normal_tex".to_string(),
                            gltf_texture_to_wgpu_texture(
                                &self.graphics_state.device,
                                &self.graphics_state.queue,
                                &info.texture(),
                                false,
                                gltf_scene,
                            ),
                            &TextureProperty::Texture2D,
                            shader,
                        );
                    } else {
                        material.set_texture(
                            "normal_tex".to_string(),
                            Texture::normal1x1(
                                &self.graphics_state.device,
                                &self.graphics_state.queue,
                            ),
                            &TextureProperty::Texture2D,
                            shader,
                        );
                    }

                    if !is_fully_transparent {
                        material.build(
                            &self.graphics_state.device,
                            &shader.bind_group_layout.as_ref().unwrap(),
                        );
                        let mut mesh =
                            Mesh::new(vertices, indices, transform, gltf_material_name.to_string());
                        mesh.build(
                            &self.graphics_state.device,
                            &self.graphics_state.object_bind_group_layout,
                        );
                        self.meshes.push(mesh);
                    }
                } else {
                    // TODO - default material
                    eprintln!("Can't find material '{}'", gltf_material_name);
                }
            }
        }

        for ch in node.children() {
            self.parse_gltf_node(&ch, gltf_scene, transform)?;
        }

        Ok(())
    }
}

fn gltf_texture_to_wgpu_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    tex: &gltf::texture::Texture,
    is_srgb: bool,
    gltf_scene: &GltfScene,
) -> Texture {
    let image = &gltf_scene.images[tex.index()];
    let image_size = image.width as usize * image.height as usize;
    match image.format {
        gltf::image::Format::R8G8B8 | gltf::image::Format::B8G8R8 => {
            let modified_rgb8_data = rgb8_to_rgba8(&image.pixels, image_size);
            Texture::from_bytes_2d(
                device,
                queue,
                &modified_rgb8_data,
                image.width,
                image.height,
                gltf_format_to_wgpu_format(image.format, is_srgb),
                &gltf_sampler_to_wgpu_sampler(&tex.sampler()),
                Some("GLTF Texture 2D"),
            )
        }
        gltf::image::Format::R16G16B16 => {
            let modified_rgb16_data = rgb16_to_rgba16(&image.pixels, image_size);
            Texture::from_bytes_2d(
                device,
                queue,
                &modified_rgb16_data,
                image.width,
                image.height,
                gltf_format_to_wgpu_format(image.format, is_srgb),
                &gltf_sampler_to_wgpu_sampler(&tex.sampler()),
                Some("GLTF Texture 2D"),
            )
        }
        _ => Texture::from_bytes_2d(
            device,
            queue,
            &image.pixels,
            image.width,
            image.height,
            gltf_format_to_wgpu_format(image.format, is_srgb),
            &gltf_sampler_to_wgpu_sampler(&tex.sampler()),
            Some("GLTF Texture 2D"),
        ),
    }
}

fn rgb8_to_rgba8(orig_data: &[u8], size: usize) -> Vec<u8> {
    let mut data = vec![0; 4 * size];
    for i in 0..size {
        data[4 * i] = orig_data[3 * i];
        data[4 * i + 1] = orig_data[3 * i + 1];
        data[4 * i + 2] = orig_data[3 * i + 2];
        data[4 * i + 3] = 255;
    }
    data
}
fn rgb16_to_rgba16(orig_data: &[u8], size: usize) -> Vec<u8> {
    let mut data = vec![0; 8 * size];
    for i in 0..size {
        data[8 * i] = orig_data[6 * i];
        data[8 * i + 1] = orig_data[6 * i + 1];
        data[8 * i + 2] = orig_data[6 * i + 2];
        data[8 * i + 3] = orig_data[6 * i + 3];
        data[8 * i + 4] = orig_data[6 * i + 4];
        data[8 * i + 5] = orig_data[6 * i + 5];
        data[8 * i + 6] = 0;
        data[8 * i + 7] = 60;
    }
    data
}

fn gltf_format_to_wgpu_format(format: gltf::image::Format, is_srgb: bool) -> wgpu::TextureFormat {
    match format {
        Format::R8 => wgpu::TextureFormat::R8Unorm,
        Format::R8G8 => wgpu::TextureFormat::Rg8Unorm,
        Format::R8G8B8 | Format::R8G8B8A8 => {
            if is_srgb {
                wgpu::TextureFormat::Rgba8UnormSrgb
            } else {
                wgpu::TextureFormat::Rgba8Unorm
            }
        }
        Format::B8G8R8 | Format::B8G8R8A8 => {
            if is_srgb {
                wgpu::TextureFormat::Bgra8UnormSrgb
            } else {
                wgpu::TextureFormat::Bgra8Unorm
            }
        }
        Format::R16 => wgpu::TextureFormat::R16Float,
        Format::R16G16 => wgpu::TextureFormat::Rg16Float,
        Format::R16G16B16 | Format::R16G16B16A16 => wgpu::TextureFormat::Rgba16Float,
    }
}

fn gltf_sampler_to_wgpu_sampler<'a>(
    gltf_sampler: &gltf::texture::Sampler,
) -> wgpu::SamplerDescriptor<'a> {
    let address_mode_u = match gltf_sampler.wrap_s() {
        WrappingMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
        WrappingMode::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
        WrappingMode::Repeat => wgpu::AddressMode::Repeat,
    };
    let address_mode_v = match gltf_sampler.wrap_t() {
        WrappingMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
        WrappingMode::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
        WrappingMode::Repeat => wgpu::AddressMode::Repeat,
    };
    let mag_filter = match gltf_sampler.mag_filter() {
        Some(mag_filter) => match mag_filter {
            MagFilter::Nearest => wgpu::FilterMode::Nearest,
            MagFilter::Linear => wgpu::FilterMode::Linear,
        },
        _ => wgpu::FilterMode::Nearest,
    };
    let (min_filter, mipmap_filter) = match gltf_sampler.min_filter() {
        Some(min_filter) => match min_filter {
            MinFilter::Nearest => (wgpu::FilterMode::Nearest, wgpu::FilterMode::Nearest),
            MinFilter::Linear => (wgpu::FilterMode::Linear, wgpu::FilterMode::Nearest),
            MinFilter::NearestMipmapNearest => {
                (wgpu::FilterMode::Nearest, wgpu::FilterMode::Nearest)
            }
            MinFilter::LinearMipmapNearest => (wgpu::FilterMode::Linear, wgpu::FilterMode::Nearest),
            MinFilter::NearestMipmapLinear => (wgpu::FilterMode::Nearest, wgpu::FilterMode::Linear),
            MinFilter::LinearMipmapLinear => (wgpu::FilterMode::Linear, wgpu::FilterMode::Linear),
        },
        _ => (wgpu::FilterMode::Nearest, wgpu::FilterMode::Nearest),
    };

    wgpu::SamplerDescriptor {
        address_mode_u,
        address_mode_v,
        mag_filter,
        min_filter,
        mipmap_filter,
        ..Default::default()
    }
}