use crate::math::{Mat4, Quat, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub orientation: Quat,
    pub fov_y_radians: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Camera {
            position: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            fov_y_radians: 60.0f32.to_radians(),
            aspect,
            near: 0.1,
            far: 5000.0,
        }
    }

    pub fn forward(&self) -> Vec3 {
        self.orientation.rotate_vec3(Vec3::FORWARD)
    }

    pub fn up(&self) -> Vec3 {
        self.orientation.rotate_vec3(Vec3::UP)
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at(self.position, self.position + self.forward(), self.up())
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective(self.fov_y_radians, self.aspect, self.near, self.far)
    }

    pub fn view_proj(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}
