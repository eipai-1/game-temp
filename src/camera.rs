// --
use cgmath::*;
use std::f32::consts::FRAC_PI_2;
use winit::{
    event::*,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::realm;

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

pub const ZERO: f32 = 1e-6;

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

    //返回方向向量
    pub fn direction(&self) -> Vector3<f32> {
        Vector3 {
            x: self.yaw.0.cos() * self.pitch.0.cos(),
            y: self.pitch.0.sin(),
            z: self.yaw.0.sin() * self.pitch.0.cos(),
        }
        .normalize()
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_to_rh(self.position, self.direction(), Vector3::unit_y())
    }

    pub fn reset(&mut self) {
        self.position = Point3::new(0.0, 7.0, 0.0);
        self.yaw = Rad(0.0);
        self.pitch = Rad(-45.0);
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
    pub selected_block: Option<Point3<i32>>,
    pub pre_selected_block: Option<Point3<i32>>,
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
            fov_sensitivity: 0.003,
            dx: 0.0,
            dy: 0.0,
            selected_block: None,
            pre_selected_block: None,
        }
    }

    pub fn process_events(
        &mut self,
        event: &WindowEvent,
        camera: &mut Camera,
        realm: &mut realm::Realm,
        queue: &wgpu::Queue,
    ) -> bool {
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
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                //println!("left mouse button pressed");
                if let Some(selected_block) = self.selected_block {
                    realm.destory_block(selected_block, queue);
                }
                true
            }
            WindowEvent::CursorMoved { position, .. } if self.is_fov => {
                self.dx = (self.center_x as f64 - position.x) as f32;
                self.dy = (self.center_y as f64 - position.y) as f32;
                camera.yaw -= Rad(self.dx) * self.fov_sensitivity;
                camera.pitch += Rad(self.dy) * self.fov_sensitivity;
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

    pub fn update_camera(&mut self, camera: &mut Camera, dt: f32, data: &mut realm::RealmData) {
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
        };

        self.update_wf(camera, data);
    }

    fn update_wf(&mut self, camera: &Camera, data: &mut realm::RealmData) {
        match dda(camera.direction(), camera.position, data) {
            Some(new_position) => {
                data.is_wf_visible = true;
                data.update_wf_uniform(new_position.0);
                self.selected_block = Some(new_position.0);
                self.pre_selected_block = Some(new_position.1);
            }
            None => {
                data.is_wf_visible = false;
                self.selected_block = None;
                self.pre_selected_block = None;
            }
        }
    }
}

//0号为选中方块，1号为选中方块的前一个方块
fn dda(
    direction: Vector3<f32>,
    origin: Point3<f32>,
    data: &realm::RealmData,
) -> Option<(Point3<i32>, Point3<i32>)> {
    let mut current_block = origin.map(|x| x.floor() as i32);
    let mut pre_block = current_block;

    let step = direction.map(|x| if x > 0.0 { 1 } else { -1 });

    let (mut t_max, t_delta) = {
        let mut t_max = Vector3::zero();
        let mut t_delta = Vector3::zero();

        for i in 0..3 {
            let dir_component = direction[i];
            let origin_component = origin[i];
            let cv_component = current_block[i] as f32;

            if dir_component.abs() < ZERO {
                t_max[i] = f32::INFINITY;
                t_delta[i] = f32::INFINITY;
            } else {
                let next_bound = if dir_component > 0.0 {
                    cv_component + 1.0
                } else {
                    cv_component
                };
                t_max[i] = (next_bound - origin_component) / dir_component;
                t_delta[i] = 1.0 / dir_component.abs();
            }
        }
        (t_max, t_delta)
    };

    for _ in 0..data.wf_max_len as u32 {
        let axis = if t_max.x < t_max.y {
            if t_max.x < t_max.z {
                0
            } else {
                2
            }
        } else {
            if t_max.y < t_max.z {
                1
            } else {
                2
            }
        };

        current_block[axis] += step[axis];
        t_max[axis] += t_delta[axis];
        let block = data.get_block(current_block);
        if block.tp != realm::BlockType::Empty {
            return Some((current_block, pre_block));
        }

        pre_block = current_block;
    }

    None
}

#[cfg(test)]
mod tests {
    use crate::{
        camera::{dda, Camera},
        realm,
    };
    use cgmath::*;

    #[test]
    fn test_dda1() {
        let mut data = realm::RealmData::new();
        let mut position = Point3 {
            x: 0.5,
            y: 0.5,
            z: -1.0,
        };

        let mut camera = Camera::new(position, Deg(90.0), Deg(45.0));
        let mut ans = Point3 { x: 0, y: 1, z: 0 };
        assert_eq!(
            dda(camera.direction(), camera.position, &mut data)
                .unwrap()
                .0,
            ans
        );

        position = Point3 {
            x: 0.5,
            y: 1.5,
            z: -1.0,
        };
        camera = Camera::new(position, Deg(90.0), Deg(-45.0));
        ans = Point3 { x: 0, y: 0, z: 0 };
        assert_eq!(
            dda(camera.direction(), camera.position, &data).unwrap().0,
            ans
        );
    }
}
