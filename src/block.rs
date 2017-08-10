#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Block {
    pub ty: BlockType,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Air,
    Dirt,
    Grass,
    Stone,
    Cobblestone,
    Wood,
    Log,
    Bedrock,
    Sand,
    Gravel,
    GoldOre,
    IronOre,
    CoalOre,
    Leaf,
    Sponge,
    Sandstone,
}

impl Default for BlockType {
    fn default() -> BlockType {
        BlockType::Air
    }
}

impl From<u8> for BlockType {
    fn from(b: u8) -> BlockType {
        use self::BlockType::*;
        match b {
            0 => Air,
            1 => Dirt,
            2 => Grass,
            3 => Stone,
            _ => unreachable!(),
        }
    }
}

impl Block {
    pub fn is_air(self) -> bool {
        self.ty == BlockType::Air
    }
    /*pub fn has_solid_top_surface(&self) -> bool {

    }*/
}

impl From<u8> for Block {
    fn from(b: u8) -> Block {
        Block {
            ty: BlockType::from(b)
        }
    }
}
