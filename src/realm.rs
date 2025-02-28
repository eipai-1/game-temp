use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Serialize};
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

const WF_SIZE: f32 = 0.01;
const WF_WIDTH: f32 = 0.1;

#[rustfmt::skip]
pub const INDICES: &[u16] = &[
    0,  1,  2,  0,  2,  3, /* 之后每个加4就行 */ 
    4,  5,  6,  4,  6,  7,
    8,  9,  10, 8,  10, 11,
    12, 13, 14, 12, 14, 15,
    16, 17, 18, 16, 18, 19,
    20, 21, 22, 20, 22, 23,
];

#[rustfmt::skip]
pub const WIREFRAME_INDCIES: &[u16] = &[
    //顶面
    0,  4,  7,  7,  3,  0,
    5,  4,  8,  5,  8,  9,
    15, 11, 8,  15, 8,  12,
    0,  1,  13, 0,  13, 12,

    //正面
    12, 8,  10, 12, 10, 14,
    9,  8,  24, 9,  24, 25,
    30, 26, 24, 30, 24, 28,
    12, 13, 29, 12, 29, 28,

    //后
    4,  0,  2,  4,  2,  6,
    1,  0,  16, 1,  16, 17,
    22, 18, 16, 22, 16, 20,
    4,  5,  21, 4,  21, 20,

    //下
    28, 24, 27, 28, 27, 31,
    25, 24, 20, 25, 20, 21,
    19, 23, 20, 19, 20, 16,
    28, 29, 17, 28, 17, 16,

    //左
    0,  12, 14, 0,  14, 2,
    15, 12, 28, 15, 28, 31,
    18, 30, 28, 18, 28, 16,
    0,  3,  19, 0,  19, 16,

    //右
    8,  4,  6,  8,  6,  10,
    7,  4,  20, 7,  20, 23,
    26, 22, 20, 26, 20, 24,
    8,  11, 27, 8,  27, 24,
];

const CHUNK_SIZE: usize = 4;
const CHUNK_HEIGHT: usize = 16;

const WORLD_FILE_DIR: &str = "./worlds";

#[repr(usize)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Serialize, Deserialize)]
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

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WireframeUniform {
    pub position: [f32; 3],
    _padding: f32,
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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Chunk {
    blocks: Vec<BlockType>,
    x: i32,
    z: i32,
}

impl Chunk {
    fn get_block_type(&self, x: usize, y: usize, z: usize) -> BlockType {
        return self.blocks[x * CHUNK_SIZE * CHUNK_HEIGHT + y * CHUNK_SIZE + z];
    }

    fn set_block_type(&mut self, x: usize, y: usize, z: usize, block_type: BlockType) {
        self.blocks[x * CHUNK_SIZE * CHUNK_HEIGHT + y * CHUNK_SIZE + z] = block_type;
    }

    fn save(&self, world_dir: &str) -> anyhow::Result<()> {
        let path = Path::new(world_dir)
            .join("chunks")
            .join(format!("x{}", self.x))
            .join(format!("y{}.chunk", self.z));

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("创建区块目录失败")?;
        }

        let encoded = bincode::serialize(self).context("区块序列化失败")?;

        std::fs::write(&path, encoded).context("写入区块失败")?;

        Ok(())
    }

    fn load(world_dir: &str, x: i32, z: i32) -> anyhow::Result<Self> {
        let path = Path::new(world_dir)
            .join("chunks")
            .join(format!("x{}", x))
            .join(format!("y{}.chunk", z));

        let data = std::fs::read(&path).context("读取区块文件失败")?;

        let chunk: Chunk = bincode::deserialize(&data).context("解析区块数据失败")?;

        if chunk.blocks.len() != CHUNK_SIZE * CHUNK_SIZE * CHUNK_HEIGHT {
            anyhow::bail!("区块数据损坏：方块数量不匹配");
        }

        Ok(chunk)
    }
}

pub struct Realm {
    pub data: RealmData,
    pub render_res: RenderResources,
}

pub struct RealmData {
    pub chunks: Vec<Chunk>,
    pub all_block: Vec<Block>,
    pub wf_max_len: f32,
    pub instances: Vec<Vec<Instance>>,

    pub wf_uniform: WireframeUniform,
    pub is_wf_visible: bool,

    pub center_chunk_pos: Point2<i32>,

    pub name: &'static str,
}

impl RealmData {
    pub fn new() -> Self {
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
        all_block[grass.block_type as usize] = grass;
        //草方块创建完成
        let mut chunks: Vec<Chunk> = Vec::new();

        let mut instances: Vec<Vec<Instance>> = Vec::new();
        for _i in 0..BLOCK_NUM {
            instances.push(Vec::new());
        }

        let blocks = vec![BlockType::default(); CHUNK_SIZE * CHUNK_SIZE * CHUNK_HEIGHT];

        let mut chunk = Chunk { x: 0, z: 0, blocks };

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

                    //游戏使用右手坐标系 拿出右手，食指方向为y轴，大拇指方向为x轴时，此时中指方向为z轴
                    //if x == 0 && y == 0 && z == 0 {
                    //    chunk.set_block_type(x, y, z, BlockType::Dirt);
                    //} else if x == 1 && y == 0 && z == 0 {
                    //    chunk.set_block_type(x, y, z, BlockType::UnderStone);
                    //} else if x == 0 && y == 1 && z == 0 {
                    //    chunk.set_block_type(x, y, z, BlockType::Grass);
                    //} else if x == 0 && y == 0 && z == 1 {
                    //    chunk.set_block_type(x, y, z, BlockType::Stone);
                    //}
                }
            }
        }

        chunks.push(chunk);
        for chunk in &chunks {
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_HEIGHT {
                    for z in 0..CHUNK_SIZE {
                        let tp = chunk.get_block_type(x, y, z);
                        if tp != BlockType::Empty {
                            instances[tp as usize].push(Instance {
                                position: [x as f32, y as f32, z as f32],
                            });
                        }
                    }
                }
            }
        }

        let wf_uniform = WireframeUniform {
            position: [0.0, 0.0, 0.0],
            _padding: 0.0,
        };

        let wf_max_len: f32 = 8.0;

        let is_wf_visible = true;

        let center_chunk_pos = Point2 { x: 0, y: 0 };

        let name = "./data/worlds/default_name_1";

        Self {
            all_block,
            chunks,
            instances,
            wf_uniform,
            wf_max_len,
            is_wf_visible,
            center_chunk_pos,
            name,
        }
    }

    pub fn get_block_type(&self, mut x: i32, y: i32, mut z: i32) -> BlockType {
        let chunk_x = (x as f32 / CHUNK_SIZE as f32).floor() as i32;
        let chunk_z = (z as f32 / CHUNK_SIZE as f32).floor() as i32;

        let i32_size = CHUNK_SIZE as i32;
        x = x.rem_euclid(i32_size);
        z = z.rem_euclid(i32_size);

        //println!("chunk_x:{}, chunk_z:{} ", chunk_x, chunk_z);
        //println!("x={}, y={}, z={}", x, y, z);
        if x < 0 || z < 0 {
            panic!("PANIC! x or z less than zero: x={},z={}", x, z);
        }

        //对纵坐标位于区块外的方块，统一检测为虚空
        //区块范围为[0, CHUNK_HEIGHT]
        if y < 0 || y >= CHUNK_HEIGHT as i32 {
            return BlockType::Empty;
        }

        for chunk in &self.chunks {
            if chunk.x == chunk_x && chunk.z == chunk_z {
                return chunk.blocks[x as usize * CHUNK_SIZE * CHUNK_HEIGHT
                    + y as usize * CHUNK_SIZE
                    + z as usize];
            }
        }

        BlockType::Empty
    }

    pub fn update_wf_uniform(&mut self, new_position: Point3<i32>) {
        self.wf_uniform.position = [
            new_position.x as f32,
            new_position.y as f32,
            new_position.z as f32,
        ];
    }
}

pub struct RenderResources {
    //pub wf_vertices: Vec<WireframeVertex>,
    pub wf_vertex_buffer: Buffer,
    pub wf_index_buffer: Buffer,
    pub instance_buffers: Vec<Buffer>,
    pub block_vertex_buffers: Vec<Buffer>,
    pub wf_uniform_buffer: Buffer,
}

impl RenderResources {
    fn new(device: &Device, data: &RealmData) -> Self {
        let mut block_vertex_buffers: Vec<Buffer> = Vec::new();
        for block in &data.all_block {
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

        let wf_vertices = generate_wf_vertices();

        let wf_vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("wireframe vertex buffer"),
            contents: bytemuck::cast_slice(&wf_vertices),
            usage: BufferUsages::VERTEX,
        });

        let wf_index_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Wireframe index buffer"),
            contents: bytemuck::cast_slice(WIREFRAME_INDCIES),
            usage: BufferUsages::INDEX,
        });

        let wf_uniform_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Wireframe uniform buffer"),
            contents: bytemuck::bytes_of(&data.wf_uniform),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let mut instance_buffers: Vec<Buffer> = Vec::new();
        for i in 0..BLOCK_NUM {
            instance_buffers.push(device.create_buffer_init(&util::BufferInitDescriptor {
                label: Some("Block buffer"),
                contents: bytemuck::cast_slice(&data.instances[i]),
                usage: BufferUsages::VERTEX,
            }));
        }
        Self {
            block_vertex_buffers,
            wf_index_buffer,
            wf_vertex_buffer,
            wf_uniform_buffer,
            instance_buffers,
            //wf_vertices,
        }
    }
}

impl Realm {
    pub fn new(device: &Device) -> Self {
        let data = RealmData::new();
        let render_res = RenderResources::new(device, &data);

        Self { data, render_res }
    }
}

fn generate_wf_vertices() -> Vec<WireframeVertex> {
    //方块8顶点
    let mut v8 = vec![
        //顶面4顶点，从上往下看，顺时针方向，从落在y轴上的点开始
        WireframeVertex {
            position: [0.0, 1.0, 0.0],
        },
        WireframeVertex {
            position: [1.0, 1.0, 0.0],
        },
        WireframeVertex {
            position: [1.0, 1.0, 1.0],
        },
        WireframeVertex {
            position: [0.0, 1.0, 1.0],
        },
        //底面4顶点，还是同样顺序
        WireframeVertex {
            position: [0.0, 0.0, 0.0],
        },
        WireframeVertex {
            position: [1.0, 0.0, 0.0],
        },
        WireframeVertex {
            position: [1.0, 0.0, 1.0],
        },
        WireframeVertex {
            position: [0.0, 0.0, 1.0],
        },
    ];

    #[rustfmt::skip]
    let dir8 = vec![
        Vector3 { x: 1, y: -1, z: 1 },
        Vector3 { x: -1, y: -1, z: 1 },
        Vector3 { x: -1, y: -1, z: -1 },
        Vector3 { x: 1, y: -1, z: -1 },
        //底面4个
        Vector3 { x: 1, y: 1, z: 1 },
        Vector3 { x: -1, y: 1, z: 1 },
        Vector3 { x: -1, y: 1, z: -1 },
        Vector3 { x: 1, y: 1, z: -1 },
    ];

    let mut v32: Vec<WireframeVertex> = Vec::new();

    for (i, v) in v8.iter_mut().enumerate() {
        if dir8[i].x > 0 {
            v.position[0] -= WF_SIZE;
        } else {
            v.position[0] += WF_SIZE;
        }

        if dir8[i].y > 0 {
            v.position[1] -= WF_SIZE;
        } else {
            v.position[1] += WF_SIZE;
        }

        if dir8[i].z > 0 {
            v.position[2] -= WF_SIZE;
        } else {
            v.position[2] += WF_SIZE;
        }

        v32.push(*v);

        let mut new_v = *v;
        if dir8[i].x > 0 {
            new_v.position[0] += WF_WIDTH;
        } else {
            new_v.position[0] -= WF_WIDTH;
        }
        v32.push(new_v);

        new_v = *v;
        if dir8[i].y > 0 {
            new_v.position[1] += WF_WIDTH;
        } else {
            new_v.position[1] -= WF_WIDTH;
        }
        v32.push(new_v);

        new_v = *v;
        if dir8[i].z > 0 {
            new_v.position[2] += WF_WIDTH;
        } else {
            new_v.position[2] -= WF_WIDTH;
        }
        v32.push(new_v);
    }

    //println!("{:#?}", v32);
    v32
}

#[cfg(test)]
mod tests {
    use crate::realm::BlockType;

    use super::RealmData;
    use super::*;

    #[test]
    fn test_get_set_block_type() {
        let data = RealmData::new();

        assert_eq!(data.get_block_type(0, 0, 0), BlockType::UnderStone);
        assert_eq!(data.get_block_type(0, 1, 0), BlockType::Stone);
        assert_eq!(data.get_block_type(0, 2, 0), BlockType::Dirt);
        assert_eq!(data.get_block_type(0, 3, 0), BlockType::Grass);
        assert_eq!(data.get_block_type(0, 0, -1), BlockType::Empty);
    }

    #[test]
    fn test_chunk_file() -> anyhow::Result<()> {
        let data = RealmData::new();
        data.chunks[0].save(data.name)?;

        let chunk = Chunk::load(data.name, 0, 0).unwrap();
        assert_eq!(chunk, data.chunks[0]);
        Ok(())
    }
}
