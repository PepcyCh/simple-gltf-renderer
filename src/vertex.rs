#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub texcoords: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 4],
    pub color: [f32; 4],
}

impl MeshVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<MeshVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    // position
                    format: wgpu::VertexFormat::Float3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    // texcoord
                    format: wgpu::VertexFormat::Float2,
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    // normal
                    format: wgpu::VertexFormat::Float3,
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    // tangent
                    format: wgpu::VertexFormat::Float4,
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    // color
                    format: wgpu::VertexFormat::Float4,
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 4,
                },
            ],
        }
    }
}

impl Default for MeshVertex {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            texcoords: [0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
            color: [0.0, 0.0, 0.0, 1.0],
        }
    }
}
