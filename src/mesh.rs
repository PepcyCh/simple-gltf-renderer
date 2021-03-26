use wgpu::util::DeviceExt;

use crate::vertex::MeshVertex;
use cgmath::prelude::*;
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

    pub fn calc_tangents(&mut self) {
        let vertex_count = self.vertices.len();
        let mut tangents_sum = vec![cgmath::Vector3::zero(); vertex_count];

        let triangle_count = self.indices.len() / 3;
        for i in 0..triangle_count {
            let i0 = self.indices[3 * i] as usize;
            let i1 = self.indices[3 * i + 1] as usize;
            let i2 = self.indices[3 * i + 2] as usize;

            let p0: cgmath::Point3<f32> = self.vertices[i0].position.into();
            let p1: cgmath::Point3<f32> = self.vertices[i1].position.into();
            let p2: cgmath::Point3<f32> = self.vertices[i2].position.into();
            let e1 = p1 - p0;
            let e2 = p2 - p0;

            let uv0: cgmath::Point2<f32> = self.vertices[i0].texcoords.into();
            let uv1: cgmath::Point2<f32> = self.vertices[i1].texcoords.into();
            let uv2: cgmath::Point2<f32> = self.vertices[i2].texcoords.into();
            let u1 = uv1 - uv0;
            let u2 = uv2 - uv0;

            let f = 1.0 / (u1.x * u2.y - u1.y * u2.x);
            let t = cgmath::Vector3::new(
                f * (u2.y * e1.x - u1.y * e2.x),
                f * (u2.y * e1.y - u1.y * e2.y),
                f * (u2.y * e1.z - u1.y * e2.z),
            );
            let t = t.normalize();
            tangents_sum[i0] += t;
            tangents_sum[i1] += t;
            tangents_sum[i2] += t;
        }

        for i in 0..vertex_count {
            let tangent = tangents_sum[i].normalize();
            let tangent = cgmath::Vector4::new(tangent.x, tangent.y, tangent.z, 1.0);
            self.vertices[i].tangent = tangent.into();
        }
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
