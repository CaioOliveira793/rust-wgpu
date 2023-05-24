use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

pub struct Camera {
    pub eye: glam::Vec3,
    pub target: glam::Vec3,
    pub up: glam::Vec3,
    pub aspect: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl Camera {
    #[rustfmt::skip]
    pub const OPENGL_TO_WGPU_MATRIX: glam::Mat4 = glam::Mat4::from_cols_array(&[
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0,
    ]);

    pub fn view_projection_matrix(&self) -> glam::Mat4 {
        let view = glam::Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = glam::Mat4::perspective_rh(self.fov_y, self.aspect, self.z_near, self.z_far);
        // NOTE: Models centered on (0, 0, 0) will be halfway inside the clipping area.
        // This is only an issue if a camera matrix is not used.
        return Self::OPENGL_TO_WGPU_MATRIX * proj * view;
        // return proj * view;
    }
}

pub struct CameraController {
    pub speed: f32,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self { speed }
    }

    pub fn process_events(&self, camera: &mut Camera, event: &WindowEvent) -> bool {
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
                    camera.eye += self.speed;
                    tracing::info!("key press W");
                    true
                }
                VirtualKeyCode::S => {
                    camera.eye -= self.speed;
                    tracing::info!("key press S");
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }
}
