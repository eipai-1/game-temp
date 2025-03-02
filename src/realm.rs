use std::path::Path;
use std::{collections::HashMap, sync::atomic::AtomicBool};

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

pub const BLOCK_EMPTY: Block = Block {
    tp: BlockType::Empty,
};

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
enum AdjacentState {
    PosX,
    NegX,
    PosZ,
    NegZ,
    Same,
}

#[allow(unused)]
#[derive(Debug, Default, Clone, Copy)]
pub struct BlockInfo {
    pub name: &'static str,
    pub block_type: BlockType,
    pub tex_offset: [[u8; 2]; 6],
}

impl BlockInfo {
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

#[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ChunkCoord {
    x: i32,
    z: i32,
}

impl ChunkCoord {
    fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq)]
pub struct Block {
    pub tp: BlockType,
}

impl Block {
    fn new(tp: BlockType) -> Self {
        Self { tp }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ChunkData {
    blocks: Vec<Block>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk {
    data: ChunkData,
    is_dirty: AtomicBool,
}

impl Chunk {
    fn new(data: ChunkData) -> Self {
        let is_dirty = AtomicBool::new(false);
        Self { data, is_dirty }
    }

    fn get_block(&self, x: usize, y: usize, z: usize) -> Block {
        return self.data.blocks[x * CHUNK_SIZE * CHUNK_HEIGHT + y * CHUNK_SIZE + z];
    }

    fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) {
        self.data.blocks[x * CHUNK_SIZE * CHUNK_HEIGHT + y * CHUNK_SIZE + z] = block;
    }

    fn save(&self, world_dir: &str, coord: &ChunkCoord) -> anyhow::Result<()> {
        let path = Path::new(world_dir)
            .join("chunks")
            .join(format!("x{}", coord.x))
            .join(format!("y{}.chunk", coord.z));

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("创建区块目录失败")?;
        }

        let encoded = bincode::serialize(&self.data).context("区块序列化失败")?;

        std::fs::write(&path, encoded).context("写入区块失败")?;

        Ok(())
    }

    fn load(world_dir: &str, coord: &ChunkCoord) -> anyhow::Result<Option<ChunkData>> {
        let path = Path::new(world_dir)
            .join("chunks")
            .join(format!("x{}", coord.x))
            .join(format!("y{}.chunk", coord.z));

        //区块不存在
        if !path.exists() {
            return Ok(None);
        }

        let data = std::fs::read(&path).context("读取区块文件失败")?;

        let chunk: ChunkData = bincode::deserialize(&data).context("解析区块数据失败")?;

        if chunk.blocks.len() != CHUNK_SIZE * CHUNK_SIZE * CHUNK_HEIGHT {
            anyhow::bail!("区块数据损坏：方块数量不匹配");
        }

        Ok(Some(chunk))
    }
}

pub struct Realm {
    pub data: RealmData,
    pub render_res: RenderResources,
}

pub struct RealmData {
    pub chunk_map: HashMap<ChunkCoord, Chunk>,
    pub all_block: Vec<BlockInfo>,
    pub wf_max_len: f32,
    pub instances: Vec<Vec<Instance>>,

    pub wf_uniform: WireframeUniform,
    pub is_wf_visible: bool,

    pub center_chunk_pos: ChunkCoord,
    chunk_rad: i32,

    pub name: &'static str,
}

impl RealmData {
    pub fn new() -> Self {
        let mut all_block: Vec<BlockInfo> = vec![BlockInfo::default(); BLOCK_NUM];

        let empty = BlockInfo::new(
            "Empty",
            [[0, 0], [0, 0], [0, 0], [0, 0], [0, 0], [0, 0]],
            BlockType::Empty,
        );
        all_block[empty.block_type as usize] = empty;

        let under_stone = BlockInfo::new(
            "Under stone",
            [[1, 0], [1, 0], [1, 0], [1, 0], [1, 0], [1, 0]],
            BlockType::UnderStone,
        );
        all_block[under_stone.block_type as usize] = under_stone;

        let stone = BlockInfo::new(
            "stone",
            [[0, 0], [0, 0], [0, 0], [0, 0], [0, 0], [0, 0]],
            BlockType::Stone,
        );
        all_block[stone.block_type as usize] = stone;

        let dirt = BlockInfo::new(
            "dirt",
            [[4, 0], [4, 0], [4, 0], [4, 0], [4, 0], [4, 0]],
            BlockType::Dirt,
        );
        all_block[dirt.block_type as usize] = dirt;

        //创建草方块
        let grass = BlockInfo::new(
            "grass",
            [[3, 0], [2, 0], [3, 0], [4, 0], [3, 0], [3, 0]],
            BlockType::Grass,
        );
        all_block[grass.block_type as usize] = grass;
        //草方块创建完成

        let mut instances: Vec<Vec<Instance>> = Vec::new();
        for _i in 0..BLOCK_NUM {
            instances.push(Vec::new());
        }

        let wf_uniform = WireframeUniform {
            position: [0.0, 0.0, 0.0],
            _padding: 0.0,
        };

        let wf_max_len: f32 = 8.0;

        let is_wf_visible = true;

        let center_chunk_pos = ChunkCoord { x: 0, z: 0 };

        let name = "./data/worlds/default_name_1";

        let chunk_rad: i32 = 1;
        if chunk_rad < 0 {
            panic!("invaild chunk_rad value:{}", chunk_rad);
        }

        let mut chunk_map: HashMap<ChunkCoord, Chunk> = HashMap::new();
        //init chunk
        Self::load_all_chunk(chunk_rad as i32, center_chunk_pos, &mut chunk_map, name);

        println!("chunk_map:{}", chunk_map.len());

        Self::load_all_instances(&mut instances, &chunk_map);

        Self {
            all_block,
            chunk_map,
            instances,
            wf_uniform,
            wf_max_len,
            is_wf_visible,
            center_chunk_pos,
            name,
            chunk_rad,
        }
    }

    pub fn get_block(&self, mut x: i32, y: i32, mut z: i32) -> Block {
        let coord = get_chunk_coord(x as f32, z as f32);

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
            return BLOCK_EMPTY;
        }

        let chunk = self.chunk_map.get(&coord).unwrap();
        return chunk.get_block(x as usize, y as usize, z as usize);
    }

    pub fn update_wf_uniform(&mut self, new_position: Point3<i32>) {
        self.wf_uniform.position = [
            new_position.x as f32,
            new_position.y as f32,
            new_position.z as f32,
        ];
    }

    //以chunk_pos为中心,chunk_rad为半径加载正方形区块
    //加载全部需要的区块
    fn load_all_chunk(
        chunk_rad: i32,
        chunk_pos: ChunkCoord,
        chunk_map: &mut HashMap<ChunkCoord, Chunk>,
        world_dir: &str,
    ) {
        for relative_x in -chunk_rad..=chunk_rad {
            for relative_z in -chunk_rad..=chunk_rad {
                let x = chunk_pos.x + relative_x;
                let z = chunk_pos.z + relative_z;
                let coord = ChunkCoord::new(x, z);
                Self::load_chunk(&coord, chunk_map, world_dir);
            }
        }
    }

    //加载指定的区块
    fn load_chunk(
        chunk_pos: &ChunkCoord,
        chunk_map: &mut HashMap<ChunkCoord, Chunk>,
        world_dir: &str,
    ) {
        match chunk_map.get(chunk_pos) {
            //存在就不要重复添加
            Some(_) => {}

            None => {
                match Chunk::load(world_dir, chunk_pos) {
                    //存在则读取
                    Ok(Some(data)) => {
                        chunk_map.insert(*chunk_pos, Chunk::new(data));
                    }

                    //不存在生成
                    Ok(None) => {
                        let blocks = vec![Block::default(); CHUNK_SIZE * CHUNK_SIZE * CHUNK_HEIGHT];

                        let chunk_data = ChunkData { blocks };
                        let mut chunk = Chunk::new(chunk_data);

                        for x in 0..CHUNK_SIZE {
                            for y in 0..CHUNK_HEIGHT {
                                for z in 0..CHUNK_SIZE {
                                    if y == 3 {
                                        chunk.set_block(x, y, z, Block::new(BlockType::Grass));
                                    } else if y == 2 {
                                        chunk.set_block(x, y, z, Block::new(BlockType::Dirt));
                                    } else if y == 1 {
                                        chunk.set_block(x, y, z, Block::new(BlockType::Stone));
                                    } else if y == 0 {
                                        chunk.set_block(x, y, z, Block::new(BlockType::UnderStone));
                                    }
                                }
                            }
                        }
                        chunk_map.insert(*chunk_pos, chunk);
                    }
                    //读取错误
                    Err(e) => {
                        eprintln!("区块加载错误：{}", e);
                        //区块有错先崩溃
                        panic!("区块加载错误");
                    }
                };
            }
        }
    }

    fn save_chunk(&self, coord: &ChunkCoord) {
        match self.chunk_map.get(coord).unwrap().save(&self.name, coord) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("区块保存错误:{}", e);
                panic!("区块保存错误");
            }
        }
    }

    pub fn update(&mut self, player_pos: &Point3<f32>) {
        let coord = get_chunk_coord(player_pos.x, player_pos.z);
        match Self::is_adjacent(&self.center_chunk_pos, &coord) {
            //Same Chunk
            0 => {}

            //load new chunk and unload old chunk
            1 => {
                for x in -self.chunk_rad..=self.chunk_rad {
                    self.chunk_map.remove(&ChunkCoord::new(
                        x,
                        self.center_chunk_pos.z + self.chunk_rad,
                    ));
                }
                for x in -self.chunk_rad..=self.chunk_rad {
                    Self::load_chunk(
                        &ChunkCoord::new(x, coord.z - self.chunk_rad),
                        &mut self.chunk_map,
                        &self.name,
                    );
                }
            }
            2 => {
                for x in -self.chunk_rad..=self.chunk_rad {
                    self.chunk_map.remove(&ChunkCoord::new(
                        x,
                        self.center_chunk_pos.z - self.chunk_rad,
                    ));
                }
                for x in -self.chunk_rad..=self.chunk_rad {
                    Self::load_chunk(
                        &ChunkCoord::new(x, coord.z + self.chunk_rad),
                        &mut self.chunk_map,
                        &self.name,
                    );
                }
            }
            3 => {
                for z in -self.chunk_rad..=self.chunk_rad {
                    self.chunk_map.remove(&ChunkCoord::new(
                        &self.center_chunk_pos.x + self.chunk_rad,
                        z,
                    ));
                }
                for z in -self.chunk_rad..=self.chunk_rad {
                    Self::load_chunk(
                        &ChunkCoord::new(&coord.x - self.chunk_rad, z),
                        &mut self.chunk_map,
                        &self.name,
                    );
                }
            }
            4 => {
                for z in -self.chunk_rad..=self.chunk_rad {
                    self.chunk_map.remove(&ChunkCoord::new(
                        &self.center_chunk_pos.x - self.chunk_rad,
                        z,
                    ));
                }
                for z in -self.chunk_rad..=self.chunk_rad {
                    Self::load_chunk(
                        &ChunkCoord::new(&coord.x + self.chunk_rad, z),
                        &mut self.chunk_map,
                        &self.name,
                    );
                }
            }
            _ => {}
        }
    }

    fn is_adjacent(old: &ChunkCoord, new: &ChunkCoord) -> i32 {
        if old.x == new.x {
            if old.z - new.z == 1 {
                return 1;
            } else if new.z - old.z == 1 {
                return 2;
            }
        } else if old.z == new.z {
            if old.x - new.x == 1 {
                return 3;
            } else if new.x - old.x == 1 {
                return 4;
            }
        }
        0
    }

    fn load_all_instances(
        instances: &mut Vec<Vec<Instance>>,
        chunk_map: &HashMap<ChunkCoord, Chunk>,
    ) {
        for (coord, chunk) in chunk_map {
            for (i, block) in chunk.data.blocks.iter().enumerate() {
                let ux = i / (CHUNK_SIZE * CHUNK_HEIGHT);
                let remainder_x = i % (CHUNK_SIZE * CHUNK_HEIGHT);
                let uy = remainder_x / CHUNK_SIZE;
                let uz = remainder_x % CHUNK_SIZE;

                let mut x = ux as i32;
                let y = uy as i32;
                let mut z = uz as i32;

                x += coord.x * CHUNK_SIZE as i32;
                z += coord.z * CHUNK_SIZE as i32;

                instances[block.tp as usize].push(Instance {
                    position: [x as f32, y as f32, z as f32],
                });
            }
        }
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

//x，z是实际坐标
pub fn get_chunk_coord(x: f32, z: f32) -> ChunkCoord {
    let chunk_x = (x / CHUNK_SIZE as f32).floor() as i32;
    let chunk_z = (z / CHUNK_SIZE as f32).floor() as i32;
    ChunkCoord::new(chunk_x, chunk_z)
}

#[cfg(test)]
mod tests {
    //use crate::realm::BlockType;

    use super::RealmData;
    use super::*;

    #[test]
    fn test_get_set_block() {
        //let data = RealmData::new();

        //assert_eq!(data.get_block(0, 0, 0), BlockType::UnderStone);
        //assert_eq!(data.get_block(0, 1, 0), BlockType::Stone);
        //assert_eq!(data.get_block(0, 2, 0), BlockType::Dirt);
        //assert_eq!(data.get_block(0, 3, 0), BlockType::Grass);
        //assert_eq!(data.get_block(0, 0, -1), BlockType::Empty);
    }

    #[test]
    fn test_chunk_file() -> anyhow::Result<()> {
        let data = RealmData::new();
        let coord = ChunkCoord::new(0, 0);
        data.chunk_map
            .get(&coord)
            .unwrap()
            .save(data.name, &coord)?;

        let chunk = Chunk::load(data.name, &coord).unwrap().unwrap();

        assert_eq!(chunk, data.chunk_map.get(&coord).unwrap().data);
        Ok(())
    }
}
