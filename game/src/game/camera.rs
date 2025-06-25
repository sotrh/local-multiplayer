pub trait Camera {
    fn view(&self) -> glam::Mat4;
    fn proj(&self) -> glam::Mat4;
    fn view_proj(&self) -> glam::Mat4 {
        self.proj() * self.view()
    }
}

pub struct Camera2d {
    pub(crate) width: f32,
    pub(crate) height: f32,
    position: glam::Vec2,
}

impl Camera2d {
    pub fn new(width: f32, height: f32, position: glam::Vec2) -> Self {
        Self {
            width,
            height,
            position,
        }
    }
}

impl Camera for Camera2d {
    fn view(&self) -> glam::Mat4 {
        let translation = glam::vec3(-self.position.x, -self.position.y, 0.0);
        glam::Mat4::from_translation(translation)
    }

    fn proj(&self) -> glam::Mat4 {
        let hx = self.width * 0.5;
        let hy = self.height * 0.5;
        glam::Mat4::orthographic_rh(
            self.position.x - hx,
            self.position.x + hx,
            self.position.y - hy,
            self.position.y + hy,
            0.0,
            1.0,
        )
    }
}
