use crossbeam_channel::{bounded, Receiver, Sender};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::realm::{
    Block, BlockType, Chunk, ChunkCoord, ChunkData, Instance, RealmData, BLOCK_EMPTY,
    BLOCK_NUM_PER_CHUNK, CHUNK_HEIGHT, CHUNK_SIZE,
};

use noise::{NoiseFn, Perlin};

#[allow(unused)]
pub struct ChunkGenerator {
    request_sender: Sender<ChunkRequest>,
    response_receiver: Receiver<ChunkResponse>,
    pending_chunks: Arc<Mutex<HashSet<ChunkCoord>>>,
    num_threads: usize,
}

pub struct ChunkRequest {
    pub coord: ChunkCoord,
    pub seed: u32,
}

pub struct ChunkResponse {
    pub coord: ChunkCoord,
    pub chunk: Chunk,
}

impl ChunkGenerator {
    pub fn new(num_threads: usize) -> Self {
        let (req_sender, req_receiver) = bounded(1000);
        let (resp_sender, resp_receiver) = bounded(1000);
        let pending_chunks = Arc::new(Mutex::new(HashSet::new()));

        // 创建工作线程
        for _ in 0..num_threads {
            let req_receiver: Receiver<ChunkRequest> = req_receiver.clone();
            let resp_sender = resp_sender.clone();
            let pending_chunks = Arc::clone(&pending_chunks);

            thread::spawn(move || {
                while let Ok(request) = req_receiver.recv() {
                    // 生成区块
                    //thread::sleep(std::time::Duration::from_millis(2000)); // 模拟生成时间
                    let mut chunk = Self::generate_terrain_internal(request.coord, request.seed);

                    Self::create_instance(&mut chunk, &request.coord);
                    // 将生成的区块发送回去
                    let response = ChunkResponse {
                        coord: request.coord,
                        chunk,
                    };

                    resp_sender.send(response).unwrap();

                    // 从待处理集合中移除
                    pending_chunks.lock().unwrap().remove(&request.coord);
                }
            });
        }

        Self {
            request_sender: req_sender,
            response_receiver: resp_receiver,
            pending_chunks,
            num_threads,
        }
    }

    pub fn request_chunk(&self, coord: ChunkCoord, seed: u32) -> bool {
        // 检查这个区块是否已经在生成中
        let mut pending = self.pending_chunks.lock().unwrap();
        if pending.contains(&coord) {
            return false;
        }

        // 添加到待处理集合并发送生成请求
        pending.insert(coord);
        let request = ChunkRequest { coord, seed };
        self.request_sender.send(request).unwrap();
        true
    }

    pub fn get_generated_chunks(&self) -> Vec<ChunkResponse> {
        let mut results = Vec::new();
        while let Ok(response) = self.response_receiver.try_recv() {
            results.push(response);
        }
        results
    }

    pub fn is_chunk_pending(&self, coord: &ChunkCoord) -> bool {
        self.pending_chunks.lock().unwrap().contains(coord)
    }

    fn create_instance(chunk: &mut Chunk, chunk_coord: &ChunkCoord) {
        let mut index = 0u32;
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_HEIGHT {
                for z in 0..CHUNK_SIZE {
                    let block = chunk.get_block(x, y, z);
                    if block.tp != BlockType::Empty {
                        //let abs_coord = RealmData::relative_to_absolute(chunk_coord, x, y, z);
                        if chunk.has_any_visible_face(x, y, z) {
                            chunk.instance[index as usize] = Instance {
                                position: RealmData::relative_to_absolute_array(
                                    chunk_coord,
                                    x,
                                    y,
                                    z,
                                ),
                                block_type: block.tp as u32,
                            };
                            chunk.coord_to_offset[RealmData::relative_to_index(x, y, z)] = index;
                            chunk.offset_top += 1;
                            index += 1;
                        }
                    }
                }
            }
        }
    }

    fn generate_terrain_internal(chunk_coord: ChunkCoord, seed: u32) -> Chunk {
        let perlin = Perlin::new(seed);

        let blocks = vec![BLOCK_EMPTY; BLOCK_NUM_PER_CHUNK];
        let mut chunk = Chunk::new(ChunkData { blocks });
        let mut tree_placed: Vec<Vec<bool>> =
            vec![vec![false; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let absolute_x = x + chunk_coord.x * CHUNK_SIZE;
                let absolute_z = z + chunk_coord.z * CHUNK_SIZE;

                let height = (perlin.get([absolute_x as f64 / 16.0, absolute_z as f64 / 16.0])
                    * 8.0) as i32
                    + 64;

                let tree_value = perlin.get([
                    (absolute_z + CHUNK_SIZE / 2) as f64 / 4.0,
                    (absolute_x + CHUNK_SIZE / 2) as f64 / 4.0,
                ]);

                // 树木生成逻辑（与原代码相同）
                if tree_value > 0.80 {
                    if x > 2 && z > 2 && x < CHUNK_SIZE - 2 && z < CHUNK_SIZE - 2 {
                        // ... 现有的树木生成代码 ...
                        // (从原始的generate_terrian函数复制)
                        let mut is_place = true;
                        for i in -2..=2 {
                            for j in -2..=2 {
                                if tree_placed[(x + i) as usize][(z + j) as usize] {
                                    is_place = false;
                                    break;
                                }
                            }
                        }
                        if is_place {
                            let tree_height = ((1.0 - tree_value) * 15.0 + 4.0) as i32;
                            for y in 0..tree_height {
                                chunk.set_block(x, height + y, z, Block::new(BlockType::BirchLog));
                            }
                            for i in -2..=2 {
                                for j in -2..=2 {
                                    tree_placed[(x + i) as usize][(z + j) as usize] = true;
                                }
                            }

                            // 生成树叶
                            for i in -2..=2 {
                                for j in -2..=2 {
                                    let y = height + tree_height - 2;
                                    if i == 0 && j == 0 {
                                        continue;
                                    }
                                    chunk.set_block(
                                        x + i,
                                        y,
                                        z + j,
                                        Block::new(BlockType::BirchLeaves),
                                    );
                                }
                            }
                            for i in -2i32..=2 {
                                for j in -2i32..=2 {
                                    let y = height + tree_height - 1;
                                    if i == 0 && j == 0 {
                                        continue;
                                    }
                                    if i.abs() == 2 && j.abs() == 2 {
                                        continue;
                                    }
                                    chunk.set_block(
                                        x + i,
                                        y,
                                        z + j,
                                        Block::new(BlockType::BirchLeaves),
                                    );
                                }
                            }
                            for i in -1i32..=1 {
                                for j in -1i32..=1 {
                                    let y = height + tree_height;
                                    if i.abs() == 1 && j.abs() == 1 {
                                        continue;
                                    }
                                    chunk.set_block(
                                        x + i,
                                        y,
                                        z + j,
                                        Block::new(BlockType::BirchLeaves),
                                    );
                                }
                            }
                        }
                    }
                }

                // 地形生成
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

        chunk
    }
}
