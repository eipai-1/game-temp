// --
use crate::{game_config::ZERO, physics};

use cgmath::*;
use std::f32::consts::FRAC_PI_2;
use winit::{
    event::*,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{game_config, realm};

pub const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    view_position: [f32; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        self.view_position = camera.position.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into();
    }
}

#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
    pub forward: Vector3<f32>,
    pub right: Vector3<f32>,
}

impl Camera {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        let yaw = yaw.into();
        let pitch = pitch.into();
        let forward = Vector3::new(
            yaw.0.cos() * pitch.0.cos(),
            pitch.0.sin(),
            yaw.0.sin() * pitch.0.cos(),
        )
        .normalize();
        let right = Vector3::new(-forward.z, 0.0, forward.x).normalize();

        Self {
            position: position.into(),
            yaw,
            pitch,
            forward,
            right,
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
    pub dx: f32,
    pub dy: f32,
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
}

//0号为选中方块，1号为选中方块的前一个方块
pub fn dda(
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
