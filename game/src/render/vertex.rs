use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex2d {
    position: glam::Vec2,
    uv: glam::Vec2,
}

impl Vertex2d {
    pub const VERTEX_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as _,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
        ],
    };

    pub fn new(position: glam::Vec2, uv: glam::Vec2) -> Self {
        Self { position, uv }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct InstanceColor2d {
    position: glam::Vec2,
    color: [f32;4], // glam::Vec4 is not 4 f32s
}

impl InstanceColor2d {
    pub const VERTEX_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as _,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &wgpu::vertex_attr_array![
            2 => Float32x2,
            3 => Float32x4,
        ],
    };

    pub fn new(position: glam::Vec2, color: glam::Vec4) -> Self {
        Self {
            position,
            color: color.into(),
        }
    }
}