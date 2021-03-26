use byte_slice_cast::AsByteSlice;
use wgpu::util::DeviceExt;

use crate::shader::{Shader, TextureProperty};
use crate::texture::Texture;
use std::collections::HashMap;

pub struct Material {
    pub name: String,
    pub shader: String,
    uniform_bytes: Vec<u8>,
    uniform_offsets: HashMap<String, usize>,
    uniform_buffer: Option<wgpu::Buffer>,
    textures: HashMap<String, Texture>,
    textures_index: HashMap<String, u32>,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl Material {
    pub fn from_shader(
        name: String,
        shader: &Shader,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let uniform_bytes = vec![0; shader.uniform_size];
        let mut textures = HashMap::new();
        for (tex_name, tex_ty) in &shader.texture_properties {
            // TODO - different value according to json
            // TODO - different value according to tex_ty
            let default_tex = match tex_ty {
                TextureProperty::Texture2D(default) => match default.as_str() {
                    "white" => Texture::white1x1(device, queue),
                    "black" => Texture::black1x1(device, queue),
                    "gray" | "grey" => Texture::gray1x1(device, queue),
                    "normal" => Texture::normal1x1(device, queue),
                    _ => Texture::black1x1(device, queue),
                },
                _ => Texture::white1x1(device, queue),
            };
            textures.insert(tex_name.clone(), default_tex);
        }
        // TODO - maybe these 'clone()'s can be removed by using Rc/Arc
        Self {
            name,
            shader: shader.name.clone(),
            uniform_bytes,
            uniform_offsets: shader.uniform_offsets.clone(),
            uniform_buffer: None,
            textures,
            textures_index: shader.textures_index.clone(),
            bind_group: None,
        }
    }

    pub fn set_float(&mut self, name: &str, value: f32) -> &mut Self {
        if let Some(offset) = self.uniform_offsets.get(name).cloned() {
            let value_bytes = value.to_le_bytes();
            self.uniform_bytes[offset..offset + 4].copy_from_slice(&value_bytes);
        }
        self
    }

    pub fn set_vec2(&mut self, name: &str, value: [f32; 2]) -> &mut Self {
        if let Some(offset) = self.uniform_offsets.get(name).cloned() {
            let value_bytes = value.as_byte_slice();
            self.uniform_bytes[offset..offset + 8].copy_from_slice(&value_bytes);
        }
        self
    }

    pub fn set_vec3(&mut self, name: &str, value: [f32; 3]) -> &mut Self {
        if let Some(offset) = self.uniform_offsets.get(name).cloned() {
            let value_bytes = value.as_byte_slice();
            self.uniform_bytes[offset..offset + 12].copy_from_slice(&value_bytes);
        }
        self
    }

    pub fn set_vec4(&mut self, name: &str, value: [f32; 4]) -> &mut Self {
        if let Some(offset) = self.uniform_offsets.get(name).cloned() {
            let value_bytes = value.as_byte_slice();
            self.uniform_bytes[offset..offset + 16].copy_from_slice(&value_bytes);
        }
        self
    }

    pub fn set_mat3(&mut self, name: &str, value: cgmath::Matrix3<f32>) -> &mut Self {
        if let Some(offset) = self.uniform_offsets.get(name).cloned() {
            let value_arr = [
                value.x.x, value.x.y, value.x.z, 0.0, value.y.x, value.y.y, value.y.z, 0.0,
                value.z.x, value.z.y, value.z.z, 0.0,
            ];
            let value_bytes = value_arr.as_byte_slice();
            self.uniform_bytes[offset..offset + 48].copy_from_slice(&value_bytes);
        }
        self
    }

    pub fn set_mat4(&mut self, name: &str, value: cgmath::Matrix4<f32>) -> &mut Self {
        if let Some(offset) = self.uniform_offsets.get(name).cloned() {
            let value_arr = [
                value.x.x, value.x.y, value.x.z, value.x.w, value.y.x, value.y.y, value.y.z,
                value.y.w, value.z.x, value.z.y, value.z.z, value.z.w, value.w.x, value.w.y,
                value.w.z, value.w.w,
            ];
            let value_bytes = value_arr.as_byte_slice();
            self.uniform_bytes[offset..offset + 64].copy_from_slice(&value_bytes);
        }
        self
    }

    pub fn set_texture(&mut self, name: &str, value: Texture) -> &mut Self {
        self.textures.get_mut(name).map(|data| *data = value);
        self
    }

    pub fn build(&mut self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) {
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} Uniform Buffer", &self.name)),
            contents: bytemuck::cast_slice(&self.uniform_bytes),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });
        self.uniform_buffer = Some(uniform_buffer);

        let mut entries = vec![];
        entries.push(wgpu::BindGroupEntry {
            binding: 0,
            resource: self.uniform_buffer.as_ref().unwrap().as_entire_binding(),
        });
        for (name, tex) in &self.textures {
            entries.push(wgpu::BindGroupEntry {
                binding: self.textures_index[name] * 2 + 1,
                resource: wgpu::BindingResource::TextureView(&tex.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: self.textures_index[name] * 2 + 2,
                resource: wgpu::BindingResource::Sampler(&tex.sampler),
            });
        }
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} Bind Group", self.name)),
            layout,
            entries: &entries,
        }));
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.uniform_buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&self.uniform_bytes),
        );
    }
}
