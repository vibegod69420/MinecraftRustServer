use crate::packing::BlockPos;
use crate::random::RandomSource;
use std::any::Any;
use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    pub const VALUES: [Direction; 6] = [
        Direction::Down,
        Direction::Up,
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ];

    #[inline]
    pub fn step_x(self) -> i32 {
        match self {
            Direction::West => -1,
            Direction::East => 1,
            _ => 0,
        }
    }

    #[inline]
    pub fn step_y(self) -> i32 {
        match self {
            Direction::Down => -1,
            Direction::Up => 1,
            _ => 0,
        }
    }

    #[inline]
    pub fn step_z(self) -> i32 {
        match self {
            Direction::North => -1,
            Direction::South => 1,
            _ => 0,
        }
    }

    #[inline]
    pub fn axis(self) -> Axis {
        match self {
            Direction::Down | Direction::Up => Axis::Y,
            Direction::North | Direction::South => Axis::Z,
            Direction::West | Direction::East => Axis::X,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    #[inline]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn at_center_of(pos: BlockPos) -> Self {
        let l = pos.as_long();
        Self {
            x: BlockPos::get_x(l) as f64 + 0.5,
            y: BlockPos::get_y(l) as f64 + 0.5,
            z: BlockPos::get_z(l) as f64 + 0.5,
        }
    }

    #[inline]
    pub fn add(self, x: f64, y: f64, z: f64) -> Self {
        Self {
            x: self.x + x,
            y: self.y + y,
            z: self.z + z,
        }
    }
}

pub trait ParticleOptions: Any + Debug {}
impl<T: Any + Debug> ParticleOptions for T {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockState {
    pub is_air: bool,
    pub shape_max_y_bits: u64,
}

impl BlockState {
    pub fn new(is_air: bool, shape_max_y: f64) -> Self {
        Self {
            is_air,
            shape_max_y_bits: shape_max_y.to_bits(),
        }
    }

    #[inline]
    pub fn shape_max_y(self) -> f64 {
        f64::from_bits(self.shape_max_y_bits)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockParticleKind {
    DustPillar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockParticleOption {
    pub kind: BlockParticleKind,
    pub block_state: BlockState,
}

pub trait IntProvider {
    fn sample(&self, random: &mut dyn RandomSource) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UniformInt {
    pub min: i32,
    pub max: i32,
}

impl UniformInt {
    #[inline]
    pub const fn of(min: i32, max: i32) -> Self {
        Self { min, max }
    }
}

impl IntProvider for UniformInt {
    fn sample(&self, random: &mut dyn RandomSource) -> i32 {
        if self.min >= self.max {
            self.min
        } else {
            let bound = self.max.wrapping_sub(self.min).wrapping_add(1);
            random.next_int_bound(bound).wrapping_add(self.min)
        }
    }
}

pub trait Level {
    fn get_random(&mut self) -> &mut dyn RandomSource;
    fn add_particle(
        &mut self,
        particle: &dyn ParticleOptions,
        x: f64,
        y: f64,
        z: f64,
        dx: f64,
        dy: f64,
        dz: f64,
    );
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
}

fn mth_next_double(random: &mut dyn RandomSource, min: f64, max: f64) -> f64 {
    if min >= max {
        min
    } else {
        random.next_double() * (max - min) + min
    }
}

fn block_x(pos: BlockPos) -> i32 {
    BlockPos::get_x(pos.as_long())
}

fn block_y(pos: BlockPos) -> i32 {
    BlockPos::get_y(pos.as_long())
}

fn block_z(pos: BlockPos) -> i32 {
    BlockPos::get_z(pos.as_long())
}

fn below(pos: BlockPos) -> BlockPos {
    let l = pos.as_long();
    BlockPos::of(BlockPos::as_long_coords(
        BlockPos::get_x(l),
        BlockPos::get_y(l).wrapping_sub(1),
        BlockPos::get_z(l),
    ))
}

// JAVA: net.minecraft.util.ParticleUtils#getRandomSpeedRanges
fn get_random_speed_ranges(random: &mut dyn RandomSource) -> Vec3 {
    Vec3::new(
        mth_next_double(random, -0.5, 0.5),
        mth_next_double(random, -0.5, 0.5),
        mth_next_double(random, -0.5, 0.5),
    )
}

// JAVA: net.minecraft.util.ParticleUtils#spawnParticlesOnBlockFaces
pub fn spawn_particles_on_block_faces(
    level: &mut dyn Level,
    pos: BlockPos,
    particle: &dyn ParticleOptions,
    count: &dyn IntProvider,
) {
    for dir in Direction::VALUES {
        spawn_particles_on_block_face(
            level,
            pos,
            particle,
            count,
            dir,
            &mut |r: &mut dyn RandomSource| get_random_speed_ranges(r),
            0.55,
        );
    }
}

// JAVA: net.minecraft.util.ParticleUtils#spawnParticlesOnBlockFace
pub fn spawn_particles_on_block_face(
    level: &mut dyn Level,
    pos: BlockPos,
    particle: &dyn ParticleOptions,
    count: &dyn IntProvider,
    direction: Direction,
    speed_supplier: &mut dyn FnMut(&mut dyn RandomSource) -> Vec3,
    spread: f64,
) {
    let n = count.sample(level.get_random());
    let mut i = 0;
    while i < n {
        let speed = speed_supplier(level.get_random());
        spawn_particle_on_face(level, pos, direction, particle, speed, spread);
        i += 1;
    }
}

// JAVA: net.minecraft.util.ParticleUtils#spawnParticlesAlongAxis
pub fn spawn_particles_along_axis(
    axis: Axis,
    level: &mut dyn Level,
    pos: BlockPos,
    spread: f64,
    particle: &dyn ParticleOptions,
    count: UniformInt,
) {
    let center = Vec3::at_center_of(pos);
    let is_x = axis == Axis::X;
    let is_y = axis == Axis::Y;
    let is_z = axis == Axis::Z;
    let n = count.sample(level.get_random());
    let mut i = 0;
    while i < n {
        let x = center.x
            + mth_next_double(level.get_random(), -1.0, 1.0)
                * if is_x { 0.5 } else { spread };
        let y = center.y
            + mth_next_double(level.get_random(), -1.0, 1.0)
                * if is_y { 0.5 } else { spread };
        let z = center.z
            + mth_next_double(level.get_random(), -1.0, 1.0)
                * if is_z { 0.5 } else { spread };
        let dx = if is_x {
            mth_next_double(level.get_random(), -1.0, 1.0)
        } else {
            0.0
        };
        let dy = if is_y {
            mth_next_double(level.get_random(), -1.0, 1.0)
        } else {
            0.0
        };
        let dz = if is_z {
            mth_next_double(level.get_random(), -1.0, 1.0)
        } else {
            0.0
        };
        level.add_particle(particle, x, y, z, dx, dy, dz);
        i += 1;
    }
}

// JAVA: net.minecraft.util.ParticleUtils#spawnParticleOnFace
pub fn spawn_particle_on_face(
    level: &mut dyn Level,
    pos: BlockPos,
    direction: Direction,
    particle: &dyn ParticleOptions,
    speed: Vec3,
    spread: f64,
) {
    let center = Vec3::at_center_of(pos);
    let step_x = direction.step_x();
    let step_y = direction.step_y();
    let step_z = direction.step_z();
    let x = center.x
        + if step_x == 0 {
            mth_next_double(level.get_random(), -0.5, 0.5)
        } else {
            step_x as f64 * spread
        };
    let y = center.y
        + if step_y == 0 {
            mth_next_double(level.get_random(), -0.5, 0.5)
        } else {
            step_y as f64 * spread
        };
    let z = center.z
        + if step_z == 0 {
            mth_next_double(level.get_random(), -0.5, 0.5)
        } else {
            step_z as f64 * spread
        };
    let dx = if step_x == 0 { speed.x } else { 0.0 };
    let dy = if step_y == 0 { speed.y } else { 0.0 };
    let dz = if step_z == 0 { speed.z } else { 0.0 };
    level.add_particle(particle, x, y, z, dx, dy, dz);
}

// JAVA: net.minecraft.util.ParticleUtils#spawnParticleBelow
pub fn spawn_particle_below(
    level: &mut dyn Level,
    pos: BlockPos,
    random: &mut dyn RandomSource,
    particle: &dyn ParticleOptions,
) {
    let x = block_x(pos) as f64 + random.next_double();
    let y = block_y(pos) as f64 - 0.05;
    let z = block_z(pos) as f64 + random.next_double();
    level.add_particle(particle, x, y, z, 0.0, 0.0, 0.0);
}

// JAVA: net.minecraft.util.ParticleUtils#spawnParticleInBlock
pub fn spawn_particle_in_block(
    level: &mut dyn Level,
    pos: BlockPos,
    count: i32,
    particle: &dyn ParticleOptions,
) {
    let state = level.get_block_state(pos);
    let max_y = if state.is_air {
        1.0
    } else {
        state.shape_max_y()
    };
    spawn_particles(level, pos, count, 0.5, max_y, true, particle);
}

// JAVA: net.minecraft.util.ParticleUtils#spawnParticles
pub fn spawn_particles(
    level: &mut dyn Level,
    pos: BlockPos,
    count: i32,
    spread_xz: f64,
    max_y: f64,
    always_spawn: bool,
    particle: &dyn ParticleOptions,
) {
    let mut i = 0;
    while i < count {
        let random = level.get_random();
        let dx = random.next_gaussian() * 0.02;
        let dy = random.next_gaussian() * 0.02;
        let dz = random.next_gaussian() * 0.02;
        let offset = 0.5 - spread_xz;
        let x = block_x(pos) as f64 + offset + random.next_double() * spread_xz * 2.0;
        let y = block_y(pos) as f64 + random.next_double() * max_y;
        let z = block_z(pos) as f64 + offset + random.next_double() * spread_xz * 2.0;

        let should_spawn =
            always_spawn || !level.get_block_state(below(BlockPos::containing(x, y, z))).is_air;
        if should_spawn {
            level.add_particle(particle, x, y, z, dx, dy, dz);
        }
        i += 1;
    }
}

// JAVA: net.minecraft.util.ParticleUtils#spawnSmashAttackParticles
pub fn spawn_smash_attack_particles(level: &mut dyn Level, pos: BlockPos, count: i32) {
    let center = Vec3::at_center_of(pos).add(0.0, 0.5, 0.0);
    let particle = BlockParticleOption {
        kind: BlockParticleKind::DustPillar,
        block_state: level.get_block_state(pos),
    };

    let mut i = 0i32;
    while (i as f32) < (count as f32 / 3.0f32) {
        let random = level.get_random();
        let x = center.x + random.next_gaussian() / 2.0;
        let y = center.y;
        let z = center.z + random.next_gaussian() / 2.0;
        let dx = random.next_gaussian() * (0.2f32 as f64);
        let dy = random.next_gaussian() * (0.2f32 as f64);
        let dz = random.next_gaussian() * (0.2f32 as f64);
        level.add_particle(&particle, x, y, z, dx, dy, dz);
        i += 1;
    }

    let mut i = 0i32;
    while (i as f32) < (count as f32 / 1.5f32) {
        let random = level.get_random();
        let x = center.x + 3.5 * (i as f64).cos() + random.next_gaussian() / 2.0;
        let y = center.y;
        let z = center.z + 3.5 * (i as f64).sin() + random.next_gaussian() / 2.0;
        let dx = random.next_gaussian() * (0.05f32 as f64);
        let dy = random.next_gaussian() * (0.05f32 as f64);
        let dz = random.next_gaussian() * (0.05f32 as f64);
        level.add_particle(&particle, x, y, z, dx, dy, dz);
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::{RandomSource, XoroshiroRandomSource};

    #[derive(Debug, Clone)]
    struct RecordedParticle {
        name: String,
        x: f64,
        y: f64,
        z: f64,
        dx: f64,
        dy: f64,
        dz: f64,
    }

    struct MockLevel {
        random: XoroshiroRandomSource,
        particles: Vec<RecordedParticle>,
        block_state: BlockState,
    }

    impl MockLevel {
        fn new(seed: i64) -> Self {
            Self {
                random: XoroshiroRandomSource::new(seed),
                particles: Vec::new(),
                block_state: BlockState::new(false, 1.0),
            }
        }

        fn with_block_state(mut self, state: BlockState) -> Self {
            self.block_state = state;
            self
        }
    }

    impl Level for MockLevel {
        fn get_random(&mut self) -> &mut dyn RandomSource {
            &mut self.random
        }

        fn add_particle(
            &mut self,
            particle: &dyn ParticleOptions,
            x: f64,
            y: f64,
            z: f64,
            dx: f64,
            dy: f64,
            dz: f64,
        ) {
            self.particles.push(RecordedParticle {
                name: format!("{:?}", particle),
                x,
                y,
                z,
                dx,
                dy,
                dz,
            });
        }

        fn get_block_state(&self, _pos: BlockPos) -> BlockState {
            self.block_state
        }
    }

    struct FixedIntProvider(i32);
    impl IntProvider for FixedIntProvider {
        fn sample(&self, _random: &mut dyn RandomSource) -> i32 {
            self.0
        }
    }

    #[test]
    fn spawn_particle_below_uses_separate_random_and_zero_velocity() {
        let mut level = MockLevel::new(0);
        let mut rng = XoroshiroRandomSource::new(42);
        let pos = BlockPos::of(BlockPos::as_long_coords(3, 10, -5));
        let particle = "test_particle";
        spawn_particle_below(&mut level, pos, &mut rng, &particle);

        let mut expected_rng = XoroshiroRandomSource::new(42);
        let ex: f64 = 3.0 + expected_rng.next_double();
        let ey: f64 = 10.0 - 0.05;
        let ez: f64 = -5.0 + expected_rng.next_double();

        assert_eq!(level.particles.len(), 1);
        let p = &level.particles[0];
        assert_eq!(p.x.to_bits(), ex.to_bits());
        assert_eq!(p.y.to_bits(), ey.to_bits());
        assert_eq!(p.z.to_bits(), ez.to_bits());
        assert_eq!(p.dx.to_bits(), 0.0f64.to_bits());
        assert_eq!(p.dy.to_bits(), 0.0f64.to_bits());
        assert_eq!(p.dz.to_bits(), 0.0f64.to_bits());
    }

    #[test]
    fn spawn_particle_on_face_east_gates_speed_and_uses_spread() {
        let mut level = MockLevel::new(1);
        let pos = BlockPos::of(BlockPos::as_long_coords(0, 0, 0));
        let speed = Vec3::new(0.1, 0.2, 0.3);
        let particle = "p";
        spawn_particle_on_face(&mut level, pos, Direction::East, &particle, speed, 0.55);

        let mut rng = XoroshiroRandomSource::new(1);
        let ex: f64 = 0.5 + 1.0 * 0.55;
        let ey: f64 = 0.5 + mth_next_double(&mut rng, -0.5, 0.5);
        let ez: f64 = 0.5 + mth_next_double(&mut rng, -0.5, 0.5);

        assert_eq!(level.particles.len(), 1);
        let p = &level.particles[0];
        assert_eq!(p.x.to_bits(), ex.to_bits());
        assert_eq!(p.y.to_bits(), ey.to_bits());
        assert_eq!(p.z.to_bits(), ez.to_bits());
        assert_eq!(p.dx.to_bits(), 0.0f64.to_bits());
        assert_eq!(p.dy.to_bits(), 0.2f64.to_bits());
        assert_eq!(p.dz.to_bits(), 0.3f64.to_bits());
    }

    #[test]
    fn spawn_particles_along_axis_respects_axis_and_uniform_int_edges() {
        let mut level = MockLevel::new(7);
        let pos = BlockPos::of(BlockPos::as_long_coords(0, 0, 0));
        let particle = "p";
        spawn_particles_along_axis(Axis::Y, &mut level, pos, 2.0, &particle, UniformInt::of(2, 2));
        assert_eq!(level.particles.len(), 2);
        for p in &level.particles {
            assert_eq!(p.dx.to_bits(), 0.0f64.to_bits());
            assert_eq!(p.dz.to_bits(), 0.0f64.to_bits());
            assert!(p.dy >= -1.0 && p.dy <= 1.0);
        }

        let mut rng = XoroshiroRandomSource::new(0);
        assert_eq!(UniformInt::of(5, 3).sample(&mut rng), 5);
        assert_eq!(UniformInt::of(-2, -2).sample(&mut rng), -2);
    }

    #[test]
    fn spawn_particles_respects_always_spawn_and_air_check() {
        let mut air_level =
            MockLevel::new(123).with_block_state(BlockState::new(true, 0.0));
        let pos = BlockPos::of(BlockPos::as_long_coords(0, 5, 0));
        let particle = "p";
        spawn_particles(&mut air_level, pos, 10, 0.5, 1.0, false, &particle);
        assert_eq!(air_level.particles.len(), 0);

        let mut forced_level =
            MockLevel::new(123).with_block_state(BlockState::new(true, 0.0));
        spawn_particles(&mut forced_level, pos, 3, 0.5, 1.0, true, &particle);
        assert_eq!(forced_level.particles.len(), 3);

        let mut neg_level = MockLevel::new(1);
        spawn_particles(&mut neg_level, pos, -1, 0.5, 1.0, true, &particle);
        assert_eq!(neg_level.particles.len(), 0);
    }

    #[test]
    fn spawn_smash_attack_particles_counts_and_negative_count() {
        let mut level = MockLevel::new(99);
        let pos = BlockPos::of(BlockPos::as_long_coords(0, 0, 0));
        spawn_smash_attack_particles(&mut level, pos, 3);
        assert_eq!(level.particles.len(), 3); // 1 + 2
        assert_eq!(level.particles[0].y.to_bits(), 1.0f64.to_bits());

        let mut neg_level = MockLevel::new(0);
        spawn_smash_attack_particles(&mut neg_level, pos, -3);
        assert_eq!(neg_level.particles.len(), 0);
    }

    #[test]
    fn spawn_particles_on_block_faces_hits_all_directions() {
        let mut level = MockLevel::new(5);
        let pos = BlockPos::of(BlockPos::as_long_coords(0, 64, 0));
        let particle = "p";
        let count = FixedIntProvider(1);
        spawn_particles_on_block_faces(&mut level, pos, &particle, &count);
        assert_eq!(level.particles.len(), 6);

        // Direction order: Down, Up, North, South, West, East
        let center_y: f64 = 64.5;
        assert_eq!(level.particles[0].y.to_bits(), (center_y - 0.55f64).to_bits()); // Down
        assert_eq!(level.particles[1].y.to_bits(), (center_y + 0.55f64).to_bits()); // Up
    }
}
