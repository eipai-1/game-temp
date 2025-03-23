use cgmath::*;
use std::time::Duration;

use crate::realm::{Block, BlockType, RealmData};

// 物理常量
pub const GRAVITY: f32 = 9.8;
pub const TERMINAL_VELOCITY: f32 = 20.0;
pub const EPSILON: f32 = 1e-4;

/// 碰撞体表示玩家或实体的碰撞外壳
pub struct CollisionBody {
    // 碰撞体的中心点
    pub position: Point3<f32>,
    // 速度向量
    pub velocity: Vector3<f32>,
    // 碰撞体的大小（宽度/2, 高度/2, 深度/2）
    pub half_size: Vector3<f32>,
    // 是否在地面上
    pub on_ground: bool,
}

/// 碰撞检测的结果
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

impl CollisionBody {
    pub fn new(position: Point3<f32>, half_size: Vector3<f32>) -> Self {
        Self {
            position,
            velocity: Vector3::new(0.0, 0.0, 0.0),
            half_size,
            on_ground: false,
        }
    }

    /// 获取碰撞体的最小顶点（左下后）
    pub fn min_point(&self) -> Point3<f32> {
        Point3::new(
            self.position.x - self.half_size.x,
            self.position.y - self.half_size.y,
            self.position.z - self.half_size.z,
        )
    }

    /// 获取碰撞体的最大顶点（右上前）
    pub fn max_point(&self) -> Point3<f32> {
        Point3::new(
            self.position.x + self.half_size.x,
            self.position.y + self.half_size.y,
            self.position.z + self.half_size.z,
        )
    }

    /// 应用重力和其他物理效果
    pub fn apply_physics(&mut self, dt: f32) {
        if !self.on_ground {
            // 应用重力
            self.velocity.y -= GRAVITY * dt;

            // 限制最大下落速度
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
            (self.min_point().x - block_max.x).max(block_min.x - self.max_point().x),
            (self.min_point().y - block_max.y).max(block_min.y - self.max_point().y),
            (self.min_point().z - block_max.z).max(block_min.z - self.max_point().z),
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
        let min_x = (block_max.x - self.min_point().x).min(self.max_point().x - block_min.x);
        let min_y = (block_max.y - self.min_point().y).min(self.max_point().y - block_min.y);
        let min_z = (block_max.z - self.min_point().z).min(self.max_point().z - block_min.z);

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
            let normal = if self.position.y < (block_min.y + block_max.y) / 2.0 {
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
            (self.min_point().x - 1.0).floor() as i32,
            (self.min_point().y - 1.0).floor() as i32,
            (self.min_point().z - 1.0).floor() as i32,
        );

        let max_block = Vector3::new(
            (self.max_point().x + 1.0).ceil() as i32,
            (self.max_point().y + 1.0).ceil() as i32,
            (self.max_point().z + 1.0).ceil() as i32,
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
            // 反弹速度 (可以添加弹性系数)
            let elasticity = 0.0; // 0表示完全没有反弹
            let impulse = collision.normal * (-velocity_dot_normal * (1.0 + elasticity));
            self.velocity = self.velocity + impulse;
        }

        // 检查是否接触地面
        if collision.normal.y > 0.7 {
            // 如果法线主要朝上
            self.on_ground = true;
        }
    }

    /// 更新物理并处理碰撞
    pub fn update(&mut self, dt: f32, data: &RealmData) {
        // 保存原始位置，用于回退
        let original_position = self.position;

        // 应用物理效果（如重力）
        self.apply_physics(dt);

        // 假设我们会失去地面接触
        self.on_ground = false;

        // 执行运动积分（使用半隐式欧拉方法）
        self.position = self.position + self.velocity * dt;

        // 检测和解决碰撞
        let collisions = self.check_surrounding_blocks(data);

        // 如果没有碰撞，可以直接返回
        if collisions.is_empty() {
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
        if self.velocity.magnitude() < EPSILON {
            self.velocity = Vector3::zero();
        }
    }

    /// 光线投射检测，获取玩家视线前方的方块
    pub fn raycast(
        &self,
        direction: Vector3<f32>,
        max_distance: f32,
        data: &RealmData,
    ) -> Option<(Point3<i32>, Point3<i32>)> {
        // 标准化方向向量
        let ray_dir = direction.normalize();

        // 起点（玩家视线位置）
        let ray_start = self.position;

        // 使用数字微分分析算法(DDA)进行光线追踪
        let mut map_pos = Point3::new(
            ray_start.x.floor() as i32,
            ray_start.y.floor() as i32,
            ray_start.z.floor() as i32,
        );

        // 计算沿着每个轴方向到下一个网格边界的距离
        let delta_dist = Vector3::new(
            if ray_dir.x.abs() < EPSILON {
                f32::INFINITY
            } else {
                1.0 / ray_dir.x.abs()
            },
            if ray_dir.y.abs() < EPSILON {
                f32::INFINITY
            } else {
                1.0 / ray_dir.y.abs()
            },
            if ray_dir.z.abs() < EPSILON {
                f32::INFINITY
            } else {
                1.0 / ray_dir.z.abs()
            },
        );

        // 决定步进方向和初始边界距离
        let step = Vector3::new(
            if ray_dir.x < 0.0 { -1 } else { 1 },
            if ray_dir.y < 0.0 { -1 } else { 1 },
            if ray_dir.z < 0.0 { -1 } else { 1 },
        );

        let mut side_dist = Vector3::new(
            if ray_dir.x < 0.0 {
                (ray_start.x - map_pos.x as f32) * delta_dist.x
            } else {
                (map_pos.x as f32 + 1.0 - ray_start.x) * delta_dist.x
            },
            if ray_dir.y < 0.0 {
                (ray_start.y - map_pos.y as f32) * delta_dist.y
            } else {
                (map_pos.y as f32 + 1.0 - ray_start.y) * delta_dist.y
            },
            if ray_dir.z < 0.0 {
                (ray_start.z - map_pos.z as f32) * delta_dist.z
            } else {
                (map_pos.z as f32 + 1.0 - ray_start.z) * delta_dist.z
            },
        );

        // 进行DDA算法
        let mut distance = 0.0;
        let mut previous_pos = map_pos;

        while distance < max_distance {
            // 记录上一个位置
            previous_pos = map_pos;

            // 沿着最近的网格边界前进
            if side_dist.x < side_dist.y && side_dist.x < side_dist.z {
                side_dist.x += delta_dist.x;
                map_pos.x += step.x;
                distance = side_dist.x;
            } else if side_dist.y < side_dist.z {
                side_dist.y += delta_dist.y;
                map_pos.y += step.y;
                distance = side_dist.y;
            } else {
                side_dist.z += delta_dist.z;
                map_pos.z += step.z;
                distance = side_dist.z;
            }

            // 检查当前位置是否为实心方块
            let block = data.get_block(map_pos);
            if block.tp.is_solid() {
                return Some((map_pos, previous_pos));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 这里可以添加测试用例
    #[test]
    fn test_collision_detection() {
        // 测试代码将在实际项目中实现
    }
}
