use std::vec;

use cgmath::*;
use wgpu::util::DeviceExt;
use wgpu::*;

use crate::basic_config;
use crate::camera::*;
use crate::control;
use crate::game_config;
use crate::realm;
use crate::realm::RealmData;

pub const JUMP_SPEED: f32 = 10.0;
pub const GRAVITATIONAL_ACCELERATION: f32 = 9.8;
pub const DELTA_DISPLACEMENT: f32 = 1e-6;
pub const ZERO_VELOCITY: f32 = 1e-3;

//玩家实体的尺寸
pub const PLAYER_SIZE: Vector3<f32> = Vector3::new(0.5, 1.8, 0.5);

//最大速度
pub const TERMINAL_VELOCITY: f32 = 100.0;

pub enum EntityType {
    Player,
}

pub struct Entity {
    //原点为坐标原点，三个大小按3个坐标轴正向延伸
    pub position: Point3<f32>,
    pub velocity: Vector3<f32>,
    pub entity_type: EntityType,
    pub is_grounded: bool,
    pub size: Vector3<f32>,
    pub is_testing: bool,
}

pub struct CollisionResult {
    // 是否发生碰撞
    pub collided: bool,
    // 碰撞的方向（用单位向量表示）
    pub normal: Vector3<f32>,
    // 碰撞的体素位置
    pub block_position: Option<Point3<i32>>,
    // 碰撞的穿透深度
    pub penetration: f32,
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
    pub is_move_speed_set: bool,

    //记录上一帧的移动速度，在下一帧减去，以更新视角移动时的速度矢量
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
        position.y = realm.get_first_none_empty_block(1.0, 1.0) as f32 + 5.0;
        let camera_position = position + Vector3::new(0.0, PLAYER_SIZE.y, 0.0);
        let camera = Camera::new(camera_position, cgmath::Deg(90.0), cgmath::Deg(-45.0));
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

        Self {
            entity: Entity::new(position, PLAYER_SIZE, EntityType::Player),
            camera,
            projection,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,
            camera_controller,
            is_move_speed_set: false,
            move_velocity: Vector3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn update_camera(&mut self, data: &mut realm::RealmData) {
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
        // 保留Y方向速度（重力）
        let y_velocity = self.entity.velocity.y;

        // 计算新的移动速度
        let mut move_velocity: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
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

        // 直接设置水平速度（不累积）
        self.entity.velocity = Vector3::new(self.move_velocity.x, y_velocity, self.move_velocity.z);

        self.entity.update(dt, data);
        self.camera.position = self.entity.position;
        self.camera.position.y += PLAYER_SIZE.y;
        self.update_camera(data);
        self.camera_uniform
            .update_view_proj(&self.camera, &self.projection);
    }
}

impl Entity {
    pub fn new(position: Point3<f32>, size: Vector3<f32>, entity_type: EntityType) -> Self {
        let is_testing = false;

        let is_grounded = false;
        Self {
            position,
            velocity: Vector3::new(0.0, 0.0, 0.0),
            entity_type,
            size,
            is_grounded,
            is_testing,
        }
    }

    fn apply_gravity(&mut self, dt: f32) {
        if !self.is_grounded {
            self.velocity.y -= GRAVITATIONAL_ACCELERATION * dt;

            if self.velocity.y < -TERMINAL_VELOCITY {
                self.velocity.y = -TERMINAL_VELOCITY;
            }
        }
    }

    /// 检查与某个体素是否发生碰撞
    fn check_block_collision(
        &self,
        block_pos: Point3<i32>,
        data: &RealmData,
    ) -> Option<CollisionResult> {
        // 获取方块类型
        let block = data.get_block(block_pos);

        // 如果方块不是实心的，则没有碰撞
        if !block.tp.is_solid() {
            return None;
        }

        // 方块的AABB表示
        let block_min = Point3::new(block_pos.x as f32, block_pos.y as f32, block_pos.z as f32);
        let block_max = Point3::new(
            block_pos.x as f32 + 1.0,
            block_pos.y as f32 + 1.0,
            block_pos.z as f32 + 1.0,
        );

        // 计算玩家AABB和方块AABB的重叠情况
        let overlap_min = Vector3::new(
            (self.position.x - block_max.x).max(block_min.x - (self.position.x + self.size.x)),
            (self.position.y - block_max.y).max(block_min.y - (self.position.y + self.size.y)),
            (self.position.z - block_max.z).max(block_min.z - (self.position.z + self.size.z)),
        );

        // 如果任一轴上没有重叠，则没有碰撞
        if overlap_min.x > 0.0 || overlap_min.y > 0.0 || overlap_min.z > 0.0 {
            return None;
        }

        // 计算穿透深度和碰撞法线
        let (penetration, axis) = self.find_min_penetration(block_min, block_max);

        Some(CollisionResult {
            collided: true,
            normal: axis,
            block_position: Some(block_pos),
            penetration,
        })
    }

    /// 找出最小穿透深度和对应的轴
    fn find_min_penetration(
        &self,
        block_min: Point3<f32>,
        block_max: Point3<f32>,
    ) -> (f32, Vector3<f32>) {
        // 计算各个轴上的穿透深度
        let min_x =
            (block_max.x - self.position.x).min((self.position.x + self.size.x) - block_min.x);
        let min_y =
            (block_max.y - self.position.y).min((self.position.y + self.size.y) - block_min.y);
        let min_z =
            (block_max.z - self.position.z).min((self.position.z + self.size.z) - block_min.z);

        // 确定穿透最小的轴
        if min_x <= min_y && min_x <= min_z {
            // X轴穿透最小
            let normal = if self.position.x < (block_min.x + block_max.x) / 2.0 {
                Vector3::new(-1.0, 0.0, 0.0)
            } else {
                Vector3::new(1.0, 0.0, 0.0)
            };
            (min_x, normal)
        } else if min_y <= min_x && min_y <= min_z {
            // Y轴穿透最小
            let center_y = self.position.y + self.size.y / 2.0;
            let normal = if center_y < (block_min.y + block_max.y) / 2.0 {
                Vector3::new(0.0, -1.0, 0.0)
            } else {
                Vector3::new(0.0, 1.0, 0.0)
            };
            (min_y, normal)
        } else {
            // Z轴穿透最小
            let normal = if self.position.z < (block_min.z + block_max.z) / 2.0 {
                Vector3::new(0.0, 0.0, -1.0)
            } else {
                Vector3::new(0.0, 0.0, 1.0)
            };
            (min_z, normal)
        }
    }

    /// 检测所有可能发生碰撞的体素
    fn check_surrounding_blocks(&self, data: &RealmData) -> Vec<CollisionResult> {
        let mut collisions = Vec::new();

        // 确定需要检查的区域
        let min_block = Vector3::new(
            (self.position.x - 1.0).floor() as i32,
            (self.position.y - 1.0).floor() as i32,
            (self.position.z - 1.0).floor() as i32,
        );

        let max_block = Vector3::new(
            (self.position.x + self.size.x + 1.0).ceil() as i32,
            (self.position.y + self.size.y + 1.0).ceil() as i32,
            (self.position.z + self.size.z + 1.0).ceil() as i32,
        );

        // 检查所有可能碰撞的方块
        for x in min_block.x..max_block.x {
            for y in min_block.y..max_block.y {
                for z in min_block.z..max_block.z {
                    let block_pos = Point3::new(x, y, z);
                    if let Some(collision) = self.check_block_collision(block_pos, data) {
                        collisions.push(collision);
                    }
                }
            }
        }

        collisions
    }

    /// 解决碰撞，并更新角色位置
    fn resolve_collision(&mut self, collision: &CollisionResult) {
        // 调整位置以解决穿透
        let correction = collision.normal * collision.penetration;
        self.position = self.position + correction;

        // 计算速度在碰撞法线方向上的分量
        let velocity_dot_normal = self.velocity.dot(collision.normal);

        // 如果物体正在向碰撞平面移动
        if velocity_dot_normal < 0.0 {
            // 使用投影保留平行于碰撞面的速度分量（滑动效果）
            // 计算平行于碰撞面的速度
            let parallel_velocity = self.velocity - collision.normal * velocity_dot_normal;

            // 设置速度为平行分量（保留滑动，消除穿透）
            self.velocity = parallel_velocity;
        }

        // 检查是否接触地面
        if collision.normal.y > 0.7 {
            // 如果法线主要朝上
            self.is_grounded = true;
        }
    }

    /// 更新物理并处理碰撞
    pub fn update(&mut self, dt: f32, data: &RealmData) {
        // 保存原始位置，用于回退
        let original_position = self.position;

        // 应用物理效果（如重力）
        self.apply_gravity(dt);

        // 假设我们会失去地面接触
        self.is_grounded = false;

        // 执行运动积分（使用半隐式欧拉方法）
        self.position = self.position + self.velocity * dt;

        // 检测和解决碰撞
        let collisions = self.check_surrounding_blocks(data);

        // 如果没有碰撞，可以直接返回
        if collisions.is_empty() {
            // 检查是否站在地面上
            let ground_test_position = Point3::new(
                self.position.x,
                self.position.y - 0.1, // 向下偏移一点点检测地面
                self.position.z,
            );

            // 检查四个角落是否有地面
            let corners = [
                Point3::new(
                    ground_test_position.x,
                    ground_test_position.y,
                    ground_test_position.z,
                ),
                Point3::new(
                    ground_test_position.x + self.size.x,
                    ground_test_position.y,
                    ground_test_position.z,
                ),
                Point3::new(
                    ground_test_position.x,
                    ground_test_position.y,
                    ground_test_position.z + self.size.z,
                ),
                Point3::new(
                    ground_test_position.x + self.size.x,
                    ground_test_position.y,
                    ground_test_position.z + self.size.z,
                ),
            ];

            for corner in &corners {
                let block_pos = Point3::new(
                    corner.x.floor() as i32,
                    corner.y.floor() as i32,
                    corner.z.floor() as i32,
                );

                if data.get_block(block_pos).tp.is_solid() {
                    self.is_grounded = true;
                    break;
                }
            }

            return;
        }

        // 回退到原始位置
        self.position = original_position;

        // 分离轴处理 - 仅在每个轴上移动小增量并检测碰撞

        // 1. X轴移动
        self.position.x += self.velocity.x * dt;

        let x_collisions = self.check_surrounding_blocks(data);
        for collision in &x_collisions {
            self.resolve_collision(collision);
        }

        // 2. Y轴移动
        self.position.y += self.velocity.y * dt;

        let y_collisions = self.check_surrounding_blocks(data);
        for collision in &y_collisions {
            self.resolve_collision(collision);
        }

        // 3. Z轴移动
        self.position.z += self.velocity.z * dt;

        let z_collisions = self.check_surrounding_blocks(data);
        for collision in &z_collisions {
            self.resolve_collision(collision);
        }

        // 如果速度非常小，则认为静止
        if self.velocity.magnitude() < game_config::ZERO {
            self.velocity = Vector3::zero();
        }
    }
}
