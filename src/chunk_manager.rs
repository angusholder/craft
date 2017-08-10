use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::iter::FromIterator;
use std::ops::Index;
use std::path::PathBuf;

use fnv::{ FnvHashMap, FnvHashSet };
use glium::{ Display,  VertexBuffer, Frame, Surface, Program };
use glium::{ DrawParameters, Depth, DepthTest };
use glium::uniforms::{ MagnifySamplerFilter, MinifySamplerFilter };
use glium::index::{ NoIndices, PrimitiveType };
use glium::texture::{ RawImage2d, SrgbTexture2d };
use image;

use block::Block;
use chunk::{ Chunk, EMPTY_CHUNK, CHUNK_SIDE_LENGTH };
use chunk_loader::ChunkLoader;
use chunk_generator::ChunkGenerator;
use chunk_mesher::ChunkMesher;
use math::*;
use chunk_mesher::ChunkVertex;
use player::Camera;
use utils::{ SETTINGS, ui };

pub struct ChunkManager {
    chunks: FnvHashMap<ChunkCoord, Box<Chunk>>,
    chunk_vbufs: FnvHashMap<ChunkCoord, VertexBuffer<ChunkVertex>>,
    chunk_mesher: ChunkMesher,
    chunk_loader: ChunkLoader,
    chunk_generator: ChunkGenerator,
    chunk_states: ChunkStates,
    texture: SrgbTexture2d,
    program: Program,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChunkState {
    Saved,
    Loading,

    NonExistent,
    Generating,
    Unmeshed,
    Meshing,

    Ready,
}

//#[derive(Clone, PartialEq, Eq, Debug)]
//pub enum ChunkStateE {
//    Saved,
//    Loading,
//
//    NonExistent,
//    Generating,
//    Unmeshed(Box<Chunk>),
//    Meshing(Box<Chunk>),
//
//    Ready(Box<Chunk>, VertexBuffer<ChunkVertex>),
//}

impl ChunkState {
    pub fn exists(self) -> bool {
        self != ChunkState::NonExistent
    }

    pub fn is_in_memory(self) -> bool {
        use self::ChunkState::*;
        match self {
            Ready | Meshing | Unmeshed => true,
            Saved | Loading | NonExistent | Generating => false,
        }
    }
}

pub struct ChunkStates {
    states: FnvHashMap<ChunkCoord, ChunkState>,
}

impl ChunkStates {
    pub fn new() -> ChunkStates {
        ChunkStates {
            states: FnvHashMap::default(),
        }
    }

    pub fn get(&self, coord: ChunkCoord) -> ChunkState {
        self.states.get(&coord).cloned().unwrap_or(ChunkState::NonExistent)
    }

    pub fn get_mut(&mut self, coord: ChunkCoord) -> &mut ChunkState {
        self.states.entry(coord).or_insert(ChunkState::NonExistent)
    }

    pub fn set(&mut self, coord: ChunkCoord, state: ChunkState) {
        self.states.insert(coord, state);
    }
}

impl ChunkManager {
    pub fn new(display: &Display, save_path: PathBuf) -> ChunkManager {
        let file = File::open("texture/texture.png").unwrap();
        let loaded_image = image::load(BufReader::new(file), image::ImageFormat::PNG).unwrap().to_rgba();
        let dimensions = loaded_image.dimensions();
        let raw_image = RawImage2d::from_raw_rgba(loaded_image.into_vec(), dimensions);
        let texture = SrgbTexture2d::new(display, raw_image).unwrap();

        let program = program!(display,
            150 => {
                vertex: include_str!("../shader/cube_150.glslv"),
                fragment: include_str!("../shader/cube_150.glslf")
            },
        ).unwrap();

        let (chunk_loader, chunk_states) = ChunkLoader::new(save_path);

        ChunkManager {
            chunks: FnvHashMap::default(),
            chunk_vbufs: FnvHashMap::default(),
            chunk_mesher: ChunkMesher::new(),
            chunk_generator: ChunkGenerator::new(),
            chunk_loader,
            chunk_states,
            texture,
            program,
        }
    }

    pub fn update_view(&mut self, view: Camera) {
        use self::ChunkState::*;
        let mut loaded_chunk_coords = FnvHashSet::from_iter(
            self.chunk_states.states.iter()
                .filter(|&(_, state)| state.is_in_memory())
                .map(|(coord, _)| *coord)
        );

        for coord in view.chunks_in_range() {
            let state = self.chunk_states.get_mut(coord);
            match *state {
                NonExistent => {
                    self.chunk_generator.start_generate(coord);
                    *state = ChunkState::Generating;
                }
                Saved => {
                    self.chunk_loader.enqueue_load(coord);
                    *state = ChunkState::Loading;
                }
                Unmeshed => {
                    self.chunk_mesher.start_meshing(coord, self.chunks.get(&coord).unwrap());
                    *state = ChunkState::Meshing;
                }
                _ => {}
            }
            let was_in_memory = loaded_chunk_coords.remove(&coord);
            assert!(was_in_memory == state.is_in_memory());
        }

        for out_of_range_coord in loaded_chunk_coords {
            let state = self.chunk_states.get_mut(out_of_range_coord);

            match *state {
                Ready => {
                    self.chunk_vbufs.remove(&out_of_range_coord).unwrap();
                }
                Unmeshed | Meshing => {}
                Saved | Loading | NonExistent | Generating => unreachable!(),
            }

            let chunk = self.chunks.remove(&out_of_range_coord).unwrap();
            self.chunk_loader.enqueue_unload(out_of_range_coord, chunk);
            *state = ChunkState::Saved;
        }
    }

    pub fn tick(&mut self, display: &Display, view: Camera) {
        use self::ChunkState::*;

        for (coord, chunk) in self.chunk_loader.iter_loaded() {
            let state = self.chunk_states.get_mut(coord);
            match *state {
                Saved | Ready | Unmeshed | Meshing | Generating | NonExistent => unreachable!(),
                Loading => {
                    self.chunks.insert(coord, chunk);
                    *state = ChunkState::Unmeshed;
                }
            }
        }

        for (coord, chunk) in self.chunk_generator.iter_generated() {
            let state = self.chunk_states.get_mut(coord);
            match *state {
                Saved | Ready | Unmeshed | Meshing | Loading | NonExistent => unreachable!(),
                Generating => {
                    self.chunks.insert(coord, chunk);
                    *state = ChunkState::Unmeshed;
                }
            }
        }

        self.update_view(view);

        for (coord, mesh) in self.chunk_mesher.iter_meshed() {
            let state = self.chunk_states.get_mut(coord);
            match *state {
                Ready | Unmeshed => unreachable!(),
                Meshing => {
                    let vbuf = VertexBuffer::new(display, &mesh).unwrap();
                    self.chunk_vbufs.insert(coord, vbuf);
                    *state = ChunkState::Ready;
                }

                // The mesh is useless if we've been evicted from memory in the meantime.
                // Throw it away.
                Saved | Loading | NonExistent | Generating => continue,
            }
        }

        ui.window(im_str!("Chunk States")).build(|| {
            let states = [ Saved, Loading, NonExistent, Generating, Unmeshed, Meshing, Ready ];
            for state in states.iter().cloned() {
                let state_count = self.chunk_states.states.values().filter(|&&s| s == state).count();
                ui.text(im_str!("{:?}: {}", state, state_count));
            }
        })
    }

    pub fn render(&mut self, frame: &mut Frame, screen_from_world: &Matrix4<f32>) {
        let draw_params = DrawParameters {
            depth: Depth {
                test: DepthTest::IfLess,
                write: true,
                ..Depth::default()
            },

            ..DrawParameters::default()
        };

        //println!("Rendering {} chunks", self.chunk_vbufs.len());

        let texture = self.texture.sampled()
            .minify_filter(MinifySamplerFilter::Nearest)
            .magnify_filter(MagnifySamplerFilter::Nearest);
        for (chunk_coord, vbuf) in self.chunk_vbufs.iter() {
            let uniforms = uniform! {
                uWorldToScreen: Into::<[[f32; 4]; 4]>::into(*screen_from_world),
                uChunkOffset: *chunk_coord,
                tBlocks: texture,
            };

            frame.draw(vbuf, NoIndices(PrimitiveType::TrianglesList), &self.program, &uniforms, &draw_params).unwrap();
        }
    }

    pub fn get_chunk(&mut self, coord: ChunkCoord) -> &Chunk {
        if let Some(chunk) = self.chunks.get(&coord) {
            chunk
        } else {
            &EMPTY_CHUNK
        }
    }

    pub fn get_chunk_mut(&mut self, coord: ChunkCoord) -> Option<&mut Chunk> {
        if let Some(chunk) = self.chunks.get_mut(&coord) {
            Some(chunk)
        } else {
            None
        }
    }

    pub fn get_block(&mut self, coord: Coord) -> Block {
        let chunk_coord = ChunkCoord::from_world_pos(coord);
        let chunk = self.get_chunk(chunk_coord);
        chunk.get(Coord::new(coord.x % (CHUNK_SIDE_LENGTH as i32), coord.y, coord.z % (CHUNK_SIDE_LENGTH as i32)))
    }

    pub fn get_chunk_state(&self, coord: ChunkCoord) -> ChunkState {
        self.chunk_states.get(coord)
    }

    // pub fn make_cache<'a>(&'a mut self, coord: Coord) -> ChunkCache<'a> {

    // }
}

impl Drop for ChunkManager {
    fn drop(&mut self) {
        for (coord, chunk) in self.chunks.drain() {
            self.chunk_loader.enqueue_unload(coord, chunk);
        }
    }
}

//pub struct ChunkCache<'a> {
//    chunk_manager: &'a mut ChunkManager,
//    chunks: Vec<Box<Chunk>>,
//    offset_x: i32,
//    offset_z: i32,
//    length_x: i32,
//    length_z: i32,
//}
//
//impl<'a> ChunkCache <'a> {
//
//}
