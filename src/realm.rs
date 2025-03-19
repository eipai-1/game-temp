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
const WF_WIDTH: f32 = 0.04;
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

const CHUNK_SIZE: i32 = 16;
const CHUNK_HEIGHT: i32 = 256;
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
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
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

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
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

impl ChunkData {
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Block {
        self.blocks[(x * CHUNK_SIZE * CHUNK_HEIGHT + y * CHUNK_SIZE + z) as usize]
    }

    //仅为创建可见方块创建实例
    pub fn create_instance(
        &self,
        chunk_coord: &ChunkCoord,
        coord_to_offset: &mut Vec<u64>,
        available_offset: &mut u64,
    ) -> Vec<Instance> {
        let mut instance = vec![Instance::default(); BLOCK_NUM_PER_CHUNK];
        let mut index = 0u64;
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_HEIGHT {
                for z in 0..CHUNK_SIZE {
                    let block = self.get_block(x, y, z);
                    if block.tp != BlockType::Empty {
                        if Self::has_any_visible_face(self, x, y, z) {
                            instance[index as usize] = Instance {
                                position: Self::relative_to_absolute(chunk_coord, x, y, z),
                                block_type: block.tp as u32,
                            };
                            coord_to_offset[Self::relative_to_index(x, y, z)] = index;
                            index += 1;
                        }
                    }
                }
            }
        }
        *available_offset = index;
        instance
    }

    fn relative_to_absolute(chunk_coord: &ChunkCoord, x: i32, y: i32, z: i32) -> [f32; 3] {
        [
            (chunk_coord.x * CHUNK_SIZE + x) as f32,
            y as f32,
            (chunk_coord.z * CHUNK_SIZE + z) as f32,
        ]
    }

    pub fn relative_to_index(x: i32, y: i32, z: i32) -> usize {
        (x * CHUNK_SIZE * CHUNK_HEIGHT + y * CHUNK_SIZE + z) as usize
    }

    // 检查方块是否有至少一个可见面
    fn has_any_visible_face(&self, x: i32, y: i32, z: i32) -> bool {
        // 检查六个相邻位置
        [
            (x + 1, y, z),
            (x - 1, y, z),
            (x, y + 1, z),
            (x, y - 1, z),
            (x, y, z + 1),
            (x, y, z - 1),
        ]
        .iter()
        //any()遍历检查是否有ture，否则返回false
        .any(|(nx, ny, nz)| {
            // 边界检查
            if *nx < 0
                || *nx >= CHUNK_SIZE
                || *ny < 0
                || *ny >= CHUNK_HEIGHT
                || *nz < 0
                || *nz >= CHUNK_SIZE
            {
                return false; // 区块边界，不可见
            }

            // 相邻方块为空，则面可见
            self.get_block(*nx, *ny, *nz).tp == BlockType::Empty
        })
    }
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
    coord_to_offset: Vec<u64>,

    instance: Vec<Instance>,
    is_dirty: AtomicBool,

    //为第一个空位置
    pub offset_top: u64,
}

impl Chunk {
    fn new(data: ChunkData) -> Self {
        let is_dirty = AtomicBool::new(false);
        let coord_to_offset = vec![u64::MAX; BLOCK_NUM_PER_CHUNK];
        let instance: Vec<Instance> = Vec::new();
        let offset_top = 0;
        Self {
            data,
            is_dirty,
            instance,
            coord_to_offset,
            offset_top,
        }
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

    pub fn init_instance(&mut self, chunk_coord: &ChunkCoord) {
        self.instance =
            self.data
                .create_instance(chunk_coord, &mut self.coord_to_offset, &mut self.offset_top);
    }
}

pub struct RealmData {
    pub chunk_map: HashMap<ChunkCoord, Chunk>,
    pub all_block: Vec<BlockInfo>,
    pub wf_max_len: f32,
    pub wf_uniform: WireframeUniform,
    pub is_wf_visible: bool,

    pub center_chunk_pos: ChunkCoord,
    chunk_rad: i32,

    pub name: &'static str,

    pub seed: u32,
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

        let wf_max_len: f32 = 6.0;

        let is_wf_visible = true;

        let center_chunk_pos = ChunkCoord { x: 0, z: 0 };

        let name = "./data/worlds/default_name_1";

        let chunk_rad: i32 = 1;
        if chunk_rad < 0 {
            panic!("invaild chunk_rad value:{}", chunk_rad);
        }

        let mut chunk_map: HashMap<ChunkCoord, Chunk> = HashMap::new();

        let seed = 2025318;
        //init chunk
        Self::load_all_chunk(
            chunk_rad as i32,
            center_chunk_pos,
            &mut chunk_map,
            name,
            seed,
        );

        println!("chunk_map.len():{}", chunk_map.len());
        Self::debug_print(&chunk_map.get(&center_chunk_pos).unwrap());

        Self {
            all_block,
            chunk_map,
            wf_uniform,
            wf_max_len,
            is_wf_visible,
            center_chunk_pos,
            name,
            chunk_rad,
            seed,
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
        seed: u32,
    ) {
        for relative_x in -chunk_rad..=chunk_rad {
            for relative_z in -chunk_rad..=chunk_rad {
                let x = center_chunk_pos.x + relative_x;
                let z = center_chunk_pos.z + relative_z;
                let coord = ChunkCoord::new(x, z);
                Realm::init_chunk(&coord, chunk_map, world_dir, seed);
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
    pub fn debug_print(chunk: &Chunk) {
        for x in 0..chunk.offset_top {
            println!("instance:{:?}", chunk.instance[x as usize]);
        }
    }
}

pub struct RenderResources {
    //pub wf_vertices: Vec<WireframeVertex>,
    pub wf_vertex_buffer: Buffer,
    pub wf_index_buffer: Buffer,
    pub instance_buffers: HashMap<ChunkCoord, Buffer>,
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

        let instance_buffers = Self::init_instance_buffers(device, data);

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
            instance_buffers,
            block_materials_buffer,
            block_materials_bind_group,
            block_materials_bind_group_layout,
            //wf_vertices,
        }
    }

    pub fn init_instance_buffers(device: &Device, data: &RealmData) -> HashMap<ChunkCoord, Buffer> {
        let mut instance_buffers = HashMap::new();
        for (coord, chunk) in &data.chunk_map {
            instance_buffers.insert(
                *coord,
                device.create_buffer_init(&util::BufferInitDescriptor {
                    label: Some("Block buffer"),
                    contents: bytemuck::cast_slice(&chunk.instance),
                    usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                }),
            );
        }
        instance_buffers
    }

    pub fn insert_instance_buffer(
        &mut self,
        device: &Device,
        coord: &ChunkCoord,
        instance: &[Instance],
    ) {
        self.instance_buffers.insert(
            *coord,
            device.create_buffer_init(&util::BufferInitDescriptor {
                label: Some("Block buffer"),
                contents: bytemuck::cast_slice(&instance),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            }),
        );
    }
}

pub struct Realm {
    pub data: RealmData,
    pub render_res: RenderResources,
}
impl Realm {
    pub fn new(device: &Device) -> Self {
        let data = RealmData::new();
        let render_res = RenderResources::new(device, &data);

        Self { data, render_res }
    }

    //生成地形后直接添加到chunk_map中
    fn generate_terrian(
        chunk_map: &mut HashMap<ChunkCoord, Chunk>,
        chunk_coord: &ChunkCoord,
        seed: u32,
    ) {
        use noise::{NoiseFn, Perlin};

        let perlin = Perlin::new(seed);

        let blocks = vec![BLOCK_EMPTY; BLOCK_NUM_PER_CHUNK];
        let mut chunk = Chunk::new(ChunkData { blocks });

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let absolute_x = x + chunk_coord.x * CHUNK_SIZE;
                let absolute_z = z + chunk_coord.z * CHUNK_SIZE;
                //get返回值为[-1, 1]
                let height = (perlin.get([absolute_x as f64 / 16.0, absolute_z as f64 / 16.0])
                    * 8.0) as i32
                    + 32;
                for y in 0..height {
                    let block = if y == height - 1 {
                        Block::new(BlockType::Grass)
                    } else if y > height - 5 {
                        Block::new(BlockType::Dirt)
                    } else {
                        Block::new(BlockType::Stone)
                    };
                    chunk.set_block(x, y, z, block);
                }
                chunk.set_block(x, 0, z, Block::new(BlockType::UnderStone));
            }
        }

        chunk.init_instance(chunk_coord);

        chunk_map.insert(*chunk_coord, chunk);
    }

    pub fn get_first_none_empty_block(&self, x: f32, z: f32) -> i32 {
        let mut pos = Point3::new(x as i32, 0, z as i32);

        for y in 0..CHUNK_HEIGHT {
            if self.data.get_block(pos) == BLOCK_EMPTY {
                return y + 1;
            }
            pos.y += 1;
        }
        return 90;
    }

    //区块初始化
    fn init_chunk(
        chunk_pos: &ChunkCoord,
        chunk_map: &mut HashMap<ChunkCoord, Chunk>,
        world_dir: &str,
        seed: u32,
    ) {
        match chunk_map.get(chunk_pos) {
            //存在就不要重复添加
            Some(_) => {}

            None => {
                match Chunk::load(world_dir, chunk_pos) {
                    //存在则读取
                    Ok(Some(data)) => {
                        let mut chunk = Chunk::new(data);
                        chunk.init_instance(chunk_pos);
                        chunk_map.insert(*chunk_pos, chunk);
                    }

                    //不存在则生成
                    Ok(None) => {
                        Self::generate_terrian(chunk_map, chunk_pos, seed);
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

    fn load_chunk(&mut self, device: &Device, new_chunk_pos: &ChunkCoord, queue: &Queue) {
        Self::init_chunk(
            new_chunk_pos,
            &mut self.data.chunk_map,
            self.data.name,
            self.data.seed,
        );

        self.render_res.insert_instance_buffer(
            device,
            new_chunk_pos,
            &self.data.chunk_map[new_chunk_pos].instance,
        );
    }

    fn unload_chunk(&mut self, chunk_pos: &ChunkCoord) {
        self.data.chunk_map.remove(chunk_pos);
    }

    pub fn update(&mut self, player_pos: &Point3<f32>, device: &Device, queue: &Queue) {
        let new_coord = get_chunk_coord(player_pos.x as i32, player_pos.z as i32);
        let dx = new_coord.x - self.data.center_chunk_pos.x;
        let dz = new_coord.z - self.data.center_chunk_pos.z;

        if dx == 0 && dz == 0 {
            return;
        }
        println!("chunk updated");
        if dx >= -1 && dx <= 1 && dz >= -1 && dz <= 1 {
            self.update_helper(dx, dz, device, queue);
        } else {
            self.reload_all_chunk(&new_coord, device);
        }
        //记得更新中心位置
        self.data.center_chunk_pos = new_coord;
    }

    //两个offset都只有-1, 0, 1三个值
    //此时的区块中心还没更新
    fn update_helper(&mut self, x_offset: i32, z_offset: i32, device: &Device, queue: &Queue) {
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
                //卸载和加载顺序不能反
                self.load_chunk(device, &new_chunk_pos, queue);
                self.unload_chunk(&old_chunk_pos);
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
                self.load_chunk(device, &new_chunk_pos, queue);
                self.unload_chunk(&old_chunk_pos);
            }
        }
    }

    fn reload_all_chunk(&mut self, new_coord: &ChunkCoord, device: &Device) {
        RealmData::load_all_chunk(
            self.data.chunk_rad,
            *new_coord,
            &mut self.data.chunk_map,
            self.data.name,
            self.data.seed,
        );
        self.render_res.instance_buffers =
            RenderResources::init_instance_buffers(device, &self.data);
    }

    //返回位置是否合法
    fn set_block_data(&mut self, coord: Point3<i32>, block: Block) -> bool {
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

    fn update_adjacent_block(&mut self, coord: Point3<i32>, queue: &Queue) {
        //更新相邻方块
        let chunk_coord = get_chunk_coord(coord.x, coord.z);
        let local_coord = get_local_coord(coord);
        [
            (local_coord.x + 1, local_coord.y, local_coord.z),
            (local_coord.x - 1, local_coord.y, local_coord.z),
            (local_coord.x, local_coord.y + 1, local_coord.z),
            (local_coord.x, local_coord.y - 1, local_coord.z),
            (local_coord.x, local_coord.y, local_coord.z + 1),
            (local_coord.x, local_coord.y, local_coord.z - 1),
        ]
        .iter()
        .for_each(|(x, y, z)| {
            //如果不再缓冲区内，则说明不可见
            //又 如果为可见方块，则需要令其可见
            //故需要将其插入缓冲区
            if self.data.chunk_map[&chunk_coord].coord_to_offset
                [ChunkData::relative_to_index(*x, *y, *z)]
                == u64::MAX
                && self.data.get_block(Point3::new(*x, *y, *z)) != BLOCK_EMPTY
            {
                self.update_single_block_buffer(Point3::new(*x, *y, *z), queue);
            }
        });
    }

    pub fn place_block(&mut self, block_coord: Point3<i32>, block: Block, queue: &Queue) {
        if self.set_block_data(block_coord, block) {
            let chunk_coord = get_chunk_coord(block_coord.x, block_coord.z);
            let block_offset = get_local_coord(block_coord);

            //offset为选中方块的偏移量
            let mut offset = self.data.chunk_map[&chunk_coord].coord_to_offset
                [ChunkData::relative_to_index(block_offset.x, block_offset.y, block_offset.z)];

            let mut instance = Instance {
                position: [
                    block_coord.x as f32,
                    block_coord.y as f32,
                    block_coord.z as f32,
                ],
                block_type: block.tp as u32,
            };

            /* 如果是空方块则为摧毁方块操作
             * 则需要更新相邻方块
             * 且偏移量一定有效，这是在camera摧毁方块操作保证了的！
             * 同时用缓冲区顶部的数据覆盖当前数据
             * 同时还需要更新offset_top和coord_to_offset
             */
            if block.tp == BlockType::Empty {
                self.update_adjacent_block(block_coord, queue);
                instance = self.data.chunk_map[&chunk_coord].instance
                    [self.data.chunk_map[&chunk_coord].offset_top as usize - 1];

                let chunk = self.data.chunk_map.get_mut(&chunk_coord).unwrap();

                //用数组末尾的值覆盖要删除的值
                chunk.instance[offset as usize] = instance;
                chunk.coord_to_offset[ChunkData::relative_to_index(
                    instance.position[0] as i32,
                    instance.position[1] as i32,
                    instance.position[2] as i32,
                )] = u64::MAX;

                chunk.offset_top -= 1;

            /*
             * 方块非空，此时为放置操作
             * 偏移量设置为offset_top
             * 然后递增offset_top
             */
            } else {
                let chunk = self.data.chunk_map.get_mut(&chunk_coord).unwrap();
                offset = chunk.offset_top;
                chunk.offset_top += 1;
                chunk.coord_to_offset[ChunkData::relative_to_index(
                    instance.position[0] as i32,
                    instance.position[1] as i32,
                    instance.position[2] as i32,
                )] = offset;
            }

            /*
             * 如果offset为MAX，则说明该方块不在缓冲区内
             * 则为新添加的方块， 其位置位于top_offset， 然后需要递增offset_top
             */

            queue.write_buffer(
                &self.render_res.instance_buffers[&chunk_coord],
                offset * std::mem::size_of::<Instance>() as u64,
                bytemuck::bytes_of(&instance),
            );
        }
    }

    //把单个方块插入缓冲区中 令单个方块可见
    //假定方块为实体
    pub fn update_single_block_buffer(&mut self, block_coord: Point3<i32>, queue: &Queue) {
        let chunk_coord = get_chunk_coord(block_coord.x, block_coord.z);
        let block_offset = get_local_coord(block_coord);

        let offset = self.data.chunk_map[&chunk_coord].offset_top;

        //更新offset_top和coord_to_offset
        let chunk = self.data.chunk_map.get_mut(&chunk_coord).unwrap();
        chunk.offset_top += 1;
        chunk.coord_to_offset
            [ChunkData::relative_to_index(block_offset.x, block_offset.y, block_offset.z)] = offset;

        let block = chunk.get_block(block_offset.x, block_offset.y, block_offset.z);

        queue.write_buffer(
            &self.render_res.instance_buffers[&chunk_coord],
            offset * std::mem::size_of::<Instance>() as u64,
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

    fn write_instance_buffer(
        &mut self,
        queue: &Queue,
        chunk_coord: &ChunkCoord,
        local_coord: &Point3<i32>,
        instance: &Instance,
        offset: u64,
    ) {
        queue.write_buffer(
            &self.render_res.instance_buffers[&chunk_coord],
            offset,
            bytemuck::bytes_of(instance),
        );
    }

    //block_offset是区块内的方块偏移量
    fn get_offset(&self, block_offset: &Point3<i32>) -> u64 {
        //let chunk_base_offset = self.data.coord_to_offset[chunk_coord] as u64;

        let block_offset = (block_offset.x * (CHUNK_SIZE * CHUNK_HEIGHT)
            + block_offset.y * CHUNK_SIZE
            + block_offset.z) as u64;

        //总方块数 = 区块偏移量 + 区块内偏移量
        //总偏移量 = 总方块数 * 实例大小
        block_offset * std::mem::size_of::<Instance>() as u64
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
