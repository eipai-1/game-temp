use wgpu::{util::DeviceExt, *};

const BLOCK_NUM: usize = 5;

use cgmath::*;

pub const TEXT_FRAC: f32 = 16.0 / 512.0;
pub const VERTICES: &[Vertex] = &[
    //方块坐标：其中每条边都从原点向每个轴的正方向延伸一格

    //正面---正常从正面看
    //后面统一按以下顺序
    //正面左上角
    Vertex {
        position: [0.0, 1.0, 1.0],
        tex_coords: [0.0, 0.0],
    },
    //正面右上角
    Vertex {
        position: [1.0, 1.0, 1.0],
        tex_coords: [TEXT_FRAC, 0.0],
    },
    //正面右下角
    Vertex {
        position: [1.0, 0.0, 1.0],
        tex_coords: [TEXT_FRAC, TEXT_FRAC],
    },
    //正面左下角
    Vertex {
        position: [0.0, 0.0, 1.0],
        tex_coords: [0.0, TEXT_FRAC],
    },
    //上面---从上面看---摄像机上方向为z轴负方向
    Vertex {
        position: [0.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [TEXT_FRAC, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, 1.0],
        tex_coords: [TEXT_FRAC, TEXT_FRAC],
    },
    Vertex {
        position: [0.0, 1.0, 1.0],
        tex_coords: [0.0, TEXT_FRAC],
    },
    //后面---摄像机上方向为y轴正方向
    Vertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [0.0, 1.0, 0.0],
        tex_coords: [TEXT_FRAC, 0.0],
    },
    Vertex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [TEXT_FRAC, TEXT_FRAC],
    },
    Vertex {
        position: [1.0, 0.0, 0.0],
        tex_coords: [0.0, TEXT_FRAC],
    },
    //下面--摄像机正方向为y轴正方向
    Vertex {
        position: [0.0, 0.0, 1.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, 0.0, 1.0],
        tex_coords: [TEXT_FRAC, 0.0],
    },
    Vertex {
        position: [1.0, 0.0, 0.0],
        tex_coords: [TEXT_FRAC, TEXT_FRAC],
    },
    Vertex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [0.0, TEXT_FRAC],
    },
    //左面--摄像机上方向y轴正向
    Vertex {
        position: [0.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [0.0, 1.0, 1.0],
        tex_coords: [TEXT_FRAC, 0.0],
    },
    Vertex {
        position: [0.0, 0.0, 1.0],
        tex_coords: [TEXT_FRAC, TEXT_FRAC],
    },
    Vertex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [0.0, TEXT_FRAC],
    },
    //右面--摄像机上方向y轴正向
    Vertex {
        position: [1.0, 1.0, 1.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [TEXT_FRAC, 0.0],
    },
    Vertex {
        position: [1.0, 0.0, 0.0],
        tex_coords: [TEXT_FRAC, TEXT_FRAC],
    },
    Vertex {
        position: [1.0, 0.0, 1.0],
        tex_coords: [0.0, TEXT_FRAC],
    },
];

#[rustfmt::skip]
pub const INDICES: &[u16] = &[
    0,  1,  2,  0,  2,  3, /* 之后每个加4就行 */ 
    4,  5,  6,  4,  6,  7,
    8,  9,  10, 8,  10, 11,
    12, 13, 14, 12, 14, 15,
    16, 17, 18, 16, 18, 19,
    20, 21, 22, 20, 22, 23,
];

pub const WIREFRAME_VERTICES: &[WireframeVertex] = &[
    WireframeVertex {
        position: [0.0, 1.001, 0.0],
    },
    WireframeVertex {
        position: [1.001, 1.001, 0.0],
    },
    WireframeVertex {
        position: [1.001, 1.001, 1.001],
    },
    WireframeVertex {
        position: [0.0, 1.001, 1.001],
    },
    WireframeVertex {
        position: [0.0, 0.0, 0.0],
    },
    WireframeVertex {
        position: [1.001, 0.0, 0.0],
    },
    WireframeVertex {
        position: [1.001, 0.0, 1.001],
    },
    WireframeVertex {
        position: [0.0, 0.0, 1.001],
    },
];

#[rustfmt::skip]
pub const WIREFRAME_INDCIES: &[u16] = &[
    0, 1, 1, 2, 2, 3, 3, 0,
    0, 4, 1, 5, 2, 6, 3, 7,
    4, 5, 5, 6, 6, 7, 7, 4,
];

const CHUNK_SIZE: usize = 4;
const CHUNK_HEIGHT: usize = 16;

#[repr(usize)]
#[derive(Clone, Copy, Default, Debug)]
pub enum BlockType {
    //没有方块 默认值
    #[default]
    Empty = 0,

    //基岩 世界基础
    UnderStone = 1,

    //石头
    Stone = 2,

    //草方块
    Grass = 3,

    //泥土
    Dirt = 4,
}

#[allow(unused)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Block {
    pub name: &'static str,
    pub block_type: BlockType,
    pub tex_offset: [[u8; 2]; 6],
}

impl Block {
    fn new(
        name: &'static str,
        //顺序为：正、上、后、下、左、右
        tex_offset: [[u8; 2]; 6],
        block_type: BlockType,
    ) -> Self {
        Self {
            name,
            block_type,
            tex_offset, //vertices,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WireframeVertex {
    pub position: [f32; 3],
}

impl WireframeVertex {
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<WireframeVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: VertexFormat::Float32x3,
            }],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub position: [f32; 3],
}

impl Instance {
    pub fn desc() -> VertexBufferLayout<'static> {
        use std::mem;
        VertexBufferLayout {
            array_stride: mem::size_of::<Instance>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[VertexAttribute {
                offset: 0,
                shader_location: 5,
                format: VertexFormat::Float32x3,
            }],
        }
    }
}

pub struct Chunk {
    position: Vector2<i32>,
    blocks: Vec<BlockType>,
}

impl Chunk {
    fn get_block_type(&self, x: usize, y: usize, z: usize) -> BlockType {
        return self.blocks[x * CHUNK_SIZE * CHUNK_HEIGHT + y * CHUNK_SIZE + z];
    }

    fn set_block_type(&mut self, x: usize, y: usize, z: usize, block_type: BlockType) {
        self.blocks[x * CHUNK_SIZE * CHUNK_HEIGHT + y * CHUNK_SIZE + z] = block_type;
    }
}

pub struct Realm {
    pub chunks: Vec<Chunk>,
    pub all_block: Vec<Block>,
    pub wf_vertex_buffer: Buffer,
    pub wf_index_buffer: Buffer,
    pub instances: Vec<Vec<Instance>>,
    pub instance_buffers: Vec<Buffer>,
    pub block_vertex_buffers: Vec<Buffer>,
}

impl Realm {
    pub fn new(device: &Device) -> Self {
        let mut all_block: Vec<Block> = vec![Block::default(); BLOCK_NUM];

        let empty = Block::new(
            "Empty",
            [[0, 0], [0, 0], [0, 0], [0, 0], [0, 0], [0, 0]],
            BlockType::Empty,
        );
        all_block[empty.block_type as usize] = empty;

        let under_stone = Block::new(
            "Under stone",
            [[1, 0], [1, 0], [1, 0], [1, 0], [1, 0], [1, 0]],
            BlockType::UnderStone,
        );
        all_block[under_stone.block_type as usize] = under_stone;

        let stone = Block::new(
            "stone",
            [[0, 0], [0, 0], [0, 0], [0, 0], [0, 0], [0, 0]],
            BlockType::Stone,
        );
        all_block[stone.block_type as usize] = stone;

        let dirt = Block::new(
            "dirt",
            [[4, 0], [4, 0], [4, 0], [4, 0], [4, 0], [4, 0]],
            BlockType::Dirt,
        );
        all_block[dirt.block_type as usize] = dirt;

        //创建草方块
        let grass = Block::new(
            "grass",
            [[3, 0], [2, 0], [3, 0], [4, 0], [3, 0], [3, 0]],
            BlockType::Grass,
        );
        all_block[grass.block_type as usize] = dirt;
        //草方块创建完成

        let mut block_vertex_buffers: Vec<Buffer> = Vec::new();
        for block in &all_block {
            let mut vertices: [Vertex; 24] = [Vertex {
                position: [0.0; 3],
                tex_coords: [0.0; 2],
            }; 24];

            for i in 0..6 {
                for j in 0..4 {
                    vertices[i * 4 + j].position = VERTICES[i * 4 + j].position;
                    vertices[i * 4 + j].tex_coords[0] = VERTICES[i * 4 + j].tex_coords[0]
                        + TEXT_FRAC * block.tex_offset[i][0] as f32;
                    vertices[i * 4 + j].tex_coords[1] = VERTICES[i * 4 + j].tex_coords[1]
                        + TEXT_FRAC * block.tex_offset[i][1] as f32;
                }
            }

            block_vertex_buffers.push(device.create_buffer_init(&util::BufferInitDescriptor {
                label: Some(block.name),
                contents: bytemuck::cast_slice(&vertices[..]),
                usage: BufferUsages::VERTEX,
            }));
        }

        let wf_vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("wireframe vertex buffer"),
            contents: bytemuck::cast_slice(&WIREFRAME_VERTICES[..]),
            usage: BufferUsages::VERTEX,
        });

        let wf_index_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Wireframe index buffer"),
            contents: bytemuck::cast_slice(WIREFRAME_INDCIES),
            usage: BufferUsages::INDEX,
        });

        let mut chunks: Vec<Chunk> = Vec::new();

        let mut instances: Vec<Vec<Instance>> = Vec::new();
        for _i in 0..BLOCK_NUM {
            instances.push(Vec::new());
        }

        let blocks = vec![BlockType::default(); CHUNK_SIZE * CHUNK_SIZE * CHUNK_HEIGHT];

        let mut chunk = Chunk {
            position: (0, 0).into(),
            blocks,
        };

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_HEIGHT {
                for z in 0..CHUNK_SIZE {
                    // if z > 9 {
                    //     chunk.set_block_type(x, y, z, BlockType::Empty);
                    // } else if z == 9 {
                    if y == 3 {
                        chunk.set_block_type(x, y, z, BlockType::Grass);
                    } else if y == 2 {
                        chunk.set_block_type(x, y, z, BlockType::Dirt);
                    } else if y == 1 {
                        chunk.set_block_type(x, y, z, BlockType::Stone);
                    } else if y == 0 {
                        chunk.set_block_type(x, y, z, BlockType::UnderStone);
                    }
                }
            }
        }

        chunks.push(chunk);
        for chunk in &chunks {
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_HEIGHT {
                    for z in 0..CHUNK_SIZE {
                        instances[chunk.get_block_type(x, y, z) as usize].push(Instance {
                            position: [x as f32, y as f32, z as f32],
                        });
                    }
                }
            }
        }

        let mut instance_buffers: Vec<Buffer> = Vec::new();
        for i in 0..BLOCK_NUM {
            instance_buffers.push(device.create_buffer_init(&util::BufferInitDescriptor {
                label: Some("Block buffer"),
                contents: bytemuck::cast_slice(&instances[i]),
                usage: BufferUsages::VERTEX,
            }));
        }

        Self {
            all_block,
            chunks,
            wf_vertex_buffer,
            wf_index_buffer,
            instances,
            instance_buffers,
            block_vertex_buffers,
        }
    }
}
