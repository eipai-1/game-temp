use crate::realm;
use crate::realm::*;
use crate::texture;
use wgpu::{util::DeviceExt, *};

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
    pub instances: Vec<realm::Instance>,
    pub render_pipeline: RenderPipeline,
    pub instance_buffer: Buffer,
}

impl BlockRenderer {
    pub fn new(
        device: &Device,
        format: TextureFormat,
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

        let mut instances: Vec<realm::Instance> = Vec::new();
        instances.push(realm::Instance {
            position: [
                178.0 + ICON_OFFSET_X,
                30.0 + BLOCK_SIZE + ICON_OFFSET_Y,
                0.0,
            ],
            block_type: BlockType::Grass as u32,
        });
        let instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Block icon instance buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: BufferUsages::VERTEX,
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
        BlockRenderer {
            vertex_buffer,
            index_buffer,
            instances,
            render_pipeline,
            instance_buffer,
        }
    }
}
