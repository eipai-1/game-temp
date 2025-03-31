use crate::entity::Player;
use crate::item;
use crate::item::Item;
use crate::realm;
use crate::realm::*;
use crate::ui::inventory_renderer;
use wgpu::{util::DeviceExt, *};

use super::inventory_renderer::{HOTBAR_TOP, SLOT_SPACING};

//修改BLOCK_SIZE的话记得同步修改shader中的BLOCK_SIZE
const BLOCK_SIZE: f32 = 64.0;
const ICON_OFFSET_Y: f32 = -17.0;
const ICON_OFFSET_X: f32 = 5.0;

pub const VERTICES: &[Vertex] = &[
    // 方块坐标：其中每条边都从原点向每个轴的正方向延伸一格
    // 按照正-上-后-下-左-右的顺序

    // 正面
    Vertex {
        position: [0.0, BLOCK_SIZE, BLOCK_SIZE],
        tex_coord: [0.0, 0.0],
    },
    Vertex {
        position: [BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE],
        tex_coord: [1.0, 0.0],
    },
    Vertex {
        position: [BLOCK_SIZE, 0.0, BLOCK_SIZE],
        tex_coord: [1.0, 1.0],
    },
    Vertex {
        position: [0.0, 0.0, BLOCK_SIZE],
        tex_coord: [0.0, 1.0],
    },
    // 上面
    Vertex {
        position: [0.0, BLOCK_SIZE, 0.0],
        tex_coord: [0.0, 0.0],
    },
    Vertex {
        position: [BLOCK_SIZE, BLOCK_SIZE, 0.0],
        tex_coord: [1.0, 0.0],
    },
    Vertex {
        position: [BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE],
        tex_coord: [1.0, 1.0],
    },
    Vertex {
        position: [0.0, BLOCK_SIZE, BLOCK_SIZE],
        tex_coord: [0.0, 1.0],
    },
    // 后面
    Vertex {
        position: [BLOCK_SIZE, BLOCK_SIZE, 0.0],
        tex_coord: [0.0, 0.0],
    },
    Vertex {
        position: [0.0, BLOCK_SIZE, 0.0],
        tex_coord: [1.0, 0.0],
    },
    Vertex {
        position: [0.0, 0.0, 0.0],
        tex_coord: [1.0, 1.0],
    },
    Vertex {
        position: [BLOCK_SIZE, 0.0, 0.0],
        tex_coord: [0.0, 1.0],
    },
    // 下面
    Vertex {
        position: [0.0, 0.0, BLOCK_SIZE],
        tex_coord: [0.0, 0.0],
    },
    Vertex {
        position: [BLOCK_SIZE, 0.0, BLOCK_SIZE],
        tex_coord: [1.0, 0.0],
    },
    Vertex {
        position: [BLOCK_SIZE, 0.0, 0.0],
        tex_coord: [1.0, 1.0],
    },
    Vertex {
        position: [0.0, 0.0, 0.0],
        tex_coord: [0.0, 1.0],
    },
    // 左面
    Vertex {
        position: [0.0, BLOCK_SIZE, 0.0],
        tex_coord: [0.0, 0.0],
    },
    Vertex {
        position: [0.0, BLOCK_SIZE, BLOCK_SIZE],
        tex_coord: [1.0, 0.0],
    },
    Vertex {
        position: [0.0, 0.0, BLOCK_SIZE],
        tex_coord: [1.0, 1.0],
    },
    Vertex {
        position: [0.0, 0.0, 0.0],
        tex_coord: [0.0, 1.0],
    },
    // 右面
    Vertex {
        position: [BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE],
        tex_coord: [0.0, 0.0],
    },
    Vertex {
        position: [BLOCK_SIZE, BLOCK_SIZE, 0.0],
        tex_coord: [1.0, 0.0],
    },
    Vertex {
        position: [BLOCK_SIZE, 0.0, 0.0],
        tex_coord: [1.0, 1.0],
    },
    Vertex {
        position: [BLOCK_SIZE, 0.0, BLOCK_SIZE],
        tex_coord: [0.0, 1.0],
    },
];

pub struct BlockRenderer {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub hb_instances: Vec<realm::Instance>,
    pub render_pipeline: RenderPipeline,
    pub hb_instance_buffer: Buffer,
    pub all_item_inventory_instances: Vec<realm::Instance>,
    pub all_item_inventory_instance_buffer: Buffer,

    //打开物品栏时的快捷栏的实例
    pub iv_hb_instance_buffer: Buffer,
}

impl BlockRenderer {
    pub fn new(
        device: &Device,
        format: TextureFormat,
        player: &Player,
        physical_size: winit::dpi::PhysicalSize<u32>,
        screen_size_uniform_bind_group_layout: &BindGroupLayout,
        block_materials_bind_group_layout: &BindGroupLayout,
        texture_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Block Icon Shader"),
            source: ShaderSource::Wgsl(include_str!("block_icon_shader.wgsl").into()),
        });
        let vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Block icon vertex buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Block icon index buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: BufferUsages::INDEX,
        });

        let mut hb_instances: Vec<realm::Instance> = Vec::new();

        for (i, item) in player.hotbar.iter().enumerate() {
            hb_instances.push(realm::Instance {
                position: [
                    i as f32 * (BLOCK_SIZE + inventory_renderer::SLOT_SPACING)
                        + inventory_renderer::HOTBAR_LEFT
                        + ICON_OFFSET_X,
                    HOTBAR_TOP + BLOCK_SIZE + ICON_OFFSET_Y,
                    0.0,
                ],
                block_type: item.item_type.get_type(),
            });
        }

        //instances.push(realm::Instance {
        //    position: [
        //        178.0 + ICON_OFFSET_X,
        //        30.0 + BLOCK_SIZE + ICON_OFFSET_Y,
        //        0.0,
        //    ],
        //    block_type: BlockType::Grass as u32,
        //});
        let hb_instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Block icon instance buffer"),
            contents: bytemuck::cast_slice(&hb_instances),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[
                screen_size_uniform_bind_group_layout,
                texture_bind_group_layout,
                block_materials_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("UI render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), realm::Instance::desc()],
                compilation_options: PipelineCompilationOptions::default(),
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Cw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });

        let all_item_inventory_instances = BlockRenderer::crate_all_item_inventory_instances(
            &player.all_item_inventory,
            physical_size,
        );
        let all_item_inventory_instance_buffer =
            device.create_buffer_init(&util::BufferInitDescriptor {
                label: Some("Block icon instance buffer"),
                contents: bytemuck::cast_slice(&all_item_inventory_instances),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });

        let iv_hb_instance = Self::create_iv_hb_instance(player, physical_size);
        let iv_hb_instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Block icon instance buffer"),
            contents: bytemuck::cast_slice(&iv_hb_instance),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        BlockRenderer {
            vertex_buffer,
            index_buffer,
            hb_instances,
            render_pipeline,
            hb_instance_buffer,
            all_item_inventory_instances,
            all_item_inventory_instance_buffer,
            iv_hb_instance_buffer,
        }
    }

    pub fn draw_hb(
        &self,
        render_pass: &mut RenderPass,
        screen_size_uniform_bind_group: &BindGroup,
        diffuse_bind_group: &BindGroup,
        block_materials_bind_group: &BindGroup,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, screen_size_uniform_bind_group, &[]);
        render_pass.set_bind_group(1, diffuse_bind_group, &[]);
        render_pass.set_bind_group(2, block_materials_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.hb_instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(
            0..realm::INDICES.len() as _,
            0,
            0..self.hb_instances.len() as _,
        );
    }

    pub fn draw_iv_hb(
        &self,
        render_pass: &mut RenderPass,
        screen_size_uniform_bind_group: &BindGroup,
        diffuse_bind_group: &BindGroup,
        block_materials_bind_group: &BindGroup,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, screen_size_uniform_bind_group, &[]);
        render_pass.set_bind_group(1, diffuse_bind_group, &[]);
        render_pass.set_bind_group(2, block_materials_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.iv_hb_instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(
            0..realm::INDICES.len() as _,
            0,
            0..self.hb_instances.len() as _,
        );
    }

    pub fn draw_all_item(
        &self,
        render_pass: &mut RenderPass,
        screen_size_uniform_bind_group: &BindGroup,
        diffuse_bind_group: &BindGroup,
        block_materials_bind_group: &BindGroup,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, screen_size_uniform_bind_group, &[]);
        render_pass.set_bind_group(1, diffuse_bind_group, &[]);
        render_pass.set_bind_group(2, block_materials_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.all_item_inventory_instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(
            0..realm::INDICES.len() as _,
            0,
            0..self.all_item_inventory_instances.len() as _,
        );
    }

    pub fn draw_dragging(
        &self,
        render_pass: &mut RenderPass,
        screen_size_uniform_bind_group: &BindGroup,
        diffuse_bind_group: &BindGroup,
        block_materials_bind_group: &BindGroup,
        dragging_instance_buffer: &Buffer,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, screen_size_uniform_bind_group, &[]);
        render_pass.set_bind_group(1, diffuse_bind_group, &[]);
        render_pass.set_bind_group(2, block_materials_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, dragging_instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(0..realm::INDICES.len() as _, 0, 0..1);
    }

    fn crate_all_item_inventory_instances(
        all_item_inventory: &Vec<Vec<Item>>,
        physical_size: winit::dpi::PhysicalSize<u32>,
    ) -> Vec<realm::Instance> {
        let mut all_item_inventory_instances: Vec<realm::Instance> = Vec::new();
        for (i, items) in all_item_inventory.iter().enumerate() {
            for (j, item) in items.iter().enumerate() {
                if item.item_type.get_type() == BlockType::Empty as u32 {
                    return all_item_inventory_instances;
                }

                let mut x = physical_size.width as f32 / 2.0 - inventory_renderer::IV_WIDTH / 2.0;
                x += (j as f32 * (BLOCK_SIZE + inventory_renderer::SLOT_SPACING))
                    + inventory_renderer::SLOT_SPACING
                    + ICON_OFFSET_X;

                let mut y = physical_size.height as f32 / 2.0 - inventory_renderer::IV_HEIGHT / 2.0;
                y += (i as f32 * (BLOCK_SIZE + inventory_renderer::SLOT_SPACING))
                    + inventory_renderer::SLOT_SPACING
                    + BLOCK_SIZE
                    + ICON_OFFSET_Y;
                all_item_inventory_instances.push(realm::Instance {
                    position: [x, y, 0.0],
                    block_type: item.item_type.get_type(),
                });
            }
        }
        all_item_inventory_instances
    }

    pub fn recreate_hb_instance_buffer(&mut self, queue: &wgpu::Queue, player: &Player) {
        self.hb_instances.clear();
        for (i, item) in player.hotbar.iter().enumerate() {
            self.hb_instances.push(realm::Instance {
                position: [
                    i as f32 * (BLOCK_SIZE + inventory_renderer::SLOT_SPACING)
                        + inventory_renderer::HOTBAR_LEFT
                        + ICON_OFFSET_X,
                    HOTBAR_TOP + BLOCK_SIZE + ICON_OFFSET_Y,
                    0.0,
                ],
                block_type: item.item_type.get_type(),
            });
        }
        queue.write_buffer(
            &self.hb_instance_buffer,
            0,
            bytemuck::cast_slice(&self.hb_instances),
        );
    }

    fn create_iv_hb_instance(
        player: &Player,
        physical_size: winit::dpi::PhysicalSize<u32>,
    ) -> Vec<realm::Instance> {
        let mut iv_hb_instance: Vec<realm::Instance> = Vec::new();
        let mut x = physical_size.width as f32 / 2.0 - inventory_renderer::IV_WIDTH / 2.0
            + SLOT_SPACING
            + ICON_OFFSET_X;
        let mut y = physical_size.height as f32 / 2.0 - inventory_renderer::IV_HEIGHT / 2.0
            + BLOCK_SIZE
            + SLOT_SPACING
            + ICON_OFFSET_Y;
        y += (BLOCK_SIZE + SLOT_SPACING) * 4.0;
        for item in player.hotbar.iter() {
            iv_hb_instance.push(realm::Instance {
                position: [x, y, 0.0],
                block_type: item.item_type.get_type(),
            });
            x += BLOCK_SIZE + SLOT_SPACING;
        }
        iv_hb_instance
    }

    pub fn update_iv_hb(
        &self,
        player: &Player,
        physical_size: winit::dpi::PhysicalSize<u32>,
        queue: &wgpu::Queue,
    ) {
        let iv_hb_instance = Self::create_iv_hb_instance(player, physical_size);
        queue.write_buffer(
            &self.iv_hb_instance_buffer,
            0,
            bytemuck::cast_slice(&iv_hb_instance),
        );
    }

    pub fn resize(
        &self,
        queue: &wgpu::Queue,
        player: &Player,
        physical_size: winit::dpi::PhysicalSize<u32>,
    ) {
        let all_item_inventory_instances = BlockRenderer::crate_all_item_inventory_instances(
            &player.all_item_inventory,
            physical_size,
        );
        queue.write_buffer(
            &self.all_item_inventory_instance_buffer,
            0,
            bytemuck::cast_slice(&all_item_inventory_instances),
        );

        let iv_hb_instance = Self::create_iv_hb_instance(player, physical_size);
        queue.write_buffer(
            &self.iv_hb_instance_buffer,
            0,
            bytemuck::cast_slice(&iv_hb_instance),
        );
    }
}
