use std::panic;
use std::path::Path;
use std::{collections::HashMap, sync::atomic::AtomicBool};

use anyhow::Context;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use wgpu::{util::DeviceExt, *};

use cgmath::*;

use crate::basic_config::{self, BasicConfig};
use crate::chunk_generator::{self, ChunkGenerator};
use crate::{camera, texture};

pub const TEXT_FRAC: f32 = 16.0 / 512.0;
pub const WF_SIZE: f32 = 0.01;
pub const WF_WIDTH: f32 = 0.04;
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

pub const INIT_CHUNK_RAD: i32 = 7;
pub const CHUNK_SIZE: i32 = 16;
pub const CHUNK_HEIGHT: i32 = 512;
pub const BLOCK_NUM_PER_CHUNK: usize = (CHUNK_SIZE * CHUNK_SIZE * CHUNK_HEIGHT) as usize;

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

    //白桦原木
    BirchLog = 5,

    //白桦树叶
    BirchLeaves = 6,

    //白桦木板
    BirchPlank = 7,

    TestBlock = 8,
}
//添加方块之后记得方块数量
pub const BLOCK_NUM: usize = 9;

impl BlockType {
    pub fn is_transparent(&self) -> bool {
        match self {
            //BlockType::BirchLeaves => true,
            BlockType::Empty => true,
            _ => false,
        }
    }
}

#[repr(u32)]
#[derive(FromPrimitive)]
pub enum BlockMaterials {
    Empty = 999,
    UnderStone = 0,
    Stone = 1,
    GrassBlockSide = 2,
    GrassBlockTop = 3,
    Dirt = 4,
    BirchLogTop = 5,
    BirchLog = 6,
    BirchLeaves = 7,
    BirchPlank = 8,
    TestBlock = 9,
}
// 添加材质后记得修改材质数量
pub const BLOCK_MATERIALS_NUM: u32 = 10;

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
            attributes: &[VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: VertexFormat::Float32x3,
            }],
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
    pub x: i32,
    pub z: i32,
}

impl ChunkCoord {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq)]
pub struct Block {
    pub tp: BlockType,
}

impl Block {
    pub fn new(tp: BlockType) -> Self {
        Self { tp }
    }
}

/*
 * 这是存储到文件中的数据
 */
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ChunkData {
    pub blocks: Vec<Block>,
}

impl ChunkData {
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Block {
        self.blocks[(x * CHUNK_SIZE * CHUNK_HEIGHT + y * CHUNK_SIZE + z) as usize]
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
#[derive(Debug, Clone)]
pub struct Chunk {
    data: ChunkData,
    pub coord_to_offset: Vec<u32>,
    pub instance: Vec<Instance>,
    //is_dirty: AtomicBool,

    //为第一个空位置
    pub offset_top: u32,
}

impl Chunk {
    pub fn new(data: ChunkData) -> Self {
        //let is_dirty = AtomicBool::new(false);
        let coord_to_offset = vec![u32::MAX; BLOCK_NUM_PER_CHUNK];
        let instance: Vec<Instance> = vec![Instance::default(); BLOCK_NUM_PER_CHUNK];
        let offset_top = 0;
        Self {
            data,
            //is_dirty,
            instance,
            coord_to_offset,
            offset_top,
        }
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Block {
        if x < 0 || y < 0 || z < 0 || x >= CHUNK_SIZE || y >= CHUNK_HEIGHT || z >= CHUNK_SIZE {
            return BLOCK_EMPTY;
        }
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
    pub fn has_any_visible_face(&self, x: i32, y: i32, z: i32) -> bool {
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
        //any()遍历检查是否有true，否则返回false
        .any(|(nx, ny, nz)| {
            // 相邻方块为透明，则面可见
            self.get_block(*nx, *ny, *nz).tp.is_transparent()
        })
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
        //草方块创建完成
        let all_block = create_all_block();

        let wf_uniform = WireframeUniform {
            position: [0.0, 0.0, 0.0],
            _padding: 0.0,
        };

        let wf_max_len: f32 = 6.0;

        let is_wf_visible = true;

        let center_chunk_pos = ChunkCoord { x: 0, z: 0 };

        let name = "./data/worlds/default_name_1";

        let chunk_rad: i32 = INIT_CHUNK_RAD;
        if chunk_rad < 0 {
            panic!("invaild chunk_rad value:{}", chunk_rad);
        }

        let chunk_map: HashMap<ChunkCoord, Chunk> = HashMap::new();

        let seed = 2025318;
        //init chunk
        //Self::load_all_chunk(
        //    chunk_rad as i32,
        //    center_chunk_pos,
        //    &mut chunk_map,
        //    name,
        //    seed,
        //);

        println!("chunk_map.len():{}", chunk_map.len());
        //Self::debug_print(&chunk_map.get(&center_chunk_pos).unwrap());

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

    pub fn get_block(&self, absolute_coord: Point3<i32>) -> Block {
        let mut x = absolute_coord.x;
        let y = absolute_coord.y;
        let mut z = absolute_coord.z;
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
    //fn load_all_chunk(
    //    chunk_rad: i32,
    //    center_chunk_pos: ChunkCoord,
    //    chunk_map: &mut HashMap<ChunkCoord, Chunk>,
    //    world_dir: &str,
    //    seed: u32,
    //) {
    //    for relative_x in -chunk_rad..=chunk_rad {
    //        for relative_z in -chunk_rad..=chunk_rad {
    //            let x = center_chunk_pos.x + relative_x;
    //            let z = center_chunk_pos.z + relative_z;
    //            let coord = ChunkCoord::new(x, z);
    //            Realm::init_chunk(&coord, chunk_map, world_dir, seed, &ChunkGenerator::new(4));
    //        }
    //    }
    //}

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

    pub fn load_all_instance(&mut self) {
        let coords = self.chunk_map.keys().cloned().collect::<Vec<ChunkCoord>>();
        for coord in coords {
            self.create_instance(&coord);
        }
    }

    //仅为创建可见方块创建实例
    pub fn create_instance(&mut self, chunk_coord: &ChunkCoord) {
        let mut index = 0u32;
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_HEIGHT {
                for z in 0..CHUNK_SIZE {
                    let block = self.chunk_map.get(chunk_coord).unwrap().get_block(x, y, z);
                    if block.tp != BlockType::Empty {
                        let abs_coord = Self::relative_to_absolute(chunk_coord, x, y, z);
                        if self.has_any_visible_face(abs_coord.x, abs_coord.y, abs_coord.z) {
                            let chunk = self.chunk_map.get_mut(chunk_coord).unwrap();
                            chunk.instance[index as usize] = Instance {
                                position: Self::relative_to_absolute_array(chunk_coord, x, y, z),
                                block_type: block.tp as u32,
                            };
                            chunk.coord_to_offset[Self::relative_to_index(x, y, z)] = index;
                            chunk.offset_top += 1;
                            index += 1;
                        }
                    }
                }
            }
        }
    }

    pub fn relative_to_absolute_array(
        chunk_coord: &ChunkCoord,
        x: i32,
        y: i32,
        z: i32,
    ) -> [f32; 3] {
        [
            (chunk_coord.x * CHUNK_SIZE + x) as f32,
            y as f32,
            (chunk_coord.z * CHUNK_SIZE + z) as f32,
        ]
    }

    pub fn relative_to_absolute(chunk_coord: &ChunkCoord, x: i32, y: i32, z: i32) -> Point3<i32> {
        Point3::new(
            chunk_coord.x * CHUNK_SIZE + x,
            y,
            chunk_coord.z * CHUNK_SIZE + z,
        )
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
        //any()遍历检查是否有true，否则返回false
        .any(|(nx, ny, nz)| {
            // 相邻方块为透明，则面可见
            self.get_block(Point3::new(*nx, *ny, *nz))
                .tp
                .is_transparent()
        })
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
    render_pipeline: RenderPipeline,
    pub diffuse_bind_group: BindGroup,
    pub texture_bind_group_layout: BindGroupLayout,
    pub render_pipeline_layout: PipelineLayout,
}
impl RenderResources {
    fn new(
        basic_config: &BasicConfig,
        data: &RealmData,
        camera_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let device = &basic_config.device;
        let block_vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("block vertex buffer"),
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

        //以下是创建 render_pipeline 和 buffer
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("First Shader"),
            source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        //let num_indices = realm::INDICES.len() as u32;

        let diffuse_texture =
            texture::Texture::load_blocks("res/texture", &basic_config.device, &basic_config.queue)
                .unwrap();

        let texture_bind_group_layout =
            basic_config
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Texture bind group layout"),
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Texture {
                                sample_type: TextureSampleType::Float { filterable: true },
                                view_dimension: TextureViewDimension::D2Array,
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

        let diffuse_bind_group =
            basic_config
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("First diffuse bind group"),
                    layout: &texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                        },
                    ],
                });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("First render pipeline layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &texture_bind_group_layout,
                &block_materials_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("First render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), Instance::desc()],
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
                    format: basic_config.config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });
        //render_pipeline和buffer创建完成

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
            diffuse_bind_group,
            render_pipeline,
            texture_bind_group_layout,
            render_pipeline_layout,
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
    chunk_generator: ChunkGenerator,
    pub is_loading: bool,
    is_init: bool,
    chunk_update_timer: f64,
    chunk_check_interval: f64,
    pub pre_dx: i32,
    pub pre_dz: i32,
    pre_center_chunk_pos: ChunkCoord,
}
impl Realm {
    pub fn new(basic_config: &BasicConfig, camera_bind_group_layout: &BindGroupLayout) -> Self {
        let mut data = RealmData::new();
        data.load_all_instance();

        let render_res = RenderResources::new(basic_config, &data, camera_bind_group_layout);
        let chunk_generator = ChunkGenerator::new(1);

        let is_loading = true;

        let chunk_check_interval = 0.05; // 0.5秒检查一次

        let pre_center_chunk_pos = data.center_chunk_pos;

        Self {
            data,
            render_res,
            chunk_generator,
            is_loading,
            chunk_update_timer: 0.0,
            chunk_check_interval,
            pre_dx: 0,
            pre_dz: 0,
            pre_center_chunk_pos,
            is_init: true,
        }
    }

    #[allow(unused)]
    fn generate_terrian_test(
        chunk_map: &mut HashMap<ChunkCoord, Chunk>,
        chunk_coord: &ChunkCoord,
        seed: u32,
    ) {
        let blocks = vec![BLOCK_EMPTY; BLOCK_NUM_PER_CHUNK];
        let mut chunk = Chunk::new(ChunkData { blocks });

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_HEIGHT {
                for z in 0..CHUNK_SIZE {
                    if y == 0 {
                        chunk.set_block(x, y, z, Block::new(BlockType::UnderStone));
                    } else if y == 1 {
                        chunk.set_block(x, y, z, Block::new(BlockType::Stone));
                    } else if y == 2 {
                        chunk.set_block(x, y, z, Block::new(BlockType::Dirt));
                    } else if y == 3 {
                        chunk.set_block(x, y, z, Block::new(BlockType::Grass));
                    }
                }
            }
        }
        chunk_map.insert(*chunk_coord, chunk);
    }

    #[allow(unused)]
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
        chunk_generator: &ChunkGenerator,
    ) -> bool {
        match chunk_map.get(chunk_pos) {
            //存在就不要重复添加
            Some(_) => {
                return true;
            }

            None => {
                if chunk_generator.is_chunk_pending(chunk_pos) {
                    return false;
                }
                match Chunk::load(world_dir, chunk_pos) {
                    //不存在则尝试读取
                    Ok(Some(data)) => {
                        let chunk = Chunk::new(data);
                        chunk_map.insert(*chunk_pos, chunk);
                        return true;
                    }

                    //读取失败则生成
                    Ok(None) => {
                        chunk_generator.request_chunk(*chunk_pos, seed);
                        return false;
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

    //加载区块同时加载进GPU资源中
    fn load_chunk(&mut self, device: &Device, new_chunk_pos: &ChunkCoord) -> bool {
        let loaded = Self::init_chunk(
            new_chunk_pos,
            &mut self.data.chunk_map,
            self.data.name,
            self.data.seed,
            &self.chunk_generator,
        );

        if loaded {
            self.data.create_instance(new_chunk_pos);
            self.render_res.insert_instance_buffer(
                device,
                new_chunk_pos,
                &self.data.chunk_map.get(new_chunk_pos).unwrap().instance,
            );
        }
        loaded
    }

    fn process_generated_chunks(&mut self, device: &Device) {
        let generated_chunks = self.chunk_generator.get_generated_chunks();
        for respose in generated_chunks {
            self.data.chunk_map.insert(respose.coord, respose.chunk);
            //self.data.create_instance(&respose.coord);
            self.render_res.insert_instance_buffer(
                device,
                &respose.coord,
                &self.data.chunk_map.get(&respose.coord).unwrap().instance,
            );
            //println!("并行加载区块:{:?}", respose.coord);
        }
    }

    fn unload_chunk(&mut self, chunk_pos: &ChunkCoord) {
        self.render_res.instance_buffers.remove(chunk_pos);
        self.data.chunk_map.remove(chunk_pos);
        //println!("卸载区块:{:?}", chunk_pos);
    }

    pub fn update(&mut self, player_pos: &Point3<f32>, device: &Device, dt: f64) {
        self.process_generated_chunks(device);

        //控制轮询频率
        if self.is_loading {
            if self.chunk_update_timer > self.chunk_check_interval {
                self.chunk_update_timer = 0.0;

                //不只更新边界
                if self.pre_dx.abs() > 1 || self.pre_dz.abs() > 1 || self.is_init {
                    //这里要检查的是新的中心区块
                    if self.check_all_chunk_loaded(&ChunkCoord {
                        x: self.data.center_chunk_pos.x + self.pre_dx,
                        z: self.data.center_chunk_pos.z + self.pre_dz,
                    }) {
                        self.is_loading = false;
                        self.data.center_chunk_pos = ChunkCoord::new(
                            self.data.center_chunk_pos.x + self.pre_dx,
                            self.data.center_chunk_pos.z + self.pre_dz,
                        );
                        self.unload_all_redundant_chunk();

                        if self.is_init {
                            self.is_init = false;
                        }

                        println!(
                            "更新所有区块:offset:{},{}, 现在center_chunk_pos=({},{})",
                            self.pre_dx,
                            self.pre_dz,
                            self.data.center_chunk_pos.x,
                            self.data.center_chunk_pos.z
                        );
                    }

                //仅更新边界
                } else {
                    if self.check_border_loaded(self.pre_dx, self.pre_dz) {
                        println!(
                            "更新边界前中心:({},{})",
                            self.data.center_chunk_pos.x, self.data.center_chunk_pos.z
                        );
                        self.is_loading = false;
                        self.data.center_chunk_pos = ChunkCoord::new(
                            self.data.center_chunk_pos.x + self.pre_dx,
                            self.data.center_chunk_pos.z + self.pre_dz,
                        );
                        self.unload_border_chunk();
                        println!(
                            "更新边界:offset:{},{} 现在cenetr_chunk_pos=({},{})",
                            self.pre_dx,
                            self.pre_dz,
                            self.data.center_chunk_pos.x,
                            self.data.center_chunk_pos.z
                        );
                    }
                }

                //已经在加载了，如果加载时间比较短先不更新
            } else {
                self.chunk_update_timer += dt;
                return;
            }

        //如果没有区块正在加载
        } else {
            let new_coord = get_chunk_coord(player_pos.x as i32, player_pos.z as i32);
            let dx = new_coord.x - self.data.center_chunk_pos.x;
            let dz = new_coord.z - self.data.center_chunk_pos.z;
            //则检查是否需要加载
            if dx == 0 && dz == 0 {
                return;
            }

            //走出中心区块，而且相应的区块没有加载，则加载
            if dx.abs() > 1 || dz.abs() > 1 {
                if !self.check_all_chunk_loaded(&new_coord) {
                    self.is_loading = true;
                    self.load_all_chunk(device, &new_coord);

                    println!("加载所有区块:offset:{},{}", dx, dz);
                } else {
                    self.data.center_chunk_pos = new_coord;

                    //一切正常的话，以下应该不会执行
                    //self.unload_all_redundant_chunk();
                }
            } else {
                if !self.check_border_loaded(dx, dz) {
                    println!("加载边界:offset:{},{}", dx, dz);
                    self.is_loading = true;
                    self.update_helper(dx, dz, device);
                } else {
                    self.data.center_chunk_pos = new_coord;
                    self.unload_border_chunk();
                }
            }

            //self.pre_center_chunk_pos = self.data.center_chunk_pos;
            self.pre_dx = dx;
            self.pre_dz = dz;
        }
    }

    //两个offset都只有-1, 0, 1三个值
    //此时的区块中心还没更新
    fn update_helper(&mut self, x_offset: i32, z_offset: i32, device: &Device) {
        if z_offset != 0 {
            for x in -self.data.chunk_rad..=self.data.chunk_rad {
                //let old_chunk_pos = ChunkCoord::new(
                //    self.data.center_chunk_pos.x + x,
                //    self.data.center_chunk_pos.z - self.data.chunk_rad * z_offset,
                //);
                let new_chunk_pos = ChunkCoord::new(
                    self.data.center_chunk_pos.x + x + x_offset,
                    self.data.center_chunk_pos.z + (self.data.chunk_rad + 1) * z_offset,
                );
                //卸载和加载顺序不能反
                self.load_chunk(device, &new_chunk_pos);
                //self.unload_chunk(&old_chunk_pos);
            }
        }
        if x_offset != 0 {
            for z in -self.data.chunk_rad..=self.data.chunk_rad {
                //let old_chunk_pos = ChunkCoord::new(
                //    self.data.center_chunk_pos.x - self.data.chunk_rad * x_offset,
                //    self.data.center_chunk_pos.z + z,
                //);
                let new_chunk_pos = ChunkCoord::new(
                    self.data.center_chunk_pos.x + (self.data.chunk_rad + 1) * x_offset,
                    self.data.center_chunk_pos.z + z + z_offset,
                );
                self.load_chunk(device, &new_chunk_pos);
                //self.unload_chunk(&old_chunk_pos);
            }
        }
    }

    fn check_all_chunk_loaded(&self, new_coord: &ChunkCoord) -> bool {
        for x in -self.data.chunk_rad..=self.data.chunk_rad {
            for z in -self.data.chunk_rad..=self.data.chunk_rad {
                let chunk_pos = ChunkCoord::new(new_coord.x + x, new_coord.z + z);
                if !self.data.chunk_map.contains_key(&chunk_pos) {
                    return false;
                }
            }
        }
        true
    }

    fn unload_border_chunk(&mut self) {
        if self.data.chunk_map.contains_key(&ChunkCoord::new(
            self.data.center_chunk_pos.x,
            self.data.center_chunk_pos.z - self.data.chunk_rad - 2,
        )) {
            for x in -self.data.chunk_rad - 1..=self.data.chunk_rad + 1 {
                let old_chunk_pos = ChunkCoord::new(
                    self.data.center_chunk_pos.x + x,
                    self.data.center_chunk_pos.z - (self.data.chunk_rad + 2),
                );
                self.unload_chunk(&old_chunk_pos);
            }
        }

        if self.data.chunk_map.contains_key(&ChunkCoord::new(
            self.data.center_chunk_pos.x,
            self.data.center_chunk_pos.z + self.data.chunk_rad + 2,
        )) {
            for x in -self.data.chunk_rad - 1..=self.data.chunk_rad + 1 {
                let old_chunk_pos = ChunkCoord::new(
                    self.data.center_chunk_pos.x + x,
                    self.data.center_chunk_pos.z + (self.data.chunk_rad + 2),
                );
                self.unload_chunk(&old_chunk_pos);
            }
        }

        if self.data.chunk_map.contains_key(&ChunkCoord::new(
            self.data.center_chunk_pos.x + self.data.chunk_rad + 2,
            self.data.center_chunk_pos.z,
        )) {
            for z in -self.data.chunk_rad - 1..=self.data.chunk_rad + 1 {
                let old_chunk_pos = ChunkCoord::new(
                    self.data.center_chunk_pos.x + (self.data.chunk_rad + 2),
                    self.data.center_chunk_pos.z + z,
                );
                self.unload_chunk(&old_chunk_pos);
            }
        }

        if self.data.chunk_map.contains_key(&ChunkCoord::new(
            self.data.center_chunk_pos.x - self.data.chunk_rad - 2,
            self.data.center_chunk_pos.z,
        )) {
            for z in -self.data.chunk_rad - 1..=self.data.chunk_rad + 1 {
                let old_chunk_pos = ChunkCoord::new(
                    self.data.center_chunk_pos.x - (self.data.chunk_rad + 2),
                    self.data.center_chunk_pos.z + z,
                );
                self.unload_chunk(&old_chunk_pos);
            }
        }
    }

    fn unload_all_redundant_chunk(&mut self) {
        //卸载多余区块
        self.data.chunk_map.retain(|coord, _| {
            (coord.x - self.data.center_chunk_pos.x).abs() <= self.data.chunk_rad + 1
                && (coord.z - self.data.center_chunk_pos.z).abs() <= self.data.chunk_rad + 1
        });
    }

    fn load_all_chunk(&mut self, device: &Device, new_coord: &ChunkCoord) {
        for x in -self.data.chunk_rad..=self.data.chunk_rad {
            for z in -self.data.chunk_rad..=self.data.chunk_rad {
                let chunk_pos = ChunkCoord::new(new_coord.x + x, new_coord.z + z);
                self.load_chunk(device, &chunk_pos);
            }
        }
    }

    fn check_border_loaded(&self, x_offset: i32, z_offset: i32) -> bool {
        if z_offset != 0 {
            for x in -self.data.chunk_rad..=self.data.chunk_rad {
                let new_chunk_pos = ChunkCoord::new(
                    self.data.center_chunk_pos.x + x + x_offset,
                    self.data.center_chunk_pos.z + (self.data.chunk_rad + 1) * z_offset,
                );
                if !self.data.chunk_map.contains_key(&new_chunk_pos) {
                    return false;
                }
            }
        }
        if x_offset != 0 {
            for z in -self.data.chunk_rad..=self.data.chunk_rad {
                let new_chunk_pos = ChunkCoord::new(
                    self.data.center_chunk_pos.x + (self.data.chunk_rad + 1) * x_offset,
                    self.data.center_chunk_pos.z + z + z_offset,
                );
                if !self.data.chunk_map.contains_key(&new_chunk_pos) {
                    return false;
                }
            }
        }
        true
    }

    pub fn reload_all_chunk(&mut self, new_coord: &ChunkCoord, device: &Device) {
        // 卸载所有区块
        //self.data.chunk_map.clear();
        //self.render_res.instance_buffers.clear();
        println!("重新加载所有区块");

        // 卸载多余区块
        self.data.chunk_map.retain(|coord, _| {
            (coord.x - new_coord.x).abs() > self.data.chunk_rad + 1
                || (coord.z - new_coord.z).abs() > self.data.chunk_rad + 1
        });

        // 请求所有需要的区块
        for relative_x in -self.data.chunk_rad..=self.data.chunk_rad {
            for relative_z in -self.data.chunk_rad..=self.data.chunk_rad {
                let x = new_coord.x + relative_x;
                let z = new_coord.z + relative_z;
                let coord = ChunkCoord::new(x, z);

                Self::init_chunk(
                    &coord,
                    &mut self.data.chunk_map,
                    self.data.name,
                    self.data.seed,
                    &self.chunk_generator,
                );
            }
        }
        self.render_res.instance_buffers =
            RenderResources::init_instance_buffers(device, &self.data);
    }

    //返回位置是否合法
    fn set_block_data(&mut self, abs_coord: Point3<i32>, block: Block) -> bool {
        let chunk_coord = get_chunk_coord(abs_coord.x, abs_coord.z);

        // 检查区块是否存在
        if let Some(chunk) = self.data.chunk_map.get_mut(&chunk_coord) {
            let local_coord = get_local_coord(abs_coord);

            // 检查 y 坐标是否在有效范围内
            if abs_coord.y >= 0 && abs_coord.y < CHUNK_HEIGHT as i32 {
                chunk.set_block(local_coord.x, local_coord.y, local_coord.z, block);

                // 标记区块为需要更新状态
                //chunk
                //    .is_dirty
                //    .store(true, std::sync::atomic::Ordering::Relaxed);
                return true;
            }
        }
        false
    }

    fn get_offset(&self, abs_coord: Point3<i32>) -> Option<u32> {
        let chunk_coord = get_chunk_coord(abs_coord.x, abs_coord.z);
        let local_coord = get_local_coord(abs_coord);
        if let Some(chunk) = self.data.chunk_map.get(&chunk_coord) {
            return Some(
                chunk.coord_to_offset
                    [RealmData::relative_to_index(local_coord.x, local_coord.y, local_coord.z)],
            );
        }
        None
    }

    fn update_adjacent_block(&mut self, abs_coord: Point3<i32>, queue: &Queue) {
        //更新相邻方块
        let x = abs_coord.x;
        let y = abs_coord.y;
        let z = abs_coord.z;
        [
            (x + 1, y, z),
            (x - 1, y, z),
            (x, y + 1, z),
            (x, y - 1, z),
            (x, y, z + 1),
            (x, y, z - 1),
        ]
        .iter()
        .for_each(|(x, y, z)| {
            if let Some(offset) = self.get_offset(Point3::new(*x, *y, *z)) {
                //如果offset为MAX，则说明该方块不在缓冲区内，即不可见
                //又如果方块为非空，则讲此方块插入缓冲区
                if offset == u32::MAX {
                    if !self
                        .data
                        .get_block(Point3::new(*x, *y, *z))
                        .tp
                        .is_transparent()
                    {
                        self.update_single_block_buffer(Point3::new(*x, *y, *z), queue);
                    }
                }
            }
        });
    }

    pub fn place_block(&mut self, block_coord: Point3<i32>, block: Block, queue: &Queue) {
        if self.set_block_data(block_coord, block) {
            let chunk_coord = get_chunk_coord(block_coord.x, block_coord.z);
            let block_offset = get_local_coord(block_coord);

            //offset为选中方块的偏移量
            let mut offset = self.data.chunk_map[&chunk_coord].coord_to_offset
                [RealmData::relative_to_index(block_offset.x, block_offset.y, block_offset.z)];

            let mut instance = Instance {
                position: [
                    block_coord.x as f32,
                    block_coord.y as f32,
                    block_coord.z as f32,
                ],
                block_type: block.tp as u32,
            };

            /*
             * 如果是空方块则为摧毁方块操作
             * 则需要更新相邻方块
             * 且偏移量一定有效，这是在camera摧毁方块操作保证了的！
             * 同时用缓冲区顶部的数据覆盖当前数据
             * 同时还需要更新offset_top和coord_to_offset
             * coord_to_offset需要更新两个位置
             */
            if block.tp == BlockType::Empty {
                //这是最后一个有效实例
                instance = self.data.chunk_map[&chunk_coord].instance
                    [(self.data.chunk_map[&chunk_coord].offset_top - 1) as usize];

                let chunk = self.data.chunk_map.get_mut(&chunk_coord).unwrap();

                chunk.offset_top -= 1;

                //用数组末尾的值覆盖要删除的值
                chunk.instance[(offset) as usize] = instance;

                let move_instance = chunk.instance[chunk.offset_top as usize];
                let move_coor = get_local_coord(Point3::new(
                    move_instance.position[0] as i32,
                    move_instance.position[1] as i32,
                    move_instance.position[2] as i32,
                ));

                chunk.coord_to_offset
                    [RealmData::relative_to_index(move_coor.x, move_coor.y, move_coor.z)] = offset;

                chunk.coord_to_offset[RealmData::relative_to_index(
                    block_offset.x,
                    block_offset.y,
                    block_offset.z,
                )] = u32::MAX;

                /*
                 * 以下两个函数调用不能反！！！
                 * 否则update_adjacent_block原本写入的缓冲
                 * 可能会被write_buffer覆写！
                 *
                 * bug发生在缓冲区最后一个实例是最新实例的情况下
                 */
                queue.write_buffer(
                    &self.render_res.instance_buffers[&chunk_coord],
                    (offset * std::mem::size_of::<Instance>() as u32) as u64,
                    bytemuck::bytes_of(&instance),
                );

                self.update_adjacent_block(block_coord, queue);

                /*
                 * 方块非空，此时为放置操作
                 * 偏移量设置为offset_top
                 *
                 * 更新:
                 * offset_top
                 * coord_to_offset
                 * instance
                 */
            } else {
                let chunk = self.data.chunk_map.get_mut(&chunk_coord).unwrap();
                offset = chunk.offset_top;
                chunk.offset_top += 1;
                chunk.coord_to_offset[RealmData::relative_to_index(
                    block_offset.x,
                    block_offset.y,
                    block_offset.z,
                )] = offset;
                chunk.instance[offset as usize] = instance;
                queue.write_buffer(
                    &self.render_res.instance_buffers[&chunk_coord],
                    offset as u64 * std::mem::size_of::<Instance>() as u64,
                    bytemuck::bytes_of(&instance),
                );
            }

            /*
             * 如果offset为MAX，则说明该方块不在缓冲区内
             * 则为新添加的方块， 其位置位于top_offset， 然后需要递增offset_top
             */
        }
    }

    //把单个方块插入缓冲区中 令单个方块可见
    //假定方块为实体
    pub fn update_single_block_buffer(&mut self, abs_coord: Point3<i32>, queue: &Queue) {
        let chunk_coord = get_chunk_coord(abs_coord.x, abs_coord.z);
        let block_offset = get_local_coord(abs_coord);

        let offset = self.data.chunk_map[&chunk_coord].offset_top;

        let chunk = self.data.chunk_map.get_mut(&chunk_coord).unwrap();
        let block = chunk.get_block(block_offset.x, block_offset.y, block_offset.z);
        //更新offset_top和coord_to_offset，不需要更新instance
        let instance = Instance {
            position: [abs_coord.x as f32, abs_coord.y as f32, abs_coord.z as f32],
            block_type: block.tp as u32,
        };

        chunk.offset_top += 1;
        chunk.coord_to_offset
            [RealmData::relative_to_index(block_offset.x, block_offset.y, block_offset.z)] = offset;
        chunk.instance[offset as usize] = instance;

        //println!(
        //    "insert instance{:?} to {:?},{:?}",
        //    instance, abs_coord, offset
        //);
        queue.write_buffer(
            &self.render_res.instance_buffers[&chunk_coord],
            offset as u64 * std::mem::size_of::<Instance>() as u64,
            bytemuck::bytes_of(&instance),
        );
    }

    #[allow(unused)]
    pub fn debug_print(&self) {
        println!("chunk data:");
        for x in 0..self.data.chunk_map[&ChunkCoord { x: 0, z: 0 }].offset_top {
            println!(
                "{}: {:?}",
                x,
                self.data.chunk_map[&ChunkCoord { x: 0, z: 0 }].instance[x as usize]
            );
        }
        println!("chunk coord_to_offset:");
        let chunk = self.data.chunk_map.get(&ChunkCoord { x: 0, z: 0 }).unwrap();
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_HEIGHT {
                for z in 0..CHUNK_SIZE {
                    println!(
                        "{},{},{}: {}",
                        x,
                        y,
                        z,
                        chunk.coord_to_offset[RealmData::relative_to_index(x, y, z)]
                    );
                }
            }
        }
    }

    pub fn draw_realm(&self, render_pass: &mut RenderPass, camera_bind_group: &BindGroup) {
        render_pass.set_pipeline(&self.render_res.render_pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_bind_group(1, &self.render_res.diffuse_bind_group, &[]);
        render_pass.set_bind_group(2, &self.render_res.block_materials_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.render_res.block_vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            self.render_res.block_index_buffer.slice(..),
            IndexFormat::Uint16,
        );

        for (coord, chunk) in self.data.chunk_map.iter() {
            render_pass.set_vertex_buffer(1, self.render_res.instance_buffers[&coord].slice(..));
            render_pass.draw_indexed(0..INDICES.len() as _, 0, 0..chunk.offset_top as u32);
        }
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

fn create_all_block() -> Vec<BlockInfo> {
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

    let brich_log = BlockInfo::new(
        "birch_log",
        [
            BirchLog as u32,
            BirchLogTop as u32,
            BirchLog as u32,
            BirchLogTop as u32,
            BirchLog as u32,
            BirchLog as u32,
        ],
        BlockType::BirchLog,
    );
    all_block[brich_log.block_type as usize] = brich_log;

    let brich_leaves = BlockInfo::new(
        "birch_leaves",
        [
            BirchLeaves as u32,
            BirchLeaves as u32,
            BirchLeaves as u32,
            BirchLeaves as u32,
            BirchLeaves as u32,
            BirchLeaves as u32,
        ],
        BlockType::BirchLeaves,
    );
    all_block[brich_leaves.block_type as usize] = brich_leaves;

    let birch_plank = BlockInfo::new(
        "birch_plank",
        [
            BirchPlank as u32,
            BirchPlank as u32,
            BirchPlank as u32,
            BirchPlank as u32,
            BirchPlank as u32,
            BirchPlank as u32,
        ],
        BlockType::BirchPlank,
    );
    all_block[birch_plank.block_type as usize] = birch_plank;

    let test_block = BlockInfo::new(
        "test_block",
        [
            TestBlock as u32,
            TestBlock as u32,
            TestBlock as u32,
            TestBlock as u32,
            TestBlock as u32,
            TestBlock as u32,
        ],
        BlockType::TestBlock,
    );
    all_block[test_block.block_type as usize] = test_block;

    all_block
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
