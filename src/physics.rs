use std::vec;

use cgmath::*;
use wgpu::util::DeviceExt;
use wgpu::*;

use crate::basic_config;
use crate::camera::*;
use crate::control;
use crate::game_config;
use crate::realm;

pub const JUMP_SPEED: f32 = 10.0;
pub const GRAVITATIONAL_ACCELERATION: f32 = 9.8;
pub const DELTA_DISPLACEMENT: f32 = 1e-6;
pub const ZERO_VELOCITY: f32 = 1e-3;

pub enum EntityType {
    Player,
}

pub struct Entity {
    pub position: Point3<f32>,
    pub velocity: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub entity_type: EntityType,
    pub is_grounded: bool,
    model_vertex: Vec<Point3<f32>>,
    min_x_point: f32,
    max_x_point: f32,
    min_y_point: f32,
    max_y_point: f32,
    min_z_point: f32,
    max_z_point: f32,
    pub is_testing: bool,
}

pub struct PlayerEntity {
    pub entity: Entity,
    pub projection: Projection,
    pub camera: Camera,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: Buffer,
    pub camera_bind_group: BindGroup,
    pub camera_bind_group_layout: BindGroupLayout,
    pub camera_controller: CameraController,
    pub is_collided: bool,
    pub is_move_speed_set: bool,
    pub move_velocity: Vector3<f32>,
}

impl PlayerEntity {
    pub fn new(
        basic_config: &basic_config::BasicConfig,
        realm: &realm::Realm,
        game_config: &game_config::GameConfig,
    ) -> Self {
        //创建摄像机
        let mut position = Point3::new(1.0, 1.0, 1.0);
        position.y = realm.get_first_none_empty_block(1.0, 1.0) as f32 + 10.0;
        let camera = Camera::new(position, cgmath::Deg(90.0), cgmath::Deg(-45.0));
        let projection = Projection::new(
            basic_config.config.width,
            basic_config.config.height,
            cgmath::Deg(45.0),
            0.1,
            100.0,
        );
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

        let camera_buffer =
            basic_config
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("First camera buffer"),
                    contents: bytemuck::cast_slice(&[camera_uniform]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let camera_bind_group_layout =
            basic_config
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("camera bind group layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::VERTEX,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let camera_bind_group = basic_config
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("camera bind group"),
                layout: &camera_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: realm.render_res.wf_uniform_buffer.as_entire_binding(),
                    },
                ],
            });

        let camera_controller = CameraController::new(
            game_config.player_speed,
            basic_config.config.width / 2,
            basic_config.config.height / 2,
        );

        let is_collided = false;

        Self {
            entity: Entity::new(position, EntityType::Player),
            camera,
            projection,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,
            camera_controller,
            is_collided,
            is_move_speed_set: false,
            move_velocity: Vector3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn update_camera(&mut self, dt: f32, data: &mut realm::RealmData) {
        self.update_wf(data);
    }

    fn update_wf(&mut self, data: &mut realm::RealmData) {
        match dda(self.camera.direction(), self.camera.position, data) {
            Some(new_position) => {
                data.is_wf_visible = true;
                data.update_wf_uniform(new_position.0);
                self.camera_controller.selected_block = Some(new_position.0);
                self.camera_controller.pre_selected_block = Some(new_position.1);
            }
            None => {
                data.is_wf_visible = false;
                self.camera_controller.selected_block = None;
                self.camera_controller.pre_selected_block = None;
            }
        }
    }

    pub fn update(&mut self, dt: f32, data: &mut realm::RealmData) {
        //if self.is_move_speed_set {
        //每一帧都重新建立一个移动速度
        self.entity.velocity -= self.move_velocity;
        let mut move_velocity: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
        //println!("camera.forward: {:?}", self.camera.forward);
        if self.camera_controller.is_forward_pressed {
            move_velocity += self.camera.forward * self.camera_controller.speed;
        }
        if self.camera_controller.is_backward_pressed {
            move_velocity -= self.camera.forward * self.camera_controller.speed;
        }
        if self.camera_controller.is_left_pressed {
            move_velocity -= self.camera.right * self.camera_controller.speed;
        }
        if self.camera_controller.is_right_pressed {
            move_velocity += self.camera.right * self.camera_controller.speed;
        }
        move_velocity.y = 0.0;
        self.move_velocity = move_velocity;
        self.entity.velocity += self.move_velocity;
        //self.is_move_speed_set = true;
        //}
        self.entity.update(dt, data);
        self.camera.position = self.entity.position;
        //self.camera.position.y += 1.8;
        self.update_camera(dt, data);
        self.camera_uniform
            .update_view_proj(&self.camera, &self.projection);
    }
}

impl Entity {
    pub fn new(position: Point3<f32>, entity_type: EntityType) -> Self {
        let mut model_vertex: Vec<Point3<f32>> = vec![];
        let mut min_x_point = 0.0;
        let mut max_x_point = 0.0;
        let mut min_y_point = 0.0;
        let mut max_y_point = 0.0;
        let mut min_z_point = 0.0;
        let mut max_z_point = 0.0;
        let is_testing = false;
        match entity_type {
            EntityType::Player => {
                model_vertex.push(Point3::new(0.0, 0.0, 0.0));
                model_vertex.push(Point3::new(0.5, 0.0, 0.0));
                model_vertex.push(Point3::new(0.5, 0.0, 0.5));
                model_vertex.push(Point3::new(0.0, 0.0, 0.5));

                model_vertex.push(Point3::new(0.0, -1.8, 0.0));
                model_vertex.push(Point3::new(0.5, -1.8, 0.0));
                model_vertex.push(Point3::new(0.5, -1.8, 0.5));
                model_vertex.push(Point3::new(0.0, -1.8, 0.5));
                min_x_point = 0.0;
                max_x_point = 0.5;
                min_y_point = -1.8;
                max_y_point = 0.0;
                min_z_point = 0.0;
                max_z_point = 0.5;
            }
        }
        let is_grounded = false;
        Self {
            position,
            velocity: Vector3::new(0.0, 0.0, 0.0),
            acceleration: Vector3::new(0.0, 0.0, 0.0),
            entity_type,
            model_vertex,
            min_x_point,
            max_x_point,
            min_y_point,
            max_y_point,
            min_z_point,
            max_z_point,
            is_grounded,
            is_testing,
        }
    }

    pub fn update(&mut self, dt: f32, data: &realm::RealmData) {
        if self.is_grounded {
            self.acceleration.y = 0.0;
        } else {
            self.acceleration.y = -GRAVITATIONAL_ACCELERATION;
        }
        self.dynamic_collision_detection(dt, data)
    }

    fn static_collision_detection(&mut self, data: &realm::RealmData) -> bool {
        for vertex in self.model_vertex.iter() {
            if Self::test_aabb_collistion(self.position + (*vertex).to_vec(), data) {
                return true;
            }
        }
        false
    }

    //都是返回是否碰撞
    fn is_grounded(&mut self, data: &realm::RealmData) -> bool {
        for vertex in self.model_vertex.iter() {
            if Self::test_aabb_collistion(self.position + (*vertex).to_vec(), data) {
                return true;
            }
        }
        false
    }

    //返回是否碰撞
    fn dynamic_collision_detection(&mut self, dt: f32, data: &realm::RealmData) {
        // 计算速度和位移
        //有加速度时计算加速度
        if self.acceleration.magnitude() > game_config::ZERO {
            self.velocity = self.velocity + self.acceleration * dt;
        }
        //没有速度时不进行碰撞检测
        if self.velocity.magnitude() < ZERO_VELOCITY {
            self.is_testing = false;
            return;
        }

        self.is_testing = true;

        let mut displacement = self.velocity * dt;

        println!("velocity: {:?}", self.velocity);
        println!("start decting at {:?}", self.position);
        println!("displacement: {:?}", displacement);

        let mut temp_position = self.position;
        let mut collision_coord = Point3::new(0, 0, 0);

        //先尝试X轴移动
        temp_position.x += displacement.x;
        let mut collision_x = false;

        // 检查X轴移动是否碰撞
        for vertex in self.model_vertex.iter() {
            let test_vertex = temp_position + (*vertex).to_vec();
            if Self::test_aabb_collistion(test_vertex, data) {
                collision_x = true;
                collision_coord = test_vertex.cast::<i32>().unwrap();
                break;
            }
        }

        // 如果X轴没有碰撞，更新位置
        if !collision_x {
            self.position.x = temp_position.x;
        } else {
            if self.velocity.x > game_config::ZERO {
                displacement.x = collision_coord.x as f32 - (self.position.x + self.max_x_point);
            } else {
                displacement.x =
                    collision_coord.x as f32 + 1.0 - (self.position.x + self.min_x_point);
            }
            self.position.x += displacement.x;
            temp_position.x = self.position.x;
            self.velocity.x = 0.0;
            println!("x_collision at {:?}", collision_coord);
            println!("displacement.x:{}", displacement.x);
        }

        // 同理处理Y轴和Z轴
        temp_position.y += displacement.y;
        let mut collision_y = false;
        for vertex in self.model_vertex.iter() {
            let test_vertex = temp_position + (*vertex).to_vec();
            if Self::test_aabb_collistion(test_vertex, data) {
                collision_y = true;
                collision_coord = test_vertex.cast::<i32>().unwrap();
                break;
            }
        }

        //需要处理是否着陆
        if !collision_y {
            self.is_grounded = false;
            let mut test_position = temp_position;
            test_position.y -= ZERO_VELOCITY;
            for vertex in self.model_vertex.iter() {
                let test_vertex = test_position + (*vertex).to_vec();
                if Self::test_aabb_collistion(test_vertex, data) {
                    self.is_grounded = true;
                    break;
                }
            }
            println!("grounded_check({})-----", self.is_grounded);
            if !self.is_grounded {
                self.position.y = temp_position.y;
            }
        } else {
            if self.velocity.y > game_config::ZERO {
                displacement.y = collision_coord.y as f32 - (self.position.y + self.max_y_point);
            } else {
                displacement.y =
                    collision_coord.y as f32 + 1.0 - (self.position.y + self.min_y_point);
            }
            self.position.y += displacement.y;
            temp_position.y = self.position.y;
            self.velocity.y = 0.0;
            self.is_grounded = true;
            println!("y_collision at {:?}", collision_coord);
            println!("displacement.y:{}", displacement.y);
        }

        temp_position.z += displacement.z;
        let mut collision_z = false;
        for vertex in self.model_vertex.iter() {
            let test_vertex = temp_position + (*vertex).to_vec();
            if Self::test_aabb_collistion(test_vertex, data) {
                collision_z = true;
                collision_coord = test_vertex.cast::<i32>().unwrap();
                break;
            }
        }

        //最后这里temp_position不用更新了，前面要更新是因为后面要用到temp_position
        if !collision_z {
            self.position.z = temp_position.z;
        } else {
            if self.velocity.z > game_config::ZERO {
                displacement.z = collision_coord.z as f32 - (self.position.z + self.max_z_point);
            } else {
                displacement.z =
                    collision_coord.z as f32 + 1.0 - (self.position.z + self.min_z_point);
            }
            self.position.z += displacement.z;
            self.velocity.z = 0.0;
            println!("z_collision at {:?}", collision_coord);
            println!("displacement.z:{}", displacement.z);
        }
        println!("");
    }

    //返回是否碰撞
    fn test_aabb_collistion(position: Point3<f32>, data: &realm::RealmData) -> bool {
        if data.get_block_f32(position).tp.is_solid() {
            println!("collision at {:?}", position);
            return true;
        }
        false
    }
}
