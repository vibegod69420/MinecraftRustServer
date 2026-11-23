use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn get_x(self) -> i32 {
        self.x
    }

    pub fn get_y(self) -> i32 {
        self.y
    }

    pub fn get_z(self) -> i32 {
        self.z
    }

    pub fn mutable(self) -> MutableBlockPos {
        MutableBlockPos {
            x: self.x,
            y: self.y,
            z: self.z,
        }
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
    pub fn step_x(self) -> i32 {
        match self {
            Direction::West => -1,
            Direction::East => 1,
            _ => 0,
        }
    }

    pub fn step_y(self) -> i32 {
        match self {
            Direction::Down => -1,
            Direction::Up => 1,
            _ => 0,
        }
    }

    pub fn step_z(self) -> i32 {
        match self {
            Direction::North => -1,
            Direction::South => 1,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MutableBlockPos {
    x: i32,
    y: i32,
    z: i32,
}

impl MutableBlockPos {
    pub fn set(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    pub fn move_direction(&mut self, dir: Direction, n: i32) -> &mut Self {
        self.x = self.x.wrapping_add(dir.step_x().wrapping_mul(n));
        self.y = self.y.wrapping_add(dir.step_y().wrapping_mul(n));
        self.z = self.z.wrapping_add(dir.step_z().wrapping_mul(n));
        self
    }

    pub fn to_block_pos(self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }
}

pub trait LevelAccessor {}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
    fn next_float(&mut self) -> f32;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BlockState {
    pub id: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NoneFeatureConfiguration;

#[derive(Clone, Copy, Debug, Default)]
pub struct CoralMushroomFeature;

impl CoralMushroomFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.CoralMushroomFeature#<init>
    pub fn new<C>(_codec: C) -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.CoralMushroomFeature#placeFeature
    pub fn place_feature<L, R>(
        &self,
        level: &mut L,
        random: &mut R,
        pos: BlockPos,
        state: &BlockState,
        place_coral_block: &mut impl FnMut(&mut L, &mut R, BlockPos, &BlockState) -> bool,
    ) -> bool
    where
        L: LevelAccessor + ?Sized,
        R: RandomSource + ?Sized,
    {
        let i4: i32 = random.next_int(3).wrapping_add(3);
        let i5: i32 = random.next_int(3).wrapping_add(3);
        let i6: i32 = random.next_int(3).wrapping_add(3);
        let i7: i32 = random.next_int(3).wrapping_add(1);
        let mut mutable = pos.mutable();

        let mut i9: i32 = 0;
        while i9 <= i5 {
            let mut i10: i32 = 0;
            while i10 <= i4 {
                let mut i11: i32 = 0;
                while i11 <= i6 {
                    mutable.set(
                        i9.wrapping_add(pos.get_x()),
                        i10.wrapping_add(pos.get_y()),
                        i11.wrapping_add(pos.get_z()),
                    );
                    mutable.move_direction(Direction::Down, i7);
                    if (i9 != 0 && i9 != i5 || i10 != 0 && i10 != i4)
                        && (i11 != 0 && i11 != i6 || i10 != 0 && i10 != i4)
                        && (i9 != 0 && i9 != i5 || i11 != 0 && i11 != i6)
                        && (i9 == 0 || i9 == i5 || i10 == 0 || i10 == i4 || i11 == 0 || i11 == i6)
                        && !(random.next_float() < 0.1_f32)
                        && !place_coral_block(level, random, mutable.to_block_pos(), state)
                    {
                        // empty, matches Java
                    }
                    i11 = i11.wrapping_add(1);
                }
                i10 = i10.wrapping_add(1);
            }
            i9 = i9.wrapping_add(1);
        }

        true
    }
}

impl fmt::Display for CoralMushroomFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CoralMushroomFeature")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SeqRandom {
        ints: Vec<i32>,
        floats: Vec<f32>,
        int_idx: usize,
        float_idx: usize,
    }

    impl SeqRandom {
        fn new(ints: Vec<i32>, floats: Vec<f32>) -> Self {
            Self {
                ints,
                floats,
                int_idx: 0,
                float_idx: 0,
            }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            let v = self.ints[self.int_idx];
            self.int_idx += 1;
            v
        }

        fn next_float(&mut self) -> f32 {
            let v = self.floats[self.float_idx];
            self.float_idx += 1;
            v
        }
    }

    struct NoopLevel;
    impl LevelAccessor for NoopLevel {}

    #[test]
    fn place_feature_returns_true_and_places_face_cells() {
        let feat = CoralMushroomFeature::new(());
        let mut rng = SeqRandom::new(vec![0, 0, 0, 0], vec![1.0_f32; 64]);
        let mut level = NoopLevel;
        let pos = BlockPos::new(10, 20, 30);
        let state = BlockState { id: 1 };
        let mut calls: Vec<BlockPos> = Vec::new();
        let mut placer = |_: &mut NoopLevel, _: &mut SeqRandom, p: BlockPos, _: &BlockState| {
            calls.push(p);
            true
        };

        let ok = feat.place_feature(&mut level, &mut rng, pos, &state, &mut placer);
        assert!(ok);
        assert_eq!(calls.len(), 24); // 6 faces * 4 interior face cells
        for p in &calls {
            assert!(p.get_x() >= 10 && p.get_x() <= 13);
            assert!(p.get_y() >= 19 && p.get_y() <= 22); // moved down by 1
            assert!(p.get_z() >= 30 && p.get_z() <= 33);
        }
    }

    #[test]
    fn float_below_threshold_skips_and_consumes_only_on_face_cells() {
        let feat = CoralMushroomFeature::new(());
        let mut rng = SeqRandom::new(vec![0, 0, 0, 0], vec![0.0_f32; 64]);
        let mut level = NoopLevel;
        let pos = BlockPos::new(0, 64, 0);
        let state = BlockState { id: 1 };
        let mut count = 0usize;
        let mut placer = |_: &mut NoopLevel, _: &mut SeqRandom, _: BlockPos, _: &BlockState| {
            count += 1;
            true
        };

        let ok = feat.place_feature(&mut level, &mut rng, pos, &state, &mut placer);
        assert!(ok);
        assert_eq!(count, 0);
        assert_eq!(rng.float_idx, 24);
    }

    #[test]
    fn float_exact_threshold_does_not_skip() {
        let feat = CoralMushroomFeature::new(());
        let mut rng = SeqRandom::new(vec![0, 0, 0, 0], vec![0.1_f32; 64]);
        let mut level = NoopLevel;
        let pos = BlockPos::new(0, 64, 0);
        let state = BlockState { id: 1 };
        let mut count = 0usize;
        let mut placer = |_: &mut NoopLevel, _: &mut SeqRandom, _: BlockPos, _: &BlockState| {
            count += 1;
            true
        };

        feat.place_feature(&mut level, &mut rng, pos, &state, &mut placer);
        assert_eq!(count, 24);
    }

    #[test]
    fn negative_coordinates_and_wrapping_overflow() {
        let feat = CoralMushroomFeature::new(());
        let mut rng = SeqRandom::new(vec![0, 0, 0, 0], vec![1.0_f32; 64]);
        let mut level = NoopLevel;
        let pos = BlockPos::new(i32::MAX, -10, i32::MIN);
        let state = BlockState { id: 2 };
        let mut calls: Vec<BlockPos> = Vec::new();
        let mut placer = |_: &mut NoopLevel, _: &mut SeqRandom, p: BlockPos, _: &BlockState| {
            calls.push(p);
            true
        };

        feat.place_feature(&mut level, &mut rng, pos, &state, &mut placer);
        assert_eq!(calls.len(), 24);
        assert!(calls.iter().any(|p| p.get_x() == i32::MIN));
        assert!(calls.iter().any(|p| p.get_z() == i32::MIN));
        assert!(calls.iter().all(|p| p.get_y() >= -11 && p.get_y() <= -8));
    }

    #[test]
    fn place_coral_block_short_circuit_preserves_call_order() {
        let feat = CoralMushroomFeature::new(());
        // sizes 3,3,3,1 => 4x4x4. Provide floats only for geometric face cells.
        let mut floats = vec![1.0_f32; 24];
        floats[0] = 0.0_f32; // first geometric cell skips placer
        let mut rng = SeqRandom::new(vec![0, 0, 0, 0], floats);
        let mut level = NoopLevel;
        let pos = BlockPos::new(0, 0, 0);
        let state = BlockState { id: 3 };
        let mut count = 0usize;
        let mut placer = |_: &mut NoopLevel, _: &mut SeqRandom, _: BlockPos, _: &BlockState| {
            count += 1;
            true
        };

        feat.place_feature(&mut level, &mut rng, pos, &state, &mut placer);
        assert_eq!(count, 23);
        assert_eq!(rng.float_idx, 24);
    }
}
