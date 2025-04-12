use crate::item::{Item, ItemType};
use crate::realm;
use crate::realm::{BlockInfo, BlockType};
use crate::texture;
use crate::ui::inventory_renderer;
use wgpu::util::DeviceExt;
use wgpu::*;

const HALF_HEAD_SIZE: f32 = 0.15;
const HEAD_HEIGHT: f32 = 0.25;
const HALF_BODY_SIZE: f32 = 0.25;
const HALF_BODY_WIDTH: f32 = 0.125;
const HALF_BODY_LENGTH: f32 = 0.25;
const BODY_HEIGHT: f32 = 0.70;
const LEG_HEIGHT: f32 = 0.85;
const LEG_SIZE: f32 = 0.25;
const ARM_HEIGHT: f32 = 0.7;
const ARM_SIZE: f32 = 0.25;
const ARM_OFFSET: f32 = 0.1;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct EntityVertex {
    pub position: [f32; 3],
    pub tex_coord: [f32; 2],
}

impl EntityVertex {
    fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<EntityVertex>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct EntityInstance {
    pub position: [f32; 3],
}

impl EntityInstance {
    fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<EntityInstance>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: 0,
                shader_location: 2,
            }],
        }
    }
}

#[allow(unused)]
pub struct Player {
    pub slected_hotbar: i32,
    pub hotbar: Vec<Item>,
    pub all_item_inventory: Vec<Vec<Item>>,
    vertices: Vec<EntityVertex>,
    indices: Vec<u16>,
    instances: Vec<EntityInstance>,
    instance_buffer: Buffer,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    texture_bind_group: BindGroup,
    render_pipeline: RenderPipeline,
}

impl Player {
    pub fn new(
        all_block: &Vec<BlockInfo>,
        device: &Device,
        queue: &Queue,
        camera_bind_group_layout: &BindGroupLayout,
        format: TextureFormat,
    ) -> Self {
        let mut hotbar = vec![Item::new(ItemType::Empty); 10];
        hotbar[0] = Item::new(ItemType::Block(all_block[BlockType::Grass as usize]));
        hotbar[1] = Item::new(ItemType::Block(all_block[BlockType::Dirt as usize]));
        hotbar[2] = Item::new(ItemType::Block(all_block[BlockType::UnderStone as usize]));
        hotbar[3] = Item::new(ItemType::Block(all_block[BlockType::BirchLog as usize]));
        let mut all_item_inventory = vec![vec![Item::new(ItemType::Empty); 10]; 4];

        all_item_inventory[0][0] =
            Item::new(ItemType::Block(all_block[BlockType::UnderStone as usize]));
        all_item_inventory[0][1] =
            Item::new(ItemType::Block(all_block[BlockType::BirchLog as usize]));
        all_item_inventory[0][2] =
            Item::new(ItemType::Block(all_block[BlockType::BirchLeaves as usize]));
        all_item_inventory[0][3] = Item::new(ItemType::Block(all_block[BlockType::Grass as usize]));
        all_item_inventory[0][4] = Item::new(ItemType::Block(all_block[BlockType::Dirt as usize]));
        all_item_inventory[0][5] = Item::new(ItemType::Block(all_block[BlockType::Stone as usize]));
        all_item_inventory[0][6] =
            Item::new(ItemType::Block(all_block[BlockType::BirchPlank as usize]));
        all_item_inventory[0][7] =
            Item::new(ItemType::Block(all_block[BlockType::TestBlock as usize]));

        let vertices = Self::create_vertices();
        let indices = Self::create_indices();
        let vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });

        let instances = vec![
            EntityInstance {
                position: [1.0, 70.0, 1.0],
            };
            1
        ];

        let instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let skin_bytes = include_bytes!("../res/texture/skin.png");
        let skin_texure =
            texture::Texture::from_bytes(device, queue, skin_bytes, "skin texture").unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Texture bind group layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture bind group"),
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&skin_texure.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&skin_texure.sampler),
                },
            ],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render pipeline layout"),
            bind_group_layouts: &[&texture_bind_group_layout, camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Player Shader"),
            source: ShaderSource::Wgsl(include_str!("player_shader.wgsl").into()),
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("First render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[EntityVertex::desc(), EntityInstance::desc()],
                compilation_options: PipelineCompilationOptions::default(),
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Cw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });

        Self {
            slected_hotbar: 0,
            hotbar,
            all_item_inventory,
            vertices,
            indices,
            instances,
            vertex_buffer,
            instance_buffer,
            index_buffer,
            render_pipeline,
            texture_bind_group,
        }
    }

    pub fn update_selected_hotbar(&mut self, offset: i32) {
        self.slected_hotbar += offset;
        self.slected_hotbar += inventory_renderer::SLOTS_PER_ROW as i32;
        self.slected_hotbar %= inventory_renderer::SLOTS_PER_ROW as i32;
    }

    //按 头 -> 身体 -> 手臂 -> 腿 的顺序
    fn create_vertices() -> Vec<EntityVertex> {
        let mut vertices: Vec<EntityVertex> = Vec::new();

        // 头
        for i in 0..realm::VERTICES.len() {
            vertices.push(EntityVertex {
                position: [
                    realm::VERTICES[i].position[0] * HALF_HEAD_SIZE * 2.0 - HALF_HEAD_SIZE,
                    realm::VERTICES[i].position[1] * HEAD_HEIGHT + BODY_HEIGHT + LEG_HEIGHT,
                    realm::VERTICES[i].position[2] * HALF_HEAD_SIZE * 2.0 - HALF_HEAD_SIZE,
                ],
                tex_coord: [
                    realm::VERTICES[i].tex_coord[0],
                    realm::VERTICES[i].tex_coord[1],
                ],
            });
        }

        // 身体
        for i in 0..realm::VERTICES.len() {
            vertices.push(EntityVertex {
                position: [
                    realm::VERTICES[i].position[0] * HALF_BODY_LENGTH * 2.0 - HALF_BODY_LENGTH,
                    realm::VERTICES[i].position[1] * BODY_HEIGHT + LEG_HEIGHT,
                    realm::VERTICES[i].position[2] * HALF_BODY_WIDTH * 2.0 - HALF_BODY_WIDTH,
                ],
                tex_coord: [
                    realm::VERTICES[i].tex_coord[0],
                    realm::VERTICES[i].tex_coord[1],
                ],
            });
        }

        // 手臂
        for i in 0..realm::VERTICES.len() {
            vertices.push(EntityVertex {
                position: [
                    realm::VERTICES[i].position[0] * ARM_SIZE
                        - ARM_SIZE / 2.0
                        - HALF_BODY_LENGTH
                        - ARM_SIZE / 2.0,
                    realm::VERTICES[i].position[1] * ARM_HEIGHT + BODY_HEIGHT + ARM_OFFSET,
                    realm::VERTICES[i].position[2] * ARM_SIZE - ARM_SIZE / 2.0,
                ],
                tex_coord: [
                    realm::VERTICES[i].tex_coord[0],
                    realm::VERTICES[i].tex_coord[1],
                ],
            });
        }

        for i in 0..realm::VERTICES.len() {
            vertices.push(EntityVertex {
                position: [
                    realm::VERTICES[i].position[0] * ARM_SIZE - ARM_SIZE / 2.0
                        + HALF_BODY_LENGTH
                        + ARM_SIZE / 2.0,
                    realm::VERTICES[i].position[1] * ARM_HEIGHT + BODY_HEIGHT + ARM_OFFSET,
                    realm::VERTICES[i].position[2] * ARM_SIZE - ARM_SIZE / 2.0,
                ],
                tex_coord: [
                    realm::VERTICES[i].tex_coord[0],
                    realm::VERTICES[i].tex_coord[1],
                ],
            });
        }

        for i in 0..realm::VERTICES.len() {
            vertices.push(EntityVertex {
                position: [
                    realm::VERTICES[i].position[0] * LEG_SIZE - LEG_SIZE / 2.0 - LEG_SIZE / 2.0,
                    realm::VERTICES[i].position[1] * LEG_HEIGHT,
                    realm::VERTICES[i].position[2] * LEG_SIZE - LEG_SIZE / 2.0,
                ],
                tex_coord: [
                    realm::VERTICES[i].tex_coord[0],
                    realm::VERTICES[i].tex_coord[1],
                ],
            });
        }
        for i in 0..realm::VERTICES.len() {
            vertices.push(EntityVertex {
                position: [
                    realm::VERTICES[i].position[0] * LEG_SIZE - LEG_SIZE / 2.0 + LEG_SIZE / 2.0,
                    realm::VERTICES[i].position[1] * LEG_HEIGHT,
                    realm::VERTICES[i].position[2] * LEG_SIZE - LEG_SIZE / 2.0,
                ],
                tex_coord: [
                    realm::VERTICES[i].tex_coord[0],
                    realm::VERTICES[i].tex_coord[1],
                ],
            });
        }
        vertices
    }

    fn create_indices() -> Vec<u16> {
        let temp: Vec<u16> = Vec::new();
        let mut indcies: Vec<u16> = vec![
            0, 1, 2, 0, 2, 3, /* 之后每个加4就行 */
            4, 5, 6, 4, 6, 7, 8, 9, 10, 8, 10, 11, 12, 13, 14, 12, 14, 15, 16, 17, 18, 16, 18, 19,
            20, 21, 22, 20, 22, 23,
        ];

        for i in 1..=5 {
            for j in 0..36 {
                indcies.push(indcies[j] + i * 24);
            }
        }

        indcies
    }

    pub fn draw_entities(&self, render_pass: &mut RenderPass, camera_bind_group: &BindGroup) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
        render_pass.set_bind_group(1, camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(
            0..self.indices.len() as u32,
            0,
            0..self.instances.len() as u32,
        );
    }
}
