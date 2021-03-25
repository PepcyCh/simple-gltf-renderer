use wgpu::util::DeviceExt;

use crate::vertex::MeshVertex;
use cgmath::{Matrix, SquareMatrix};

pub struct Mesh {
    vertices: Vec<MeshVertex>,
    indices: Vec<u32>,
    transform: cgmath::Matrix4<f32>,
    pub material: String,
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub index_buffer: Option<wgpu::Buffer>,
    uniform: MeshUniform,
    uniform_buffer: Option<wgpu::Buffer>,
    pub bind_group: Option<wgpu::BindGroup>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshUniform {
    transform: [[f32; 4]; 4],
    transform_iv: [[f32; 4]; 4],
}

impl Mesh {
    pub fn new(
        vertices: Vec<MeshVertex>,
        indices: Vec<u32>,
        transform: cgmath::Matrix4<f32>,
        material: String,
    ) -> Self {
        Self {
            vertices,
            indices,
            transform,
            material,
            uniform: MeshUniform {
                transform: transform.into(),
                transform_iv: transform.transpose().invert().unwrap().into(),
            },
            vertex_buffer: None,
            index_buffer: None,
            uniform_buffer: None,
            bind_group: None,
        }
    }

    pub fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }

    pub fn build(&mut self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) {
        self.vertex_buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Object Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.vertices),
                usage: wgpu::BufferUsage::VERTEX,
            }),
        );
        self.index_buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Object Index Buffer"),
                contents: bytemuck::cast_slice(&self.indices),
                usage: wgpu::BufferUsage::INDEX,
            }),
        );
        self.uniform_buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Object Uniform Buffer"),
                contents: bytemuck::cast_slice(&[self.uniform]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            }),
        );
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Object Bing d Group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.uniform_buffer.as_ref().unwrap().as_entire_binding(),
            }],
        }))
    }
}
