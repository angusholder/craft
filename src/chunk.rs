use block::{ Block, BlockType };
use math::*;

pub static EMPTY_CHUNK: Chunk = Chunk {
    blocks: [Block { ty: BlockType::Air }; CHUNK_BLOCK_COUNT],
};

// #[derive(Clone)]
pub struct Chunk {
    blocks: [Block; CHUNK_BLOCK_COUNT],
}

impl Clone for Chunk {
    fn clone(&self) -> Chunk {
        Chunk {
            blocks: self.blocks
        }
    }
}

fn index_to_coord(i: usize) -> Coord {
    Coord {
        y: (i & 0x7F) as i32,
        z: (i >> 7 & 0xF) as i32,
        x: (i >> 11 & 0xF) as i32,
    }
}

pub const CHUNK_SIDE_LENGTH_BITS: usize = 4;
pub const CHUNK_SIDE_LENGTH: usize = 1 << CHUNK_SIDE_LENGTH_BITS;
pub const CHUNK_SIDE_LENGTH_MASK: usize = CHUNK_SIDE_LENGTH - 1;
pub const CHUNK_SECTIONS_HIGH_BITS: usize = 3; // 8 sections of height
pub const CHUNK_SECTIONS_HIGH: usize = 1 << CHUNK_SECTIONS_HIGH_BITS;
pub const CHUNK_SECTIONS_HIGH_MASK: usize = CHUNK_SECTIONS_HIGH - 1;
pub const CHUNK_HEIGHT_BITS: usize = 7;
pub const CHUNK_HEIGHT: usize = 1 << CHUNK_HEIGHT_BITS;
pub const CHUNK_HEIGHT_MASK: usize = CHUNK_HEIGHT - 1;
pub const CHUNK_BLOCK_COUNT: usize = CHUNK_SIDE_LENGTH * CHUNK_SIDE_LENGTH * CHUNK_HEIGHT;

pub const WORLD_HEIGHT: usize = CHUNK_HEIGHT;

pub const SECTION_SIZE_BITS: usize = CHUNK_SIDE_LENGTH_BITS;
pub const SECTION_SIZE: usize = CHUNK_SIDE_LENGTH;
pub const SECTION_SIZE_MASK: usize = CHUNK_SIDE_LENGTH_MASK;
pub const SECTION_BLOCK_COUNT: usize = SECTION_SIZE * SECTION_SIZE * SECTION_SIZE;

//bit_consts! {}

fn coord_to_index(coord: Coord) -> usize {
    let mut x = coord.x as usize;
    let mut y = coord.y as usize;
    let mut z = coord.z as usize;

    debug_assert!(y < WORLD_HEIGHT);
    debug_assert!(x < CHUNK_SIDE_LENGTH);
    debug_assert!(z < CHUNK_SIDE_LENGTH);

    x &= CHUNK_SIDE_LENGTH - 1;
    y &= WORLD_HEIGHT - 1;
    z &= CHUNK_SIDE_LENGTH - 1;

    y + (z * WORLD_HEIGHT) + (x * WORLD_HEIGHT * CHUNK_SIDE_LENGTH)
}

impl Chunk {
    pub fn new() -> Box<Chunk> {
        Box::new(EMPTY_CHUNK.clone())
    }

    /// `coord.x` and `coord.z` must be in the range `0..CHUNK_SIDE_LENGTH`, this is asserted in
    /// debug and wrapped in release. `coord.y` may take any value, but air blocks are returned
    /// when it is outside the range `0..WORLD_HEIGHT`.
    pub fn get(&self, coord: Coord) -> Block {
        if Self::is_valid_coord(coord) {
            self.blocks[coord_to_index(coord)]
        } else {
            Block { ty: BlockType::Air }
        }
    }

    /// `coord.x` and `coord.z` must be in the range `0..CHUNK_SIDE_LENGTH`, this is asserted in
    /// debug and wrapped in release. `coord.y` may take any value, but assignments are ignored
    /// when it is outside the range `0..WORLD_HEIGHT`.
    pub fn set(&mut self, coord: Coord, block: Block) {
        if Self::is_valid_coord(coord) {
            self.blocks[coord_to_index(coord)] = block;
        }
    }

    pub fn is_valid_coord(coord: Coord) -> bool {
        coord.y >= 0 && coord.y < WORLD_HEIGHT as i32
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.blocks.iter().map(|&b| b.ty as u8).collect()
    }

    pub fn from_bytes(bytes: &[u8]) -> Box<Chunk> {
        assert!(bytes.len() == CHUNK_BLOCK_COUNT);
        let mut chunk = Chunk::new();
        for (i, block) in chunk.blocks.iter_mut().enumerate() {
            *block = Block::from(bytes[i]);
        }
        chunk
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=(Coord, Block)> + 'a {
        self.blocks
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, b)| (index_to_coord(i), b))
    }

    pub fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item=(Coord, &mut Block)> + 'a {
        self.blocks
            .iter_mut()
            .enumerate()
            .map(|(i, b)| (index_to_coord(i), b))
    }
}
