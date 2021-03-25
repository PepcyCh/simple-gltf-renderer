use cgmath::InnerSpace;
use wgpu::util::DeviceExt;

pub struct Light {
    uniform: LightUniform,
    uniform_buffer: Option<wgpu::Buffer>,
    pub bind_group: Option<wgpu::BindGroup>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    position: [f32; 4],
    color: [f32; 4],
}

impl Light {
    pub fn point_light(position: cgmath::Point3<f32>, color: [f32; 4]) -> Self {
        Self {
            uniform: LightUniform {
                position: [position.x, position.y, position.z, 1.0],
                color,
            },
            uniform_buffer: None,
            bind_group: None,
        }
    }

    pub fn directional_light(direction: cgmath::Vector3<f32>, color: [f32; 4]) -> Self {
        let direction = (-direction).normalize();
        Self {
            uniform: LightUniform {
                position: [direction.x, direction.y, direction.z, 0.0],
                color,
            },
            uniform_buffer: None,
            bind_group: None,
        }
    }

    pub fn build(&mut self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) {
        self.uniform_buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Light Uniform Buffer"),
                contents: bytemuck::cast_slice(&[self.uniform]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            }),
        );

        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light Bing Group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.uniform_buffer.as_ref().unwrap().as_entire_binding(),
            }],
        }))
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.uniform_buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
    }
}
