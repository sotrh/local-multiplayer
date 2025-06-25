use bytemuck::{Pod, Zeroable};

use crate::game::camera::Camera;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraData {
    view_proj: glam::Mat4,
}

impl CameraData {
    pub const IDENTITY: Self = Self {
        view_proj: glam::Mat4::IDENTITY,
    };

    pub fn update(&mut self, camera: &impl Camera) {
        self.view_proj = camera.view_proj();
    }
}