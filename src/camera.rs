// --
use cgmath::*;
use std::f32::consts::FRAC_PI_2;
use winit::{
    event::*,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::realm;

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

const ZERO: f32 = 1e-6;

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

    pub fn update_camera(&self, camera: &mut Camera, dt: f32, data: &mut realm::RealmData) {
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

        update_wf(camera, data);
    }
}

fn update_wf(camera: &Camera, data: &mut realm::RealmData) {
    match dda(camera.direction(), camera.position, data) {
        Some(new_position) => {
            data.is_wf_visible = true;
            data.update_wf_uniform(new_position);
        }
        None => {
            data.is_wf_visible = false;
        }
    }
}

fn dda(dir: Vector3<f32>, position: Point3<f32>, data: &realm::RealmData) -> Option<Point3<i32>> {
    let mut cur_block = position.map(|x| x.floor() as i32);
    //println!("cur_block:{:?}", cur_block);

    //如果当前卡在方块里面，就不进行射线检测
    if data.get_block_type(cur_block.x, cur_block.y, cur_block.z) != realm::BlockType::Empty {
        //println!("stuck");
        return None;
    }

    let step = dir.map(|x| if x > 0.0 { 1 } else { -1 });

    let t_delta = dir.map(|x| {
        if x.abs() < 1e-6 {
            f32::INFINITY
        } else {
            1.0 / x.abs()
        }
    });

    let mut t = Vector3::new(0.0, 0.0, 0.0);

    //println!("position: {:#?}", position);
    //t = dir.zip(cur_block, |a, b| {
    //    if a.abs() < ZERO {
    //        f32::INFINITY
    //    } else {
    //        if b > 0 {
    //
    //        }
    //    }
    //});
    if dir.x.abs() < ZERO {
        t.x = f32::INFINITY;
    } else {
        if step.x > 0 {
            t.x = (cur_block.x as f32 + 1.0 - position.x) / dir.x;
        } else {
            t.x = (cur_block.x as f32 - position.x) / dir.x
        }
    }
    if dir.y.abs() < ZERO {
        t.y = f32::INFINITY;
    } else {
        if step.y > 0 {
            t.y = (cur_block.y as f32 + 1.0 - position.y) / dir.y;
        } else {
            t.y = (cur_block.y as f32 - position.y) / dir.y
        }
    }
    if dir.z.abs() < ZERO {
        t.z = f32::INFINITY;
    } else {
        if step.z > 0 {
            t.z = (cur_block.z as f32 + 1.0 - position.z) / dir.z;
        } else {
            t.z = (cur_block.z as f32 - position.z) / dir.z
        }
    }
    //println!("t:{:#?}", t);

    let mut traveled = 0.0;

    while traveled < data.wf_max_len {
        let tp = data.get_block_type(cur_block.x, cur_block.y, cur_block.z);
        if tp != realm::BlockType::Empty {
            return Some(cur_block);
        }

        //找出最小的坐标轴
        #[rustfmt::skip]
            let axis = if t.x < t.y {
                if t.x < t.z {0} else {1}
            } else {
                if t.y < t.z {1} else {2}
            };

        match axis {
            0 => {
                cur_block.x += 1 * step.x;
                t.x += t_delta.x as f32;
            }
            1 => {
                cur_block.y += 1 * step.y;
                t.y += t_delta.y as f32;
            }
            2 => {
                cur_block.z += 1 * step.z;
                t.z += t_delta.z as f32;
            }
            _ => {}
        }

        traveled += 1.0;
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
        let dir: Vector3<f32> = Vector3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        };
        let mut position = Point3 {
            x: 0.5,
            y: 0.5,
            z: -1.0,
        };

        let mut camera = Camera::new(position, Deg(90.0), Deg(45.0));
        let mut ans = Point3 { x: 0, y: 1, z: 0 };
        assert_eq!(
            dda(camera.direction(), camera.position, &mut data).unwrap(),
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
            dda(camera.direction(), camera.position, &data).unwrap(),
            ans
        );
    }
}
