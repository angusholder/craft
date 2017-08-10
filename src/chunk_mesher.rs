use std::thread;
use std::sync::mpsc;

use block::{ Block, BlockType };
use chunk::{ CHUNK_BLOCK_COUNT, Chunk, CHUNK_SIDE_LENGTH, WORLD_HEIGHT };
use math::*;

//pub enum RenderType {
//    // Solid materials, leaves
//    FullBlock {
//       transparency: bool,
//       side_textures: [u8; 6],
//    },
//
//    // Solid materials
//    HalfBlock {
//       side_textures: [u8; 6],
//    },
//
//    // Grass, sugar cane
//    Cross {
//        texture: u8,
//    },
//
//    // Light torches, redstone torches
//    Torch,
//
//    // Pressure plate, redstone wire
//    FlatTile {
//       transparency: bool,
//    },
//}

static BLOCK_SPECS: [[u8; 6]; 16] = [
    // [Block] = [Left, Right, Bottom, Top, Front, Back],
    [ 0,  1,  2,  3,  4,  5], // Block_Air - Dummy values to make glitches obvious
    [ 3,  3,  3,  3,  3,  3], // Block_Dirt
    [ 2,  2,  3,  1,  2,  2], // Block_Grass
    [ 0,  0,  0,  0,  0,  0], // Block_Stone
    [ 4,  4,  4,  4,  4,  4], // Block_Cobblestone
    [ 5,  5,  5,  5,  5,  5], // Block_Wood
    [20, 20, 19, 19, 20, 20], // Block_Log
    [13, 13, 13, 13, 13, 13], // Block_Bedrock
    [14, 14, 14, 14, 14, 14], // Block_Sand
    [15, 15, 15, 15, 15, 15], // Block_Gravel
    [16, 16, 16, 16, 16, 16], // Block_GoldOre
    [17, 17, 17, 17, 17, 17], // Block_IronOre
    [18, 18, 18, 18, 18, 18], // Block_CoalOre
    [24, 24, 24, 24, 24, 24], // Block_Leaf
    [28, 28, 28, 28, 28, 28], // Block_Sponge
    [38, 38, 37, 36, 38, 38], // Block_Sandstone
];

static BLOCK_NAMES: [&str; 16] = [
    "Air",
    "Dirt",
    "Grass",
    "Stone",
    "Cobblestone",
    "Wood",
    "Log",
    "Bedrock",
    "Sand",
    "Gravel",
    "Gold Ore",
    "Iron Ore",
    "Coal Ore",
    "Leaf",
    "Sponge",
    "Sandstone",
];

static UNIT_CUBE_FACES: [[u8; 3]; 36] = [
    // Left
    [0, 0, 0],
    [0, 1, 0],
    [0, 1, 1],
    [0, 1, 1],
    [0, 0, 1],
    [0, 0, 0],

    // Right
    [1, 0, 1],
    [1, 1, 1],
    [1, 1, 0],
    [1, 1, 0],
    [1, 0, 0],
    [1, 0, 1],

    // Bottom
    [1, 0, 1],
    [1, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 1],
    [1, 0, 1],

    // Top
    [1, 1, 0],
    [1, 1, 1],
    [0, 1, 1],
    [0, 1, 1],
    [0, 1, 0],
    [1, 1, 0],

    // Front
    [1, 0, 0],
    [1, 1, 0],
    [0, 1, 0],
    [0, 1, 0],
    [0, 0, 0],
    [1, 0, 0],

    // Back
    [0, 0, 1],
    [0, 1, 1],
    [1, 1, 1],
    [1, 1, 1],
    [1, 0, 1],
    [0, 0, 1],
];

static AXIS_OFFSETS: [[i8; 3]; 6] = [
    [-1, 0, 0],
    [1, 0, 0],
    [0, -1, 0],
    [0, 1, 0],
    [0, 0, -1],
    [0, 0, 1],
];

static UV_FACES_OFFSETS: [[u8; 2]; 6] = [
    [1, 1],
    [1, 0],
    [0, 0],
    [0, 0],
    [0, 1],
    [1, 1],
];

#[derive(Default, Clone, Copy)]
pub struct ChunkVertex {
    x: u8,
    y: u8,
    z: u8,

    u: u8,
    v: u8,
}

implement_vertex!{
    ChunkVertex,
    x normalize(false),
    y normalize(false),
    z normalize(false),
    u normalize(false),
    v normalize(false)
}

pub struct ChunkMesher {
    thread_handle: thread::JoinHandle<()>,
    tx_req: mpsc::Sender<Request>,
    rx_resp: mpsc::Receiver<Response>,
}

type Request = (ChunkCoord, Box<Chunk>);
type Response = (ChunkCoord, Vec<ChunkVertex>);

impl ChunkMesher {
    pub fn new() -> ChunkMesher {
        let (tx_req, rx_req): (mpsc::Sender<Request>, _) = mpsc::channel();
        let (tx_resp, rx_resp) = mpsc::channel();
        let thread_handle = thread::spawn(move || {
            for (coord, chunk) in rx_req.iter() {
                let mesh = create_mesh(&chunk);
                println!("Finished meshing chunk at {} with {} vertices ({} quads)",
                    coord, mesh.len(), mesh.len() / 6);

                tx_resp.send((coord, mesh));
            }
        });

        ChunkMesher {
            tx_req, rx_resp, thread_handle
        }
    }

    pub fn start_meshing(&mut self, coord: ChunkCoord, chunk: &Chunk) {
        self.tx_req.send((coord, Box::new(chunk.clone()))).unwrap();
    }

    pub fn iter_meshed<'a>(&'a mut self) -> ResponseIter {
        ResponseIter(self.rx_resp.try_iter())
    }
}

pub struct ResponseIter<'a>(mpsc::TryIter<'a, Response>);

impl<'a> Iterator for ResponseIter<'a> {
    type Item = Response;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

const MAX_VERTICES_PER_CHUNK: usize = CHUNK_BLOCK_COUNT * 6 * 6;
pub fn create_mesh(chunk: &Chunk) -> Vec<ChunkVertex> {
    let mut vertices = vec![ChunkVertex::default(); MAX_VERTICES_PER_CHUNK];
    let mut vertex_count = 0usize;

    for (Coord {x, y, z}, block) in chunk.iter() {
        if block.ty == BlockType::Air {
            continue;
        }
        for k in 0..6 {
            let adj_x = x + AXIS_OFFSETS[k][0] as i32;
            let adj_y = y + AXIS_OFFSETS[k][1] as i32;
            let adj_z = z + AXIS_OFFSETS[k][2] as i32;
            if 0 <= adj_x && adj_x < CHUNK_SIDE_LENGTH as i32 &&
                0 <= adj_y && adj_y < WORLD_HEIGHT as i32 &&
                0 <= adj_z && adj_z < CHUNK_SIDE_LENGTH as i32
            {
                let adjacent = chunk.get(Coord::new(adj_x, adj_y, adj_z));
                if adjacent.ty != BlockType::Air {
                    continue;
                }
            }

            let block_tex_index = BLOCK_SPECS[block.ty as usize][k];
            let u = block_tex_index % 16;
            let v = block_tex_index / 16;

            for i in 0..6 {
                vertices[vertex_count] = ChunkVertex {
                    x: x as u8 + UNIT_CUBE_FACES[6*k + i][0],
                    y: y as u8 + UNIT_CUBE_FACES[6*k + i][1],
                    z: z as u8 + UNIT_CUBE_FACES[6*k + i][2],
                    u: u + UV_FACES_OFFSETS[i][0],
                    v: v + UV_FACES_OFFSETS[i][1],
                };
                vertex_count += 1;
            }
        }
    }
    vertices.truncate(vertex_count);
    vertices.shrink_to_fit();
    vertices
}
