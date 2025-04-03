use crate::entity::Player;
use glyphon::{Color, TextArea, TextBounds};
use std::vec;
use wgpu::core::device::queue;
use wgpu::naga::Block;
use wgpu::{util::DeviceExt, *};
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::{item, realm};
use inventory_renderer::{InventoryRenderer, SLOTS_PER_COLUMN};

mod block_renderer;
pub mod inventory_renderer;
pub mod ui_text_renderer;

const CURSOR_HALF_SIZE: f32 = 12.0;
const CURSOR_HALF_LENGTH: f32 = 2.0;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct UIVertex {
    position: [f32; 2],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct UIUniform {
    screen_size: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct UIInstance {
    position: [f32; 2],
    _padding: [f32; 2],
}

impl UIInstance {
    fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<UIInstance>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[VertexAttribute {
                format: VertexFormat::Float32x2,
                offset: 0,
                shader_location: 5,
            }],
        }
    }
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: [x, y],
            _padding: [0.0, 0.0],
        }
    }
}

const CURSOR_INDEX: &[u16] = &[0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7];

impl UIVertex {
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<UIVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 2]>() as BufferAddress,
                    shader_location: 1,
                },
            ],
        }
    }
}

pub struct UI {
    pub is_invenory_open: bool,
    pub is_hotbar_open: bool,
    pub is_debug_info_open: bool,

    cursor_vertex_buffer: Buffer,
    cursor_index_buffer: Buffer,
    cursor_vertices: Vec<UIVertex>,
    pub ui_text_renderer: ui_text_renderer::UITextRenderer,
    screen_size_uniform_buffer: Buffer,
    screen_size_uniform_bind_group: BindGroup,
    instance_buffer: Buffer,
    instances: Vec<UIInstance>,
    inventory_renderer: InventoryRenderer,
    block_renderer: block_renderer::BlockRenderer,

    cursor_position: cgmath::Point2<f32>,
}

impl UI {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        scale_factor: f32,
        physical_size: winit::dpi::PhysicalSize<u32>,
        player: &Player,
        block_materials_bind_group_layout: &BindGroupLayout,
        texture_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let mut ui_text_renderer = ui_text_renderer::UITextRenderer::new(
            device,
            queue,
            format,
            scale_factor,
            physical_size,
        );

        ui_text_renderer.generate_debug_info();

        let instances = vec![UIInstance {
            position: [0.0, 0.0],
            _padding: [0.0, 0.0],
        }];

        let instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: BufferUsages::VERTEX,
        });

        let screen_size_uniform_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("UI Uniform Bind Group Layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let screen_size_uniform_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("UI Uniform Buffer"),
            contents: bytemuck::cast_slice(&[UIUniform {
                screen_size: [physical_size.width as f32, physical_size.height as f32],
            }]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let screen_size_uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("UI Uniform Bind Group"),
            layout: &screen_size_uniform_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: screen_size_uniform_buffer.as_entire_binding(),
            }],
        });

        let cursor_vertices = Self::generate_cursor_vertices(physical_size);

        let cursor_vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Cursor Vertex Buffer"),
            contents: bytemuck::cast_slice(&cursor_vertices),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        let cursor_index_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Cursor Index Buffer"),
            contents: bytemuck::cast_slice(&CURSOR_INDEX),
            usage: BufferUsages::INDEX,
        });

        let block_renderer = block_renderer::BlockRenderer::new(
            device,
            format,
            player,
            physical_size,
            &screen_size_uniform_bind_group_layout,
            block_materials_bind_group_layout,
            texture_bind_group_layout,
        );

        let inventory_renderer = InventoryRenderer::new(
            device,
            format,
            &screen_size_uniform_bind_group_layout,
            physical_size,
            block_renderer,
        );

        let block_renderer = block_renderer::BlockRenderer::new(
            device,
            format,
            player,
            physical_size,
            &screen_size_uniform_bind_group_layout,
            block_materials_bind_group_layout,
            texture_bind_group_layout,
        );

        let is_hotbar_open = true;
        let is_invenory_open = false;
        let is_debug_info_open = false;

        Self {
            cursor_vertex_buffer,
            cursor_vertices,
            ui_text_renderer,
            cursor_index_buffer,
            screen_size_uniform_buffer,
            screen_size_uniform_bind_group,
            inventory_renderer,
            instance_buffer,
            instances,
            block_renderer,
            is_hotbar_open,
            is_invenory_open,
            is_debug_info_open,
            cursor_position: cgmath::Point2::new(0.0, 0.0),
        }
    }

    fn generate_cursor_vertices(physical_size: winit::dpi::PhysicalSize<u32>) -> Vec<UIVertex> {
        let mut cursor_vertices: Vec<UIVertex> = Vec::new();
        //竖下来的指针
        //顺时针四个点 从左上角开始
        cursor_vertices.push(UIVertex {
            position: [
                physical_size.width as f32 / 2.0 - CURSOR_HALF_SIZE,
                physical_size.height as f32 / 2.0 - CURSOR_HALF_LENGTH,
            ],
            color: [1.0, 1.0, 1.0, 0.7],
        });
        cursor_vertices.push(UIVertex {
            position: [
                physical_size.width as f32 / 2.0 + CURSOR_HALF_SIZE,
                physical_size.height as f32 / 2.0 - CURSOR_HALF_LENGTH,
            ],
            color: [1.0, 1.0, 1.0, 0.7],
        });
        cursor_vertices.push(UIVertex {
            position: [
                physical_size.width as f32 / 2.0 + CURSOR_HALF_SIZE,
                physical_size.height as f32 / 2.0 + CURSOR_HALF_LENGTH,
            ],
            color: [1.0, 1.0, 1.0, 0.7],
        });
        cursor_vertices.push(UIVertex {
            position: [
                physical_size.width as f32 / 2.0 - CURSOR_HALF_SIZE,
                physical_size.height as f32 / 2.0 + CURSOR_HALF_LENGTH,
            ],
            color: [1.0, 1.0, 1.0, 0.7],
        });
        //横着的指针
        cursor_vertices.push(UIVertex {
            position: [
                physical_size.width as f32 / 2.0 - CURSOR_HALF_LENGTH,
                physical_size.height as f32 / 2.0 - CURSOR_HALF_SIZE,
            ],
            color: [1.0, 1.0, 1.0, 0.7],
        });
        cursor_vertices.push(UIVertex {
            position: [
                physical_size.width as f32 / 2.0 + CURSOR_HALF_LENGTH,
                physical_size.height as f32 / 2.0 - CURSOR_HALF_SIZE,
            ],
            color: [1.0, 1.0, 1.0, 0.7],
        });
        cursor_vertices.push(UIVertex {
            position: [
                physical_size.width as f32 / 2.0 + CURSOR_HALF_LENGTH,
                physical_size.height as f32 / 2.0 + CURSOR_HALF_SIZE,
            ],
            color: [1.0, 1.0, 1.0, 0.7],
        });
        cursor_vertices.push(UIVertex {
            position: [
                physical_size.width as f32 / 2.0 - CURSOR_HALF_LENGTH,
                physical_size.height as f32 / 2.0 + CURSOR_HALF_SIZE,
            ],
            color: [1.0, 1.0, 1.0, 0.7],
        });
        cursor_vertices
    }

    pub fn draw_text(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        left: f32,
        top: f32,
        encoder: &mut CommandEncoder,
        view: &wgpu::TextureView,
        render_pass: &mut RenderPass,
    ) {
        self.ui_text_renderer
            .draw_text(device, queue, render_pass, self.is_debug_info_open);
    }

    pub fn resize(
        &mut self,
        queue: &Queue,
        player: &Player,
        physical_size: winit::dpi::PhysicalSize<u32>,
    ) {
        let ui_uniform = UIUniform {
            screen_size: [physical_size.width as f32, physical_size.height as f32],
        };

        self.cursor_vertices = Self::generate_cursor_vertices(physical_size);
        queue.write_buffer(
            &self.cursor_vertex_buffer,
            0,
            bytemuck::cast_slice(&self.cursor_vertices),
        );

        queue.write_buffer(
            &self.screen_size_uniform_buffer,
            0,
            bytemuck::cast_slice(&[ui_uniform]),
        );

        self.inventory_renderer.resize(queue, physical_size);
        self.inventory_renderer
            .block_renderer
            .resize(queue, player, physical_size);
    }

    pub fn draw_ui(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        diffuse_bind_group: &BindGroup,
        block_materials_bind_group: &BindGroup,
        left: f32,
        top: f32,
    ) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("UI Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.inventory_renderer.render_pipeline);

        self.draw_cursor(&mut render_pass);

        if self.is_invenory_open {
            self.inventory_renderer.draw_inventory(&mut render_pass);
            self.inventory_renderer.block_renderer.draw_all_item(
                &mut render_pass,
                &self.screen_size_uniform_bind_group,
                diffuse_bind_group,
                block_materials_bind_group,
            );
            self.inventory_renderer.block_renderer.draw_iv_hb(
                &mut render_pass,
                &self.screen_size_uniform_bind_group,
                diffuse_bind_group,
                block_materials_bind_group,
            );
            if self.inventory_renderer.is_dragging {
                self.inventory_renderer.block_renderer.draw_dragging(
                    &mut render_pass,
                    &self.screen_size_uniform_bind_group,
                    diffuse_bind_group,
                    block_materials_bind_group,
                    self.inventory_renderer
                        .dragging_instance_buffer
                        .as_ref()
                        .unwrap(),
                );
            }
        }
        if self.is_hotbar_open {
            self.inventory_renderer
                .draw_hotbar(&self.screen_size_uniform_bind_group, &mut render_pass);
            self.inventory_renderer.block_renderer.draw_hb(
                &mut render_pass,
                &self.screen_size_uniform_bind_group,
                diffuse_bind_group,
                block_materials_bind_group,
            );
        }

        self.ui_text_renderer
            .draw_text(device, queue, &mut render_pass, self.is_debug_info_open);
    }

    pub fn update_ui(&mut self, position: cgmath::Point3<f32>, dt: f64, realm: &realm::Realm) {
        self.ui_text_renderer.update_debug_info(position, dt, realm);
    }

    fn draw_cursor(&self, render_pass: &mut RenderPass) {
        render_pass.set_bind_group(0, &self.screen_size_uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.cursor_vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.cursor_index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(0..CURSOR_INDEX.len() as _, 0, 0..1);
    }

    pub fn process_events(
        &mut self,
        event: &winit::event::WindowEvent,
        is_fov: bool,
        queue: &Queue,
        player: &mut Player,
        physical_size: winit::dpi::PhysicalSize<u32>,
        device: &wgpu::Device,
        all_block: &Vec<realm::BlockInfo>,
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
                let is_pressed = *state == winit::event::ElementState::Pressed;
                match keycode {
                    KeyCode::KeyE => {
                        if is_fov {
                            self.is_invenory_open = false;
                            self.is_hotbar_open = true;
                        } else {
                            self.is_invenory_open = true;
                            self.is_hotbar_open = false;
                        }
                        return true;
                    }
                    KeyCode::Escape => {
                        if self.is_invenory_open {
                            self.is_invenory_open = false;
                            self.is_hotbar_open = true;
                        }
                        return true;
                    }
                    KeyCode::F3 => {
                        if is_pressed {
                            self.is_debug_info_open = !self.is_debug_info_open;
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if *state == winit::event::ElementState::Pressed {
                    match button {
                        MouseButton::Left => {
                            if self.is_invenory_open {
                                let (x, y) = inventory_renderer::get_selected_slot(
                                    self.cursor_position.x,
                                    self.cursor_position.y,
                                    physical_size,
                                );
                                //self.ui_text_renderer
                                //    .set_text(format!("{},{}", x, y).as_str());

                                if y >= inventory_renderer::SLOTS_PER_COLUMN {
                                    return true;
                                }
                                let tp = player.all_item_inventory[y as usize][x as usize]
                                    .item_type
                                    .get_type();
                                if tp != realm::BlockType::Empty as u32 {
                                    self.inventory_renderer.is_dragging = true;
                                    self.inventory_renderer.create_dragging_instance(
                                        tp,
                                        self.cursor_position.x,
                                        self.cursor_position.y,
                                        device,
                                    );
                                }
                            }
                            return true;
                        }
                        _ => {}
                    }
                } else {
                    if self.inventory_renderer.is_dragging {
                        let (x, y) = inventory_renderer::get_selected_slot(
                            self.cursor_position.x,
                            self.cursor_position.y,
                            physical_size,
                        );
                        if x >= inventory_renderer::SLOTS_PER_ROW || y != SLOTS_PER_COLUMN {
                            self.inventory_renderer.is_dragging = false;
                            return true;
                        }
                        let selected_tp = self
                            .inventory_renderer
                            .dragging_instance
                            .unwrap()
                            .block_type;

                        let new_item =
                            item::Item::new(item::ItemType::Block(all_block[selected_tp as usize]));
                        //println!("set hotbar[{}] = {:?}", y, new_item);
                        player.hotbar[x as usize] = new_item;
                        self.inventory_renderer.is_dragging = false;
                        self.inventory_renderer
                            .block_renderer
                            .recreate_hb_instance_buffer(queue, player);
                        self.inventory_renderer.block_renderer.update_iv_hb(
                            player,
                            physical_size,
                            queue,
                        );
                    }
                    return true;
                }
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::LineDelta(_, y) => {
                    if *y > 0.0 {
                        self.inventory_renderer.update_shb(-1, queue);
                        player.update_selected_hotbar(-1);
                    } else {
                        self.inventory_renderer.update_shb(1, queue);
                        player.update_selected_hotbar(1);
                    }
                    return true;
                }
                _ => {}
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position.x = position.x as f32;
                self.cursor_position.y = position.y as f32;
                if self.inventory_renderer.is_dragging {
                    self.inventory_renderer.update_dragging_instance(
                        self.cursor_position.x,
                        self.cursor_position.y,
                        queue,
                    );
                }
                //self.ui_text_renderer
                //    .set_text(format!("{:?}", self.cursor_position).as_str());
            }
            _ => {}
        }
        false
    }
}
