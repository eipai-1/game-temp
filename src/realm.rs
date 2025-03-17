use std::panic;
use std::path::Path;
use std::{collections::HashMap, sync::atomic::AtomicBool};

use anyhow::Context;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use wgpu::{util::DeviceExt, *};

pub const BLOCK_NUM: usize = 5;

use cgmath::*;

pub const TEXT_FRAC: f32 = 16.0 / 512.0;
const WF_SIZE: f32 = 0.01;
const WF_WIDTH: f32 = 0.02;
pub const VERTICES: &[Vertex] = &[
    //方块坐标：其中每条边都从原点向每个轴的正方向延伸一格
    //按照正-上-后-下-左-右的顺序

    //正面---正常从正面看
    //后面统一按以下顺序
    //正面左上角
    Vertex {
        position: [0.0, 1.0, 1.0],
        tex_coord: [0.0, 0.0],
    },
    //正面右上角
    Vertex {
        position: [1.0, 1.0, 1.0],
        tex_coord: [1.0, 0.0],
    },
    //正面右下角
    Vertex {
        position: [1.0, 0.0, 1.0],
        tex_coord: [1.0, 1.0],
    },
    //正面左下角
    Vertex {
        position: [0.0, 0.0, 1.0],
        tex_coord: [0.0, 1.0],
    },
    //上面---从上面看---摄像机上方向为z轴负方向
    Vertex {
        position: [0.0, 1.0, 0.0],
        tex_coord: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        tex_coord: [1.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, 1.0],
        tex_coord: [1.0, 1.0],
    },
    Vertex {
        position: [0.0, 1.0, 1.0],
        tex_coord: [0.0, 1.0],
    },
    //后面---摄像机上方向为y轴正方向
    Vertex {
        position: [1.0, 1.0, 0.0],
        tex_coord: [0.0, 0.0],
    },
    Vertex {
        position: [0.0, 1.0, 0.0],
        tex_coord: [1.0, 0.0],
    },
    Vertex {
        position: [0.0, 0.0, 0.0],
        tex_coord: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, 0.0, 0.0],
        tex_coord: [0.0, 1.0],
    },
    //下面--摄像机正方向为y轴正方向
    Vertex {
        position: [0.0, 0.0, 1.0],
        tex_coord: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, 0.0, 1.0],
        tex_coord: [1.0, 0.0],
    },
    Vertex {
        position: [1.0, 0.0, 0.0],
        tex_coord: [1.0, 1.0],
    },
    Vertex {
        position: [0.0, 0.0, 0.0],
        tex_coord: [0.0, 1.0],
    },
    //左面--摄像机上方向y轴正向
    Vertex {
        position: [0.0, 1.0, 0.0],
        tex_coord: [0.0, 0.0],
    },
    Vertex {
        position: [0.0, 1.0, 1.0],
        tex_coord: [1.0, 0.0],
    },
    Vertex {
        position: [0.0, 0.0, 1.0],
        tex_coord: [1.0, 1.0],
    },
    Vertex {
        position: [0.0, 0.0, 0.0],
        tex_coord: [0.0, 1.0],
    },
    //右面--摄像机上方向y轴正向
    Vertex {
        position: [1.0, 1.0, 1.0],
        tex_coord: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        tex_coord: [1.0, 0.0],
    },
    Vertex {
        position: [1.0, 0.0, 0.0],
        tex_coord: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, 0.0, 1.0],
        tex_coord: [0.0, 1.0],
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

const CHUNK_SIZE: i32 = 2;
const CHUNK_HEIGHT: i32 = 5;
const BLOCK_NUM_PER_CHUNK: usize = (CHUNK_SIZE * CHUNK_SIZE * CHUNK_HEIGHT) as usize;

const WORLD_FILE_DIR: &str = "./worlds";

pub const BLOCK_EMPTY: Block = Block {
    tp: BlockType::Empty,
};

//同时作为usize和u32
//注意范围
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

pub const BLOCK_MATERIALS_NUM: u32 = 5;

#[repr(u32)]
#[derive(FromPrimitive)]
pub enum BlockMaterials {
    Empty = 999,
    UnderStone = 0,
    Stone = 1,
    GrassBlockSide = 2,
    GrassBlockTop = 3,
    Dirt = 4,
}

#[allow(unused)]
#[derive(Debug, Default, Clone, Copy)]
pub struct BlockInfo {
    pub name: &'static str,
    pub block_type: BlockType,
    pub tex_offset: [u32; 6],
}

impl BlockInfo {
    fn new(
        name: &'static str,
        //顺序为：正、上、后、下、左、右
        tex_offset: [u32; 6],
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
    pub tex_coord: [f32; 2],
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
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub position: [f32; 3],
    pub block_type: u32,
}

impl Instance {
    pub fn desc() -> VertexBufferLayout<'static> {
        use std::mem;
        VertexBufferLayout {
            array_stride: mem::size_of::<Instance>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 6,
                    format: VertexFormat::Uint32,
                },
            ],
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

/*
 * 这是存储到文件中的数据
 */
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ChunkData {
    blocks: Vec<Block>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Material {
    index: u32,
    _padding: [u32; 3],
}

impl Material {
    fn new(index: u32) -> Self {
        Self {
            index,
            _padding: [0, 0, 0],
        }
    }
}

/*
 * 这是游戏运行时需要的数据
 */
#[derive(Debug)]
pub struct Chunk {
    data: ChunkData,
    is_dirty: AtomicBool,
}

impl Chunk {
    fn new(data: ChunkData) -> Self {
        let is_dirty = AtomicBool::new(false);
        Self { data, is_dirty }
    }

    fn get_block(&self, x: i32, y: i32, z: i32) -> Block {
        return self.data.blocks[(x * CHUNK_SIZE * CHUNK_HEIGHT + y * CHUNK_SIZE + z) as usize];
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: Block) {
        self.data.blocks[(x * CHUNK_SIZE * CHUNK_HEIGHT + y * CHUNK_SIZE + z) as usize] = block;
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

        if chunk.blocks.len() != BLOCK_NUM_PER_CHUNK {
            anyhow::bail!("区块数据损坏：方块数量不匹配");
        }

        Ok(Some(chunk))
    }
    //i为blocks的索引

    pub fn get_instance(&self, coord: &ChunkCoord) {
        let mut instance = Vec::with_capacity(BLOCK_NUM_PER_CHUNK);
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_HEIGHT {
                for z in 0..CHUNK_SIZE {
                    let block = self.get_block(x, y, z);
                    instance.push(Instance {
                        position: [
                            (x + coord.x * CHUNK_SIZE) as f32,
                            y as f32,
                            (z + coord.z * CHUNK_SIZE) as f32,
                        ],
                        block_type: block.tp as u32,
                    });
                }
            }
        }
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
    pub instance: Vec<Instance>,

    pub wf_uniform: WireframeUniform,
    pub is_wf_visible: bool,

    pub center_chunk_pos: ChunkCoord,
    chunk_rad: i32,

    pub name: &'static str,
}

impl RealmData {
    pub fn new() -> Self {
        use BlockMaterials::*;
        let mut all_block: Vec<BlockInfo> = vec![BlockInfo::default(); BLOCK_NUM];

        let empty = BlockInfo::new(
            "Empty",
            [
                Empty as u32,
                Empty as u32,
                Empty as u32,
                Empty as u32,
                Empty as u32,
                Empty as u32,
            ],
            BlockType::Empty,
        );
        all_block[empty.block_type as usize] = empty;

        let under_stone = BlockInfo::new(
            "bedrock",
            [
                UnderStone as u32,
                UnderStone as u32,
                UnderStone as u32,
                UnderStone as u32,
                UnderStone as u32,
                UnderStone as u32,
            ],
            BlockType::UnderStone,
        );
        all_block[under_stone.block_type as usize] = under_stone;

        let stone = BlockInfo::new(
            "stone",
            [
                Stone as u32,
                Stone as u32,
                Stone as u32,
                Stone as u32,
                Stone as u32,
                Stone as u32,
            ],
            BlockType::Stone,
        );
        all_block[stone.block_type as usize] = stone;

        let dirt = BlockInfo::new(
            "dirt",
            [
                Dirt as u32,
                Dirt as u32,
                Dirt as u32,
                Dirt as u32,
                Dirt as u32,
                Dirt as u32,
            ],
            BlockType::Dirt,
        );
        all_block[dirt.block_type as usize] = dirt;

        //创建草方块
        let grass = BlockInfo::new(
            "grass_block_side",
            [
                GrassBlockSide as u32,
                GrassBlockTop as u32,
                GrassBlockSide as u32,
                Dirt as u32,
                GrassBlockSide as u32,
                GrassBlockSide as u32,
            ],
            BlockType::Grass,
        );
        all_block[grass.block_type as usize] = grass;
        //草方块创建完成

        let wf_uniform = WireframeUniform {
            position: [0.0, 0.0, 0.0],
            _padding: 0.0,
        };

        let wf_max_len: f32 = 12.0;

        let is_wf_visible = true;

        let center_chunk_pos = ChunkCoord { x: 0, z: 0 };

        let name = "./data/worlds/default_name_1";

        let chunk_rad: i32 = 1;
        if chunk_rad < 0 {
            panic!("invaild chunk_rad value:{}", chunk_rad);
        }

        let mut instance: Vec<Instance> =
            Vec::with_capacity((chunk_rad * 2 + 1).pow(2) as usize * BLOCK_NUM_PER_CHUNK);

        let mut chunk_map: HashMap<ChunkCoord, Chunk> = HashMap::new();
        //init chunk
        Self::load_all_chunk(chunk_rad as i32, center_chunk_pos, &mut chunk_map, name);

        println!("chunk_map.len():{}", chunk_map.len());
        //Self::debug_print_chunk_map(&chunk_map);

        Self::init_instance(&mut instance, &chunk_map, chunk_rad, center_chunk_pos);

        Self {
            all_block,
            chunk_map,
            instance,
            wf_uniform,
            wf_max_len,
            is_wf_visible,
            center_chunk_pos,
            name,
            chunk_rad,
        }
    }

    pub fn get_block(&self, coord: Point3<i32>) -> Block {
        let mut x = coord.x;
        let y = coord.y;
        let mut z = coord.z;
        let coord = get_chunk_coord(x, z);

        //不在区块内！返回空
        if !self.chunk_map.contains_key(&coord) {
            return BLOCK_EMPTY;
        }

        let i32_size = CHUNK_SIZE as i32;
        x = x.rem_euclid(i32_size);
        z = z.rem_euclid(i32_size);

        //println!("chunk_x:{}, chunk_z:{} ", coord.x, coord.z);
        //println!("x={}, y={}, z={}", x, y, z);

        //对纵坐标位于区块外的方块，统一检测为虚空
        //区块范围为[0, CHUNK_HEIGHT]
        if y < 0 || y >= CHUNK_HEIGHT as i32 {
            return BLOCK_EMPTY;
        }

        let chunk = self.chunk_map.get(&coord).unwrap();
        return chunk.get_block(x, y, z);
    }

    pub fn update_wf_uniform(&mut self, new_position: Point3<i32>) {
        self.wf_uniform.position = [
            new_position.x as f32,
            new_position.y as f32,
            new_position.z as f32,
        ];
    }

    //以center_chunk_pos为中心,chunk_rad为半径加载正方形区块
    //加载全部需要的区块
    fn load_all_chunk(
        chunk_rad: i32,
        center_chunk_pos: ChunkCoord,
        chunk_map: &mut HashMap<ChunkCoord, Chunk>,
        world_dir: &str,
    ) {
        for relative_x in -chunk_rad..=chunk_rad {
            for relative_z in -chunk_rad..=chunk_rad {
                let x = center_chunk_pos.x + relative_x;
                let z = center_chunk_pos.z + relative_z;
                let coord = ChunkCoord::new(x, z);
                Realm::init_chunk(&coord, chunk_map, world_dir);
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

    //#[allow(unused)]
    pub fn debug_print_chunk_map(chunk_map: &HashMap<ChunkCoord, Chunk>) {
        for (coord, chunk) in chunk_map {
            println!("Chunk at ({}, {})", coord.x, coord.z);
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_HEIGHT {
                    for z in 0..CHUNK_SIZE {
                        println!(
                            "{},{},{}:{:?}",
                            x as i32 + coord.x * CHUNK_SIZE as i32,
                            y as i32,
                            z as i32 + coord.z * CHUNK_SIZE as i32,
                            chunk.get_block(x, y, z).tp
                        );
                    }
                }
            }
        }
    }

    //根据center_chunk_pos初始化全部实例
    fn init_instance(
        instance: &mut Vec<Instance>,
        chunk_map: &HashMap<ChunkCoord, Chunk>,
        chunk_rad: i32,
        center_chunk_pos: ChunkCoord,
    ) {
        for dx in -chunk_rad..=chunk_rad {
            for dz in -chunk_rad..=chunk_rad {
                let x = dx + center_chunk_pos.x;
                let z = dz + center_chunk_pos.z;
                let chunk = chunk_map.get(&ChunkCoord { x, z }).unwrap();
                let chunk_base_x = x * CHUNK_SIZE as i32;
                let chunk_base_z = z * CHUNK_SIZE as i32;

                // 按照x -> y -> z的顺序访问块
                for x in 0..CHUNK_SIZE {
                    for y in 0..CHUNK_HEIGHT {
                        for z in 0..CHUNK_SIZE {
                            let block = chunk.get_block(x, y, z);
                            //if block.tp != BlockType::Empty {
                            instance.push(Instance {
                                position: [
                                    (chunk_base_x + x as i32) as f32,
                                    y as f32,
                                    (chunk_base_z + z as i32) as f32,
                                ],
                                block_type: block.tp as u32,
                            });
                            //}
                        }
                    }
                }
            }
        }
    }
}

pub struct RenderResources {
    //pub wf_vertices: Vec<WireframeVertex>,
    pub wf_vertex_buffer: Buffer,
    pub wf_index_buffer: Buffer,
    pub instance_buffer: Buffer,
    pub block_vertex_buffer: Buffer,
    pub block_index_buffer: Buffer,
    pub wf_uniform_buffer: Buffer,
    pub block_materials_buffer: Buffer,
    pub block_materials_bind_group: BindGroup,
    pub block_materials_bind_group_layout: BindGroupLayout,
}

impl RenderResources {
    fn new(device: &Device, data: &RealmData) -> Self {
        let block_vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("block veretx buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });

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

        let instance_buffer = Self::init_instance_buffer(device, data);

        let block_index_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("block index buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: BufferUsages::INDEX,
        });

        let mut block_materials: Vec<Material> = vec![Material::new(0); BLOCK_NUM * 6];
        for (i, block) in data.all_block.iter().enumerate() {
            for face in 0..6 {
                block_materials[i * 6 + face] = Material::new(block.tex_offset[face]);
            }
        }

        let block_materials_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("block materials buffer"),
            contents: bytemuck::cast_slice(&block_materials),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let block_materials_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Block materials bind group layout"),
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

        let block_materials_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("block materials bind group"),
            layout: &block_materials_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: block_materials_buffer.as_entire_binding(),
            }],
        });

        Self {
            block_vertex_buffer,
            block_index_buffer,
            wf_index_buffer,
            wf_vertex_buffer,
            wf_uniform_buffer,
            instance_buffer,
            block_materials_buffer,
            block_materials_bind_group,
            block_materials_bind_group_layout,
            //wf_vertices,
        }
    }

    pub fn init_instance_buffer(device: &Device, data: &RealmData) -> Buffer {
        device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Block buffer"),
            contents: bytemuck::cast_slice(&data.instance),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        })
    }
}

impl Realm {
    pub fn new(device: &Device) -> Self {
        let data = RealmData::new();
        let render_res = RenderResources::new(device, &data);

        Self { data, render_res }
    }

    //区块初始化
    fn init_chunk(
        chunk_pos: &ChunkCoord,
        chunk_map: &mut HashMap<ChunkCoord, Chunk>,
        world_dir: &str,
    ) -> Vec<Instance> {
        let mut instance: Vec<Instance> = Vec::with_capacity(BLOCK_NUM_PER_CHUNK);

        if instance.capacity() != BLOCK_NUM_PER_CHUNK {
            panic!("instance capacity error");
        }

        match chunk_map.get(chunk_pos) {
            //存在就不要重复添加
            Some(_) => {}

            None => {
                match Chunk::load(world_dir, chunk_pos) {
                    //存在则读取
                    Ok(Some(data)) => {
                        chunk_map.insert(*chunk_pos, Chunk::new(data));
                    }

                    //不存在则生成
                    Ok(None) => {
                        let blocks = vec![
                            Block::default();
                            (CHUNK_SIZE * CHUNK_SIZE * CHUNK_HEIGHT) as usize
                        ];

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
                                    instance.push(Instance {
                                        position: [
                                            (x + chunk_pos.x * CHUNK_SIZE) as f32,
                                            y as f32,
                                            (z + chunk_pos.z * CHUNK_SIZE) as f32,
                                        ],
                                        block_type: chunk.get_block(x, y, z).tp as u32,
                                    });
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
        instance
    }

    fn load_chunk(
        &mut self,
        new_chunk_pos: &ChunkCoord,
        old_chunk_pos: &ChunkCoord,
        queue: &Queue,
    ) {
        let instance = Self::init_chunk(new_chunk_pos, &mut self.data.chunk_map, self.data.name);
        queue.write_buffer(
            &self.render_res.instance_buffer,
            self.get_offset(old_chunk_pos, &Point3::new(0, 0, 0)),
            bytemuck::cast_slice(&instance),
        );
    }

    fn unload_chunk(&mut self, chunk_pos: &ChunkCoord) {
        self.data.chunk_map.remove(chunk_pos);
    }

    //记得更新center_chunk_pos
    pub fn update(&mut self, player_pos: &Point3<f32>, device: &Device, queue: &Queue) {
        let new_coord = get_chunk_coord(player_pos.x as i32, player_pos.z as i32);
        let dx = new_coord.x - self.data.center_chunk_pos.x;
        let dz = new_coord.z - self.data.center_chunk_pos.z;

        if dx == 0 && dz == 0 {
            return;
        }

        if dx >= -1 && dx <= 1 && dz >= -1 && dz <= 1 {
            self.update_helper(dx, dz, queue);
        } else {
            self.reload_all_chunk(&new_coord, device);
        }
        //记得更新中心位置
        self.data.center_chunk_pos = new_coord;
    }

    //两个offset都只有-1, 0, 1三个值
    //此时的区块中心还没更新
    fn update_helper(&mut self, x_offset: i32, z_offset: i32, queue: &Queue) {
        if z_offset != 0 {
            for x in -self.data.chunk_rad..=self.data.chunk_rad {
                let old_chunk_pos = ChunkCoord::new(
                    self.data.center_chunk_pos.x + x,
                    self.data.center_chunk_pos.z - self.data.chunk_rad * z_offset,
                );
                let new_chunk_pos = ChunkCoord::new(
                    self.data.center_chunk_pos.x + x,
                    self.data.center_chunk_pos.z + (self.data.chunk_rad + 1) * z_offset,
                );
                self.unload_chunk(&old_chunk_pos);
                self.load_chunk(&new_chunk_pos, &old_chunk_pos, queue);
            }
        }
        if x_offset != 0 {
            for z in -self.data.chunk_rad..=self.data.chunk_rad {
                let old_chunk_pos = ChunkCoord::new(
                    self.data.center_chunk_pos.x - self.data.chunk_rad * x_offset,
                    self.data.center_chunk_pos.z + z,
                );
                let new_chunk_pos = ChunkCoord::new(
                    self.data.center_chunk_pos.x + (self.data.chunk_rad + 1) * x_offset,
                    self.data.center_chunk_pos.z + z,
                );
                self.unload_chunk(&old_chunk_pos);
                self.load_chunk(&new_chunk_pos, &old_chunk_pos, queue);
            }
        }
    }

    fn reload_all_chunk(&mut self, new_coord: &ChunkCoord, device: &Device) {
        RealmData::load_all_chunk(
            self.data.chunk_rad,
            *new_coord,
            &mut self.data.chunk_map,
            self.data.name,
        );
        RealmData::init_instance(
            &mut self.data.instance,
            &self.data.chunk_map,
            self.data.chunk_rad,
            *new_coord,
        );

        self.render_res.instance_buffer = RenderResources::init_instance_buffer(device, &self.data);
    }

    //返回位置是否合法
    fn set_single_block(&mut self, coord: Point3<i32>, block: Block) -> bool {
        let chunk_coord = get_chunk_coord(coord.x, coord.z);

        // 检查区块是否存在
        if let Some(chunk) = self.data.chunk_map.get_mut(&chunk_coord) {
            let local_coord = get_local_coord(coord);

            // 检查 y 坐标是否在有效范围内
            if coord.y >= 0 && coord.y < CHUNK_HEIGHT as i32 {
                chunk.set_block(local_coord.x, local_coord.y, local_coord.z, block);

                // 标记区块为需要更新状态
                chunk
                    .is_dirty
                    .store(true, std::sync::atomic::Ordering::Relaxed);
                return true;
            }
        }
        false
    }

    pub fn place_block(&mut self, block_coord: Point3<i32>, block: Block, queue: &Queue) {
        if self.set_single_block(block_coord, block) {
            let chunk_coord = get_chunk_coord(block_coord.x, block_coord.z);
            let block_offset = get_local_coord(block_coord);
            queue.write_buffer(
                &self.render_res.instance_buffer,
                self.get_offset(&chunk_coord, &block_offset),
                bytemuck::bytes_of(&Instance {
                    position: [
                        block_coord.x as f32,
                        block_coord.y as f32,
                        block_coord.z as f32,
                    ],
                    block_type: block.tp as u32,
                }),
            );
        }
    }

    //block_offset是区块内的方块偏移量
    fn get_offset(&self, chunk_coord: &ChunkCoord, block_offset: &Point3<i32>) -> u64 {
        let size = CHUNK_SIZE as i32;
        let height = CHUNK_HEIGHT as i32;
        let instances_per_chunk = size * size * height;
        let chunk_x_offset = chunk_coord.x - (self.data.center_chunk_pos.x - self.data.chunk_rad);
        let chunk_z_offset = chunk_coord.z - (self.data.center_chunk_pos.z - self.data.chunk_rad);
        //区块偏移量 = x轴 + y轴 * 区块边长
        //总区块偏移量 = 区块偏移量 * 区块实例数
        let chunk_base_offset =
            (chunk_x_offset * (self.data.chunk_rad * 2 + 1) + chunk_z_offset) * instances_per_chunk;

        let block_offset =
            block_offset.x * (size * height) + block_offset.y * size + block_offset.z;

        //总方块数 = 区块偏移量 + 区块内偏移量
        //总偏移量 = 总方块数 * 实例大小
        ((chunk_base_offset + block_offset) as u64) * std::mem::size_of::<Instance>() as u64
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
pub fn get_chunk_coord(x: i32, z: i32) -> ChunkCoord {
    //floor 向下取整
    let chunk_x = (x as f32 / CHUNK_SIZE as f32).floor() as i32;
    let chunk_z = (z as f32 / CHUNK_SIZE as f32).floor() as i32;
    ChunkCoord::new(chunk_x, chunk_z)
}

pub fn get_local_coord(coord: Point3<i32>) -> Point3<i32> {
    let chunk_size = CHUNK_SIZE as i32;
    let local_x = coord.x.rem_euclid(chunk_size);
    let local_z = coord.z.rem_euclid(chunk_size);
    Point3::new(local_x, coord.y, local_z)
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

    #[test]
    fn test_get_block() {
        let data = RealmData::new();
        assert_eq!(
            data.get_block(Point3::new(-3, 0, -3)).tp,
            BlockType::UnderStone
        );
    }
}
