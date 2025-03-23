use cgmath::*;

use crate::{
    camera::Camera,
    realm::RealmData,
    voxel_collision::{CollisionBody, GRAVITY, TERMINAL_VELOCITY},
};

pub struct Player {
    // 碰撞体
    pub collision_body: CollisionBody,
    // 玩家摄像机
    pub camera: Camera,
    // 移动速度
    pub move_speed: f32,
    // 跳跃力度
    pub jump_force: f32,
    // 是否正在跳跃
    pub is_jumping: bool,
}

impl Player {
    pub fn new(position: Point3<f32>, move_speed: f32, jump_force: f32) -> Self {
        // 创建碰撞体，设置半径为 (0.4, 0.9, 0.4)，表示玩家宽0.8格，高1.8格
        let collision_body = CollisionBody::new(position, Vector3::new(0.4, 0.9, 0.4));

        // 创建摄像机，眼睛位置比碰撞体中心点高一些
        let eye_position = position + Vector3::new(0.0, 0.5, 0.0);
        let camera = Camera::new(eye_position, cgmath::Deg(90.0), cgmath::Deg(-20.0));

        Self {
            collision_body,
            camera,
            move_speed,
            jump_force,
            is_jumping: false,
        }
    }

    // 处理玩家移动输入
    pub fn handle_movement(&mut self, forward: bool, backward: bool, left: bool, right: bool) {
        // 重置速度
        let mut movement = Vector3::new(0.0, 0.0, 0.0);

        // 基于摄像机朝向计算移动方向
        if forward {
            let forward_dir =
                Vector3::new(self.camera.forward.x, 0.0, self.camera.forward.z).normalize();
            movement += forward_dir * self.move_speed;
        }
        if backward {
            let backward_dir =
                Vector3::new(-self.camera.forward.x, 0.0, -self.camera.forward.z).normalize();
            movement += backward_dir * self.move_speed;
        }
        if left {
            let left_dir =
                Vector3::new(-self.camera.right.x, 0.0, -self.camera.right.z).normalize();
            movement += left_dir * self.move_speed;
        }
        if right {
            let right_dir = Vector3::new(self.camera.right.x, 0.0, self.camera.right.z).normalize();
            movement += right_dir * self.move_speed;
        }

        // 应用到碰撞体上，保留Y轴速度（不影响跳跃和重力）
        let y_velocity = self.collision_body.velocity.y;
        self.collision_body.velocity = Vector3::new(movement.x, y_velocity, movement.z);
    }

    // 处理跳跃
    pub fn jump(&mut self) {
        // 只有在地面上才能跳跃
        if self.collision_body.on_ground && !self.is_jumping {
            self.collision_body.velocity.y = self.jump_force;
            self.is_jumping = true;
        }
    }

    // 更新玩家状态
    pub fn update(&mut self, dt: f32, data: &RealmData) {
        // 更新碰撞体（应用重力和碰撞检测）
        self.collision_body.update(dt, data);

        // 如果着陆，重置跳跃状态
        if self.collision_body.on_ground {
            self.is_jumping = false;
        }

        // 更新摄像机位置（跟随碰撞体，并保持适当高度）
        self.camera.position = self.collision_body.position + Vector3::new(0.0, 0.5, 0.0);
    }

    // 选择方块（光线投射）
    pub fn select_block(
        &self,
        max_distance: f32,
        data: &RealmData,
    ) -> Option<(Point3<i32>, Point3<i32>)> {
        self.collision_body
            .raycast(self.camera.direction(), max_distance, data)
    }
}

// 游戏循环中使用示例
pub fn game_loop_example(
    player: &mut Player,
    realm_data: &RealmData,
    dt: f32,
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    jump: bool,
) {
    // 处理输入
    player.handle_movement(forward, backward, left, right);

    if jump {
        player.jump();
    }

    // 更新玩家状态（包括碰撞检测）
    player.update(dt, realm_data);

    // 检测玩家是否选中了方块
    if let Some((block_pos, pre_block_pos)) = player.select_block(5.0, realm_data) {
        // 这里可以处理选中方块的逻辑，如高亮显示、破坏或放置方块等
        println!(
            "选中方块位置: {:?}, 前一个位置: {:?}",
            block_pos, pre_block_pos
        );
    }
}

// 以下是一个简化的游戏初始化示例
pub fn init_game_example() -> (Player, RealmData) {
    // 这里应该加载或生成游戏世界数据
    let realm_data = RealmData::new(); // 这里假设RealmData::new()会生成一个默认的世界

    // 创建玩家，起始位置在(10.0, 20.0, 10.0)，移动速度为5.0，跳跃力度为10.0
    let player = Player::new(Point3::new(10.0, 20.0, 10.0), 5.0, 10.0);

    (player, realm_data)
}
