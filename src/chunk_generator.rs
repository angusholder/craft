use std::sync::mpsc;
use std::thread;

use block::{ Block, BlockType };
use chunk::{ Chunk, CHUNK_SIDE_LENGTH };
use math::*;

pub struct ChunkGenerator {
    thread_handle: thread::JoinHandle<()>,
    tx_req: mpsc::Sender<ChunkCoord>,
    rx_resp: mpsc::Receiver<Response>,
}

fn fill_layer(chunk: &mut Chunk, y: i32, ty: BlockType) {
    for x in 0..CHUNK_SIDE_LENGTH as i32 {
        for z in 0..CHUNK_SIDE_LENGTH as i32 {
            chunk.set(Coord {x, y, z}, Block { ty } )
        }
    }
}

impl ChunkGenerator {
    pub fn new() -> ChunkGenerator {
        let (tx_req, rx_req) = mpsc::channel();
        let (tx_resp, rx_resp) = mpsc::channel();
        let thread_handle = thread::spawn(move || {
            for coord in rx_req {
                let mut chunk = Chunk::new();

                for y in 0..16 { fill_layer(&mut chunk, y, BlockType::Stone) }
                for y in 16..40 { fill_layer(&mut chunk, y, BlockType::Stone) }
                fill_layer(&mut chunk, 40, BlockType::Grass);

                tx_resp.send((coord, chunk));
            }
        });

        ChunkGenerator {
            tx_req, rx_resp, thread_handle
        }
    }

    pub fn start_generate(&mut self, coord: ChunkCoord) {
        self.tx_req.send(coord).unwrap();
    }

    pub fn iter_generated(&mut self) -> ResponseIter {
        ResponseIter(self.rx_resp.try_iter())
    }
}

type Response = (ChunkCoord, Box<Chunk>);

pub struct ResponseIter<'a>(mpsc::TryIter<'a, Response>);

impl<'a> Iterator for ResponseIter<'a> {
    type Item = Response;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
