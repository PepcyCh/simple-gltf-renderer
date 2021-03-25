use std::collections::HashMap;

use byte_slice_cast::AsByteSlice;
use wgpu::util::DeviceExt;

use crate::shader::{Shader, TextureProperty};
use crate::texture::Texture;

pub struct Material {
    pub name: String,
    pub shader: String,
    uniform_bytes: Vec<u8>,
    uniform_buffer: Option<wgpu::Buffer>,
    textures: Vec<(String, Texture)>,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl Material {
    pub fn new(name: String, shader: String, uniform_size: usize) -> Self {
        let uniform_bytes = vec![0; uniform_size];
        Self {
            name,
            shader,
            uniform_bytes,
            uniform_buffer: None,
            textures: vec![],
            bind_group: None,
        }
    }

    pub fn set_float(&mut self, name: String, value: f32, shader: &Shader) -> &mut Self {
        if let Some(offset) = shader.get_uniform_offset(&name) {
            let value_bytes = value.to_le_bytes();
            self.uniform_bytes[offset..offset + 4].copy_from_slice(&value_bytes);
        }
        self
    }

    pub fn set_vec2(&mut self, name: String, value: [f32; 2], shader: &Shader) -> &mut Self {
        if let Some(offset) = shader.get_uniform_offset(&name) {
            let value_bytes = value.as_byte_slice();
            self.uniform_bytes[offset..offset + 8].copy_from_slice(&value_bytes);
        }
        self
    }

    pub fn set_vec3(&mut self, name: String, value: [f32; 3], shader: &Shader) -> &mut Self {
        if let Some(offset) = shader.get_uniform_offset(&name) {
            let value_bytes = value.as_byte_slice();
            self.uniform_bytes[offset..offset + 12].copy_from_slice(&value_bytes);
        }
        self
    }

    pub fn set_vec4(&mut self, name: String, value: [f32; 4], shader: &Shader) -> &mut Self {
        if let Some(offset) = shader.get_uniform_offset(&name) {
            let value_bytes = value.as_byte_slice();
            self.uniform_bytes[offset..offset + 16].copy_from_slice(&value_bytes);
        }
        self
    }

    pub fn set_mat3(
        &mut self,
        name: String,
        value: cgmath::Matrix3<f32>,
        shader: &Shader,
    ) -> &mut Self {
        if let Some(offset) = shader.get_uniform_offset(&name) {
            let value_arr = [
                value.x.x, value.x.y, value.x.z, 0.0, value.y.x, value.y.y, value.y.z, 0.0,
                value.z.x, value.z.y, value.z.z, 0.0,
            ];
            let value_bytes = value_arr.as_byte_slice();
            self.uniform_bytes[offset..offset + 48].copy_from_slice(&value_bytes);
        }
        self
    }

    pub fn set_mat4(
        &mut self,
        name: String,
        value: cgmath::Matrix4<f32>,
        shader: &Shader,
    ) -> &mut Self {
        if let Some(offset) = shader.get_uniform_offset(&name) {
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

    pub fn set_texture(
        &mut self,
        name: String,
        value: Texture,
        ty: &TextureProperty,
        shader: &Shader,
    ) -> &mut Self {
        shader
            .texture_properties
            .iter()
            .find(|(self_name, _)| *self_name == name)
            .map(|(_, texture)| {
                if texture == ty {
                    self.textures.push((name, value));
                }
            });
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
        let mut curr_binding = 1;
        for (_, tex) in &self.textures {
            entries.push(wgpu::BindGroupEntry {
                binding: curr_binding,
                resource: wgpu::BindingResource::TextureView(&tex.view),
            });
            curr_binding += 1;
            entries.push(wgpu::BindGroupEntry {
                binding: curr_binding,
                resource: wgpu::BindingResource::Sampler(&tex.sampler),
            });
            curr_binding += 1;
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
