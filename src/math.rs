// Conversions
pub use cgmath::conv::*;

// Traits
pub use cgmath::{
    Angle, One, Zero,
    EuclideanSpace, InnerSpace, MetricSpace, VectorSpace,
    Rotation, Rotation2, Rotation3,
    Transform, Transform2, Transform3,
    SquareMatrix,
    Quaternion
};

// Constructors
pub use cgmath::{
    dot, frustum, ortho, perspective,
};

pub const VECTOR3_UP: Vector3 = Vector3 { x: 0.0, y: 1.0, z: 0.0 };
pub const VECTOR3_DOWN: Vector3 = Vector3 { x: 0.0, y: -1.0, z: 0.0 };
pub const VECTOR3_LEFT: Vector3 = Vector3 { x: -1.0, y: 0.0, z: 0.0 };
pub const VECTOR3_RIGHT: Vector3 = Vector3 { x: 1.0, y: 0.0, z: 0.0 };
pub const VECTOR3_FORWARD: Vector3 = Vector3 { x: 0.0, y: 0.0, z: -1.0 };
pub const VECTOR3_BACKWARD: Vector3 = Vector3 { x: 0.0, y: 0.0, z: 1.0 };

use cgmath;
pub type Coord = cgmath::Point3<i32>;
pub type Vector3<T=f32> = cgmath::Vector3<T>;
pub use cgmath::{ Deg, Rad, Matrix3, Matrix4, Point3, Point2 };

pub fn chunk_to_block() {}
pub fn block_to_chunk() {}

pub fn adjacent_side(coord: Coord, side: Side) -> Coord {
    match side {
        Side::Top => Coord::new(coord.x, coord.y + 1, coord.z),
        Side::Bottom => Coord::new(coord.x, coord.y - 1, coord.z),
        Side::Left => Coord::new(coord.x - 1, coord.y, coord.z),
        Side::Right => Coord::new(coord.x + 1, coord.y, coord.z),
        Side::Front => Coord::new(coord.x, coord.y, coord.z - 1),
        Side::Back => Coord::new(coord.x, coord.y, coord.z + 1),
    }
}

pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

impl Side {
    pub fn from_vector(point: Vector3<i32>) -> Option<Side> {
        match point {
            Vector3 { x: 0, y: 1, z: 0 } => Some(Side::Top),
            Vector3 { x: 0, y:-1, z: 0 } => Some(Side::Bottom),
            Vector3 { x:-1, y: 0, z: 0 } => Some(Side::Left),
            Vector3 { x: 1, y: 0, z: 0 } => Some(Side::Right),
            Vector3 { x: 0, y: 0, z: 1 } => Some(Side::Front),
            Vector3 { x: 0, y: 0, z:-1 } => Some(Side::Back),
            _ => None
        }
    }
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Forward,
    Backward,
}

pub fn point3_floor(point: Point3<f32>) -> Point3<i32> {
    Point3 {
        x: point.x.floor() as i32,
        y: point.y.floor() as i32,
        z: point.z.floor() as i32,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

impl ChunkCoord {
    pub fn new(x: i32, z: i32) -> ChunkCoord {
        ChunkCoord { x, z }
    }

    pub fn from_world_pos(v: Point3<i32>) -> ChunkCoord {
        use chunk::CHUNK_SIDE_LENGTH;
        ChunkCoord {
            x: v.x / (CHUNK_SIDE_LENGTH as i32),
            z: v.z / (CHUNK_SIDE_LENGTH as i32)
        }
    }
}

use glium::uniforms::{ AsUniformValue, UniformValue };

impl AsUniformValue for ChunkCoord {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::IntVec2([self.x, self.z])
    }
}

use std::ops::{ Add, Sub };
use std::fmt;

impl fmt::Display for ChunkCoord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.z)
    }
}

impl Add for ChunkCoord {
    type Output = ChunkCoord;

    fn add(self, rhs: Self) -> Self::Output {
        ChunkCoord::new(self.x + rhs.x, self.z + rhs.z)
    }
}

impl Sub for ChunkCoord {
    type Output = ChunkCoord;

    fn sub(self, rhs: Self) -> Self::Output {
        ChunkCoord::new(self.x - rhs.x, self.z - rhs.z)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }
}

use glium::vertex::{ Attribute, AttributeType };

unsafe impl Attribute for Color {
    fn get_type() -> AttributeType {
        AttributeType::U8U8U8U8
    }
}
