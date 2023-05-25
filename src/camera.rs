use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

use crate::util::math::degree_to_radian;

#[derive(Debug)]
pub struct CameraProjection {
    /// Field of View (radians)
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect_ratio: f32,
}

impl CameraProjection {
    pub fn new(fov: f32, near: f32, far: f32, aspect_ratio: f32) -> Self {
        Self {
            fov,
            near,
            far,
            aspect_ratio,
        }
    }

    pub fn get_projection(&self) -> glam::Mat4 {
        glam::Mat4::perspective_rh(self.fov, self.aspect_ratio, self.near, self.far)
    }
}

impl Default for CameraProjection {
    fn default() -> Self {
        Self::new(degree_to_radian(45.0), 0.1, 100.0, 16.0 / 9.0)
    }
}

#[derive(Debug)]
pub struct CameraView {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
}

impl CameraView {
    pub fn new(position: glam::Vec3, rotation: glam::Quat) -> Self {
        Self { position, rotation }
    }

    pub fn get_view(&self) -> glam::Mat4 {
        glam::Mat4::from_rotation_translation(self.rotation, self.position)
    }
}

impl Default for CameraView {
    fn default() -> Self {
        Self {
            position: glam::Vec3::new(0.0, 0.0, -10.0),
            rotation: glam::Quat::IDENTITY,
        }
    }
}

#[derive(Debug, Default)]
pub struct Camera {
    pub view: CameraView,
    pub projection: CameraProjection,
}

impl Camera {
    pub fn view_projection(&self) -> glam::Mat4 {
        self.projection.get_projection() * self.view.get_view()
    }
}

pub struct CameraController {
    pub speed: f32,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self { speed }
    }

    pub fn process_events(
        &self,
        camera: &mut Camera,
        event: &WindowEvent,
        elapsed_time: f32,
    ) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } if *state == ElementState::Pressed => match keycode {
                VirtualKeyCode::W => {
                    camera.view.position.z += self.speed * elapsed_time;
                    true
                }
                VirtualKeyCode::S => {
                    camera.view.position.z -= self.speed * elapsed_time;
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }
}
