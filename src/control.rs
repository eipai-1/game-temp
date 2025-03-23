use cgmath::*;
use winit::{
    event::*,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::camera::*;
use crate::game_config;
use crate::physics::{PlayerEntity, JUMP_SPEED};
use crate::realm;

pub struct Control {
    pub forward: Vector3<f32>,
    pub right: Vector3<f32>,
    //是否转动了视角
    pub is_rotating_view: bool,
}

impl Control {
    pub fn new(camera: &Camera) -> Self {
        Self {
            forward: camera.forward,
            right: camera.right,
            is_rotating_view: false,
        }
    }

    pub fn event_walk_forward(&mut self, is_pressed: bool, player: &mut PlayerEntity) -> bool {
        player.camera_controller.is_forward_pressed = is_pressed;
        true
    }

    pub fn event_walk_left(&mut self, is_pressed: bool, player: &mut PlayerEntity) -> bool {
        player.camera_controller.is_left_pressed = is_pressed;
        true
    }

    pub fn event_walk_backward(&mut self, is_pressed: bool, player: &mut PlayerEntity) -> bool {
        player.camera_controller.is_backward_pressed = is_pressed;
        true
    }

    pub fn event_walk_right(&mut self, is_pressed: bool, player: &mut PlayerEntity) -> bool {
        player.camera_controller.is_right_pressed = is_pressed;
        true
    }

    pub fn event_jump(&mut self, is_pressed: bool, player: &mut PlayerEntity) -> bool {
        if is_pressed {
            player.entity.velocity.y += JUMP_SPEED;
        }
        true
    }

    pub fn process_events(
        &mut self,
        player: &mut PlayerEntity,
        event: &WindowEvent,
        realm: &mut realm::Realm,
        queue: &wgpu::Queue,
        game_config: &mut game_config::GameConfig,
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
                    KeyCode::KeyW => self.event_walk_forward(is_pressed, player),
                    KeyCode::KeyA => self.event_walk_left(is_pressed, player),
                    KeyCode::KeyS => self.event_walk_backward(is_pressed, player),
                    KeyCode::KeyD => self.event_walk_right(is_pressed, player),
                    KeyCode::ShiftLeft => false,
                    KeyCode::Space => self.event_jump(is_pressed, player),
                    KeyCode::KeyE => {
                        if is_pressed {
                            player.camera_controller.is_fov = !player.camera_controller.is_fov;
                        }
                        true
                    }
                    KeyCode::F5 => {
                        if is_pressed {
                            game_config.is_debug_window_open = !game_config.is_debug_window_open;
                        }
                        true
                    }
                    _ => false,
                }
            }
            WindowEvent::Focused(true) => {
                player.camera_controller.is_fov = true;
                true
            }
            WindowEvent::Focused(false) => {
                player.camera_controller.is_fov = false;
                true
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                //println!("left mouse button pressed");
                if player.camera_controller.is_fov {
                    if let Some(selected_block) = player.camera_controller.selected_block {
                        realm.place_block(selected_block, realm::BLOCK_EMPTY, queue);
                        return true;
                    }
                }
                false
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Right,
                ..
            } => {
                if player.camera_controller.is_fov {
                    if let Some(pre_selected_block) = player.camera_controller.pre_selected_block {
                        realm.place_block(
                            pre_selected_block,
                            realm::Block {
                                tp: realm::BlockType::Stone,
                            },
                            queue,
                        );
                        return true;
                    }
                }
                false
            }
            WindowEvent::CursorMoved { position, .. } if player.camera_controller.is_fov => {
                player.camera_controller.dx =
                    (player.camera_controller.center_x as f64 - position.x) as f32;
                player.camera_controller.dy =
                    (player.camera_controller.center_y as f64 - position.y) as f32;
                player.camera.yaw -=
                    Rad(player.camera_controller.dx) * player.camera_controller.fov_sensitivity;
                player.camera.pitch +=
                    Rad(player.camera_controller.dy) * player.camera_controller.fov_sensitivity;

                if player.camera.pitch < -Rad(SAFE_FRAC_PI_2) {
                    player.camera.pitch = -Rad(SAFE_FRAC_PI_2);
                } else if player.camera.pitch > Rad(SAFE_FRAC_PI_2) {
                    player.camera.pitch = Rad(SAFE_FRAC_PI_2);
                }
                player.camera.forward = player.camera.direction();
                player.camera.right = player.camera.forward.cross(Vector3::unit_y());
                player.is_move_speed_set = false;
                true
            }
            _ => false,
        }
    }
}
