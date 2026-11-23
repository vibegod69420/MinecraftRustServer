use std::fmt::Debug;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn mutable(self) -> MutableBlockPos {
        MutableBlockPos {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }

    pub fn above(self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(1),
            z: self.z,
        }
    }

    pub fn offset(self, dx: i32, dy: i32, dz: i32) -> Self {
        Self {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    pub const fn new() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }

    pub fn set(&mut self, pos: BlockPos) -> &mut Self {
        self.x = pos.x;
        self.y = pos.y;
        self.z = pos.z;
        self
    }

    pub fn set_with_offset(&mut self, pos: &impl Pos3, dir: Direction) -> &mut Self {
        self.x = pos.x().wrapping_add(dir.step_x());
        self.y = pos.y().wrapping_add(dir.step_y());
        self.z = pos.z().wrapping_add(dir.step_z());
        self
    }

    pub fn move_dir(&mut self, dir: Direction) -> &mut Self {
        self.x = self.x.wrapping_add(dir.step_x());
        self.y = self.y.wrapping_add(dir.step_y());
        self.z = self.z.wrapping_add(dir.step_z());
        self
    }

    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> BlockPos {
        BlockPos {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }
}

pub trait Pos3 {
    fn x(&self) -> i32;
    fn y(&self) -> i32;
    fn z(&self) -> i32;

    fn as_block_pos(&self) -> BlockPos {
        BlockPos::new(self.x(), self.y(), self.z())
    }
}

impl Pos3 for BlockPos {
    fn x(&self) -> i32 {
        self.x
    }
    fn y(&self) -> i32 {
        self.y
    }
    fn z(&self) -> i32 {
        self.z
    }
}

impl Pos3 for MutableBlockPos {
    fn x(&self) -> i32 {
        self.x
    }
    fn y(&self) -> i32 {
        self.y
    }
    fn z(&self) -> i32 {
        self.z
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    pub const fn step_x(self) -> i32 {
        match self {
            Direction::West => -1,
            Direction::East => 1,
            _ => 0,
        }
    }

    pub const fn step_y(self) -> i32 {
        match self {
            Direction::Down => -1,
            Direction::Up => 1,
            _ => 0,
        }
    }

    pub const fn step_z(self) -> i32 {
        match self {
            Direction::North => -1,
            Direction::South => 1,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub id: u32,
}

impl BlockState {
    pub const BASALT: Self = Self { id: 1 };
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
    fn next_boolean(&mut self) -> bool;
}

pub trait LevelAccessor {
    fn is_empty_block(&self, pos: &impl Pos3) -> bool;
    fn is_outside_build_height(&self, pos: &impl Pos3) -> bool;
    fn set_block(&mut self, pos: &impl Pos3, state: BlockState, flags: i32) -> bool;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NoneFeatureConfiguration;

pub struct FeaturePlaceContext<'a, L: LevelAccessor + ?Sized, R: RandomSource + ?Sized> {
    pub origin: BlockPos,
    pub level: &'a mut L,
    pub random: &'a mut R,
}

#[inline]
fn mth_abs(v: i32) -> i32 {
    if v >= 0 {
        v
    } else {
        v.wrapping_neg()
    }
}

pub struct BasaltPillarFeature;

impl BasaltPillarFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.BasaltPillarFeature#<init>
    pub fn new(_config: NoneFeatureConfiguration) -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.BasaltPillarFeature#place
    pub fn place<L: LevelAccessor + ?Sized, R: RandomSource + ?Sized>(
        &self,
        ctx: &mut FeaturePlaceContext<'_, L, R>,
    ) -> bool {
        let origin = ctx.origin;
        let level = &mut *ctx.level;
        let random = &mut *ctx.random;

        if level.is_empty_block(&origin) && !level.is_empty_block(&origin.above()) {
            let mut pos = origin.mutable();
            let mut side_pos = origin.mutable();
            let mut hang_north = true;
            let mut hang_south = true;
            let mut hang_west = true;
            let mut hang_east = true;

            while level.is_empty_block(&pos) {
                if level.is_outside_build_height(&pos) {
                    return true;
                }

                level.set_block(&pos, BlockState::BASALT, 2);
                hang_north = hang_north
                    && self.place_hang_off(
                        level,
                        random,
                        side_pos.set_with_offset(&pos, Direction::North),
                    );
                hang_south = hang_south
                    && self.place_hang_off(
                        level,
                        random,
                        side_pos.set_with_offset(&pos, Direction::South),
                    );
                hang_west = hang_west
                    && self.place_hang_off(
                        level,
                        random,
                        side_pos.set_with_offset(&pos, Direction::West),
                    );
                hang_east = hang_east
                    && self.place_hang_off(
                        level,
                        random,
                        side_pos.set_with_offset(&pos, Direction::East),
                    );
                pos.move_dir(Direction::Down);
            }

            pos.move_dir(Direction::Up);
            self.place_base_hang_off(
                level,
                random,
                side_pos.set_with_offset(&pos, Direction::North),
            );
            self.place_base_hang_off(
                level,
                random,
                side_pos.set_with_offset(&pos, Direction::South),
            );
            self.place_base_hang_off(
                level,
                random,
                side_pos.set_with_offset(&pos, Direction::West),
            );
            self.place_base_hang_off(
                level,
                random,
                side_pos.set_with_offset(&pos, Direction::East),
            );
            pos.move_dir(Direction::Down);

            let mut cluster_pos = MutableBlockPos::new();
            let mut dx: i32 = -3;
            while dx < 4 {
                let mut dz: i32 = -3;
                while dz < 4 {
                    let dist = mth_abs(dx).wrapping_mul(mth_abs(dz));
                    if random.next_int(10) < 10i32.wrapping_sub(dist) {
                        cluster_pos.set(pos.offset(dx, 0, dz));
                        let mut depth: i32 = 3;

                        while level.is_empty_block(
                            side_pos.set_with_offset(&cluster_pos, Direction::Down),
                        ) {
                            cluster_pos.move_dir(Direction::Down);
                            depth = depth.wrapping_sub(1);
                            if depth <= 0 {
                                break;
                            }
                        }

                        if !level.is_empty_block(
                            side_pos.set_with_offset(&cluster_pos, Direction::Down),
                        ) {
                            level.set_block(&cluster_pos, BlockState::BASALT, 2);
                        }
                    }
                    dz = dz.wrapping_add(1);
                }
                dx = dx.wrapping_add(1);
            }

            true
        } else {
            false
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.BasaltPillarFeature#placeBaseHangOff
    fn place_base_hang_off<L: LevelAccessor + ?Sized, R: RandomSource + ?Sized>(
        &self,
        level: &mut L,
        random: &mut R,
        pos: &impl Pos3,
    ) {
        if random.next_boolean() {
            level.set_block(pos, BlockState::BASALT, 2);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.BasaltPillarFeature#placeHangOff
    fn place_hang_off<L: LevelAccessor + ?Sized, R: RandomSource + ?Sized>(
        &self,
        level: &mut L,
        random: &mut R,
        pos: &impl Pos3,
    ) -> bool {
        if random.next_int(10) != 0 {
            level.set_block(pos, BlockState::BASALT, 2);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    struct MockRandom {
        ints: Vec<i32>,
        bools: Vec<bool>,
        int_idx: usize,
        bool_idx: usize,
    }

    impl MockRandom {
        fn new(ints: Vec<i32>, bools: Vec<bool>) -> Self {
            Self {
                ints,
                bools,
                int_idx: 0,
                bool_idx: 0,
            }
        }
    }

    impl RandomSource for MockRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            let v = self.ints.get(self.int_idx).copied().unwrap_or(0);
            self.int_idx += 1;
            ((v % bound) + bound) % bound
        }

        fn next_boolean(&mut self) -> bool {
            let v = self.bools.get(self.bool_idx).copied().unwrap_or(false);
            self.bool_idx += 1;
            v
        }
    }

    struct MockLevel {
        filled: HashSet<(i32, i32, i32)>,
        build_min: i32,
        build_max: i32,
        placed: Vec<(i32, i32, i32, BlockState, i32)>,
    }

    impl LevelAccessor for MockLevel {
        fn is_empty_block(&self, pos: &impl Pos3) -> bool {
            !self.filled.contains(&(pos.x(), pos.y(), pos.z()))
        }

        fn is_outside_build_height(&self, pos: &impl Pos3) -> bool {
            let y = pos.y();
            y < self.build_min || y >= self.build_max
        }

        fn set_block(&mut self, pos: &impl Pos3, state: BlockState, flags: i32) -> bool {
            let key = (pos.x(), pos.y(), pos.z());
            self.filled.insert(key);
            self.placed.push((key.0, key.1, key.2, state, flags));
            true
        }
    }

    #[test]
    fn place_returns_false_when_invalid_start() {
        let feature = BasaltPillarFeature::new(NoneFeatureConfiguration);

        let mut level = MockLevel {
            filled: HashSet::new(),
            build_min: -64,
            build_max: 320,
            placed: Vec::new(),
        };
        let mut rng = MockRandom::new(vec![], vec![]);
        let mut ctx = FeaturePlaceContext {
            origin: BlockPos::new(0, 10, 0),
            level: &mut level,
            random: &mut rng,
        };
        // origin empty and above empty -> invalid
        assert!(!feature.place(&mut ctx));

        let mut level2 = MockLevel {
            filled: [(0, 10, 0), (0, 11, 0)].into_iter().collect(),
            build_min: -64,
            build_max: 320,
            placed: Vec::new(),
        };
        let mut rng2 = MockRandom::new(vec![], vec![]);
        let mut ctx2 = FeaturePlaceContext {
            origin: BlockPos::new(0, 10, 0),
            level: &mut level2,
            random: &mut rng2,
        };
        // origin not empty -> invalid
        assert!(!feature.place(&mut ctx2));
    }

    #[test]
    fn place_creates_pillar_and_honors_flags() {
        let mut filled = HashSet::new();
        filled.insert((0, 11, 0)); // above origin: solid
        filled.insert((0, 4, 0)); // ground below pillar
        let mut level = MockLevel {
            filled,
            build_min: -64,
            build_max: 320,
            placed: Vec::new(),
        };
        let ints = vec![1; 512]; // non-zero: hangoffs place, cluster condition often true
        let bools = vec![true; 64];
        let mut rng = MockRandom::new(ints, bools);
        let mut ctx = FeaturePlaceContext {
            origin: BlockPos::new(0, 10, 0),
            level: &mut level,
            random: &mut rng,
        };
        let feature = BasaltPillarFeature::new(NoneFeatureConfiguration);
        assert!(feature.place(&mut ctx));

        for y in 5..=10 {
            assert!(
                level.filled.contains(&(0, y, 0)),
                "missing pillar block at y={y}"
            );
        }
        assert!(level
            .placed
            .iter()
            .all(|(_, _, _, state, flags)| *state == BlockState::BASALT && *flags == 2));
    }

    #[test]
    fn hang_off_short_circuits_and_build_height_stops() {
        // Short-circuit: north hangoff fails at top, should not be attempted below.
        let mut filled = HashSet::new();
        filled.insert((0, 12, 0)); // above origin
        filled.insert((0, 9, 0)); // ground
        let mut level = MockLevel {
            filled,
            build_min: -64,
            build_max: 320,
            placed: Vec::new(),
        };
        // y=11: N=0(false), S=1, W=1, E=1; y=10: N skipped, S=1, W=1, E=1
        let mut ints = vec![0, 1, 1, 1, 1, 1, 1];
        ints.extend(std::iter::repeat(9).take(49)); // cluster
        let bools = vec![false; 8];
        let mut rng = MockRandom::new(ints, bools);
        let mut ctx = FeaturePlaceContext {
            origin: BlockPos::new(0, 11, 0),
            level: &mut level,
            random: &mut rng,
        };
        let feature = BasaltPillarFeature::new(NoneFeatureConfiguration);
        assert!(feature.place(&mut ctx));
        assert!(!level.filled.contains(&(0, 11, -1))); // north at y=11 not placed
        assert!(!level.filled.contains(&(0, 10, -1))); // north at y=10 short-circuited
        assert!(level.filled.contains(&(0, 11, 1)));
        assert!(level.filled.contains(&(0, 10, 1)));

        // Build height limit: returns true when hitting outside build height.
        let mut level2 = MockLevel {
            filled: [(0, 1, 0)].into_iter().collect(),
            build_min: 0,
            build_max: 320,
            placed: Vec::new(),
        };
        let mut rng2 = MockRandom::new(vec![1; 64], vec![false; 16]);
        let mut ctx2 = FeaturePlaceContext {
            origin: BlockPos::new(0, 0, 0),
            level: &mut level2,
            random: &mut rng2,
        };
        assert!(feature.place(&mut ctx2));
        assert!(level2.filled.contains(&(0, 0, 0)));
    }

    #[test]
    fn negative_coords_and_integer_overflow_semantics() {
        let mut filled = HashSet::new();
        filled.insert((-5, -10, 7)); // above origin
        filled.insert((-5, -13, 7)); // ground
        let mut level = MockLevel {
            filled,
            build_min: -64,
            build_max: 320,
            placed: Vec::new(),
        };
        let mut rng = MockRandom::new(vec![1; 512], vec![false; 64]);
        let mut ctx = FeaturePlaceContext {
            origin: BlockPos::new(-5, -11, 7),
            level: &mut level,
            random: &mut rng,
        };
        let feature = BasaltPillarFeature::new(NoneFeatureConfiguration);
        assert!(feature.place(&mut ctx));
        assert!(level.filled.contains(&(-5, -11, 7)));
        assert!(level.filled.contains(&(-5, -12, 7)));

        // Java Math.abs(Integer.MIN_VALUE) == Integer.MIN_VALUE
        assert_eq!(mth_abs(i32::MIN), i32::MIN);
        assert_eq!(mth_abs(-1), 1);
        assert_eq!(mth_abs(0), 0);

        // Wrapping arithmetic sanity for position helpers
        let p = BlockPos::new(i32::MAX, i32::MIN, -1);
        assert_eq!(p.above().y, i32::MIN.wrapping_add(1));
        assert_eq!(p.offset(1, 0, -1).x, i32::MIN);
    }

    #[test]
    fn float_bits_sanity() {
        // No float operations in BasaltPillarFeature; verify bit-level identity.
        let f32_bits: u32 = 0xFFC0_0001;
        let f32_val = f32::from_bits(f32_bits);
        assert_eq!(f32_val.to_bits(), f32_bits);

        let f64_bits: u64 = 0x7FF8_0000_0000_0001;
        let f64_val = f64::from_bits(f64_bits);
        assert_eq!(f64_val.to_bits(), f64_bits);
    }
}
