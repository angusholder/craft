use math::*;
use block::Block;
use chunk::CHUNK_SIDE_LENGTH;
use chunk_manager::ChunkManager;
use utils::*;

pub struct Player {
    pub camera: Camera,
}

impl Player {
    pub fn new() -> Player {
        Player {
            camera: Camera {
                pos: Point3::new(0.0, 45.0, 0.0),
                h_angle: Deg(0.0),
                v_angle: Deg(0.0),
            }
        }
    }

    pub fn move_dir(&mut self, dir: Direction) {
        let (left, up, forward) = self.camera.directions();
        let diff = match dir {
            Direction::Up => up,
            Direction::Down => -up,
            Direction::Left => left,
            Direction::Right => -left,
            Direction::Forward => forward,
            Direction::Backward => -forward,
        };
        self.camera.pos += diff * SETTINGS.move_speed;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub pos: Point3<f32>,
    h_angle: Deg<f32>,
    v_angle: Deg<f32>,
}

// Iterates over ChunkCoords inside [min, max]
pub struct ChunksInRange {
    min_x: i32,

    cur_x: i32,
    cur_z: i32,

    max_x: i32,
    max_z: i32,
}

impl Iterator for ChunksInRange {
    type Item = ChunkCoord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_z <= self.max_z {
            let next = ChunkCoord::new(self.cur_x, self.cur_z);
            self.cur_x += 1;
            if self.cur_x > self.max_x {
                self.cur_x = self.min_x;
                self.cur_z += 1;
                // if self.cur_z == self.max_z: next iteration yields None
            }
            Some(next)
        } else {
            None
        }
    }
}

impl Camera {
    pub fn chunks_in_range(&self) -> ChunksInRange {
        let range = ChunkCoord::new(SETTINGS.chunk_render_distance, SETTINGS.chunk_render_distance);
        let center_chunk = ChunkCoord::from_world_pos(self.pos.cast());
        let min_chunk = center_chunk - range;
        let max_chunk = center_chunk + range;

        ChunksInRange {
            min_x: min_chunk.x,
            max_x: max_chunk.x,
            max_z: max_chunk.z,
            cur_x: min_chunk.x,
            cur_z: min_chunk.z
        }
    }

    pub fn view(&self) -> Vector3 {
        self.directions().2
    }

    pub fn directions(&self) -> (Vector3, Vector3, Vector3) {
        let mut up = Vector3::unit_y();
        let view = Quaternion::from_axis_angle(up, self.h_angle).rotate_vector(Vector3::unit_x());
        let left = up.cross(view);
        let forward = Quaternion::from_axis_angle(left, self.v_angle).rotate_vector(view);
        (left, up, forward)
    }

    pub fn rotate_by(&mut self, dx: f32, dy: f32) {
        const MIN_V_ANGLE: Deg<f32> = Deg(-89.9);
        const MAX_V_ANGLE: Deg<f32> = Deg(89.9);

        self.h_angle += Deg(-dx * SETTINGS.mouse_sensitivity);
        self.v_angle += Deg(-dy * SETTINGS.mouse_sensitivity);

        if self.v_angle > MAX_V_ANGLE {
            self.v_angle = MAX_V_ANGLE;
        } else if self.v_angle < MIN_V_ANGLE {
            self.v_angle = MIN_V_ANGLE;
        }
    }
}

pub fn raytrace(chunks: &mut ChunkManager, pos: Point3<f32>, dir: Vector3, max_distance: f32) -> Option<(Coord, Block)> {
    const RESOLUTION: f32 = 0.01;
    let diff = dir.normalize() * RESOLUTION;
    let steps = (max_distance / RESOLUTION) as usize;

    let mut cur = pos;
    let mut cur_coord: Point3<i32> = cur.cast();

    for _ in 0..steps {
        cur += diff;
        let next_coord: Point3<i32> = cur.cast();
        if next_coord != cur_coord {
            let block = chunks.get_block(next_coord);
            if !block.is_air() {
                return Some((next_coord, block));
            }
            cur_coord = next_coord;
        }
    }

    None
}
