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
    pub instances: Vec<Instance>,
    pub instance_buffer: Buffer,
}

impl Realm {
    pub fn new(device: &Device) -> Self {
        let instances = (0..CHUNK_SIZE)
            .flat_map(|x| {
                (0..CHUNK_SIZE).map(move |z| {
                    let position: [f32; 3] = [x as f32, 0.0, z as f32];

                    Instance { position }
                })
            })
            .collect::<Vec<_>>();

        let instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("instance buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: BufferUsages::VERTEX,
        });

        Self {
            instances,
            instance_buffer,
        }
    }
}
