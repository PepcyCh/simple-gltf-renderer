use cgmath::prelude::*;
use wgpu::util::DeviceExt;

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    fovy: f32,
    aspect: f32,
    znear: f32,
    zfar: f32,
    uniform: CameraUniform,
    uniform_buffer: Option<wgpu::Buffer>,
    pub bind_group: Option<wgpu::BindGroup>,
    uniform_dirty: bool,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
    eye: [f32; 3],
    _padding: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
    );

    pub fn new(
        eye: cgmath::Point3<f32>,
        target: cgmath::Point3<f32>,
        up: cgmath::Vector3<f32>,
        fovy: f32,
        aspect: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        let view = cgmath::Matrix4::look_at(eye, target, up);
        let proj = Self::OPENGL_TO_WGPU_MATRIX
            * cgmath::perspective(cgmath::Deg(fovy), aspect, znear, zfar);
        Self {
            eye,
            target,
            up,
            fovy,
            aspect,
            znear,
            zfar,
            uniform: CameraUniform {
                view: view.into(),
                proj: proj.into(),
                eye: eye.into(),
                _padding: 0.0,
                znear,
                zfar,
            },
            uniform_buffer: None,
            bind_group: None,
            uniform_dirty: false,
        }
    }

    pub fn move_forward(&mut self, delta: f32) {
        let forward = self.target - self.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        if forward_mag > delta {
            self.eye += forward_norm * delta;
        }

        self.uniform_dirty = true;
    }

    pub fn rotate(&mut self, delta_theta: f32, delta_phi: f32) {
        let forward = (self.target - self.eye).normalize();
        let right = forward.cross(self.up);

        let delta_phi =
            if (delta_phi < 0.0 && forward.y <= -0.98) || (delta_phi > 0.0 && forward.y >= 0.98) {
                0.0
            } else {
                delta_phi
            };
        let rotate_phi = cgmath::Matrix4::from_axis_angle(right, cgmath::Deg(delta_phi));
        let rotate_theta =
            cgmath::Matrix4::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(delta_theta));
        self.eye = rotate_phi.transform_point(rotate_theta.transform_point(self.eye));

        self.uniform_dirty = true;
    }

    pub fn translate(&mut self, delta: cgmath::Vector3<f32>) {
        self.eye += delta;
        self.target += delta;
        self.uniform_dirty = true;
    }

    pub fn set_aspect(&mut self, new_aspect: f32) {
        self.aspect = new_aspect;
        self.uniform_dirty = true;
    }

    pub fn build(&mut self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) {
        self.uniform_buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Uniform Buffer"),
                contents: bytemuck::cast_slice(&[self.uniform]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            }),
        );

        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.uniform_buffer.as_ref().unwrap().as_entire_binding(),
            }],
        }));
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        if self.uniform_dirty {
            self.uniform.eye = self.eye.into();
            self.uniform.view = cgmath::Matrix4::look_at(self.eye, self.target, self.up).into();
            self.uniform.proj = (Self::OPENGL_TO_WGPU_MATRIX
                * cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar))
            .into();

            queue.write_buffer(
                &self.uniform_buffer.as_ref().unwrap(),
                0,
                bytemuck::cast_slice(&[self.uniform]),
            );
            self.uniform_dirty = false;
        }
    }
}
