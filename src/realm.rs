use wgpu::{util::DeviceExt, *};

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

const CHUNK_SIZE: u32 = 16;

pub struct Block {
    pub instances: Vec<Instance>,
    vertices: [Vertex; 24],
    pub instance_buffer: Buffer,
    pub vertex_buffer: Buffer,

    //材质偏移量
    tex_x: u32,
    tex_y: u32,
}

impl Block {
    fn new(
        instances: Vec<Instance>,
        instance_buffer: Buffer,
        device: &Device,
        tex_x: u32,
        tex_y: u32,
    ) -> Self {
        let mut vertices: [Vertex; 24] = [Vertex {
            position: [0.0; 3],
            tex_coords: [0.0; 2],
        }; 24];

        for i in 0..24 {
            vertices[i].tex_coords[0] = VERTICES[i].tex_coords[0] + TEXT_FRAC * tex_x as f32;
            vertices[i].tex_coords[1] = VERTICES[i].tex_coords[1] + TEXT_FRAC * tex_y as f32;

            vertices[i].position = VERTICES[i].position;
        }

        //println!("{:#?}", vertices);

        let vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("block vertex buffer"),
            contents: bytemuck::cast_slice(&vertices[..]),
            usage: BufferUsages::VERTEX,
        });

        Self {
            instances,
            instance_buffer,
            vertex_buffer,
            vertices,
            tex_x,
            tex_y,
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

pub struct Realm {
    pub blocks: Vec<Block>,
}

impl Realm {
    pub fn new(device: &Device) -> Self {
        let mut blocks: Vec<Block> = Vec::new();

        let under_stone_instances = (0..CHUNK_SIZE)
            .flat_map(|x| {
                (0..CHUNK_SIZE).map(move |z| {
                    let position: [f32; 3] = [x as f32, 0.0, z as f32];

                    Instance { position }
                })
            })
            .collect::<Vec<_>>();

        let under_stone_instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("under stone instance buffer"),
            contents: bytemuck::cast_slice(&under_stone_instances),
            usage: BufferUsages::VERTEX,
        });

        let under_stone = Block::new(
            under_stone_instances,
            under_stone_instance_buffer,
            device,
            1,
            0,
        );
        blocks.push(under_stone);

        let mut stone_instances = (0..CHUNK_SIZE)
            .flat_map(|x| {
                (0..CHUNK_SIZE).map(move |z| {
                    let position: [f32; 3] = [x as f32, 1.0, z as f32];

                    Instance { position }
                })
            })
            .collect::<Vec<_>>();

        let stone_instances2 = (0..CHUNK_SIZE)
            .flat_map(|x| {
                (0..CHUNK_SIZE).map(move |z| {
                    let position: [f32; 3] = [x as f32, 2.0, z as f32];

                    Instance { position }
                })
            })
            .collect::<Vec<_>>();

        stone_instances.extend(stone_instances2);

        let stone_instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("stone instance buffer"),
            contents: bytemuck::cast_slice(&stone_instances),
            usage: BufferUsages::VERTEX,
        });

        let stone = Block::new(stone_instances, stone_instance_buffer, device, 0, 0);

        blocks.push(stone);

        Self { blocks }
    }
}
