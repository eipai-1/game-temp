// --
use cgmath::*;
use std::f32::consts::FRAC_PI_2;
use winit::{
    event::*,
    keyboard::{KeyCode, PhysicalKey},
};

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
}

impl Camera {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_to_rh(
            self.position,
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y(),
        )
    }
}

pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub struct CameraController {
    pub is_fov: bool,
    pub center_x: u32,
    pub center_y: u32,
    pub speed: f32,
    pub is_forward_pressed: bool,
    pub is_backward_pressed: bool,
    pub is_left_pressed: bool,
    pub is_right_pressed: bool,
    pub is_up_pressed: bool,
    pub is_down_pressed: bool,
    pub fov_sensitivity: f32,
    dx: f32,
    dy: f32,
}

impl CameraController {
    pub fn new(speed: f32, center_x: u32, center_y: u32) -> Self {
        Self {
            is_fov: true,
            center_x,
            center_y,
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            fov_sensitivity: 0.5,
            dx: 0.0,
            dy: 0.0,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent, camera: &mut Camera, dt: f32) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        repeat: false,
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    KeyCode::KeyW | KeyCode::ArrowUp => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyA | KeyCode::ArrowLeft => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyS | KeyCode::ArrowDown => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyD | KeyCode::ArrowRight => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    KeyCode::ShiftLeft => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    KeyCode::Space => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyE => {
                        if is_pressed {
                            self.is_fov = !self.is_fov;
                        }
                        true
                    }
                    _ => false,
                }
            }
            WindowEvent::Focused(true) => {
                self.is_fov = true;
                true
            }
            WindowEvent::Focused(false) => {
                self.is_fov = false;
                true
            }
            WindowEvent::CursorMoved { position, .. } if self.is_fov => {
                self.dx = (self.center_x as f64 - position.x) as f32;
                self.dy = (self.center_y as f64 - position.y) as f32;
                camera.yaw -= Rad(self.dx) * self.fov_sensitivity * dt;
                camera.pitch += Rad(self.dy) * self.fov_sensitivity * dt;
                if camera.pitch < -Rad(SAFE_FRAC_PI_2) {
                    camera.pitch = -Rad(SAFE_FRAC_PI_2);
                } else if camera.pitch > Rad(SAFE_FRAC_PI_2) {
                    camera.pitch = Rad(SAFE_FRAC_PI_2);
                }
                true
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera, dt: f32) {
        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        //为什么这个是右边？？这不是左边吗？？
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();

        if self.is_forward_pressed {
            camera.position += forward * self.speed * dt;
        }
        if self.is_backward_pressed {
            camera.position -= forward * self.speed * dt;
        }
        if self.is_left_pressed {
            camera.position -= right * self.speed * dt;
        }
        if self.is_right_pressed {
            camera.position += right * self.speed * dt;
        }
        if self.is_up_pressed {
            camera.position.y += self.speed * dt;
        }
        if self.is_down_pressed {
            camera.position.y -= self.speed * dt;
        }
        // Keep the camera's angle from going too high/low.
    }
}
