#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn mutable(self) -> MutableBlockPos {
        MutableBlockPos {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl From<MutableBlockPos> for BlockPos {
    #[inline]
    fn from(value: MutableBlockPos) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn immutable(self) -> BlockPos {
        BlockPos {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }

    pub fn move_dir(&mut self, dir: Direction) -> &mut Self {
        match dir {
            Direction::Down => {
                self.y = self.y.wrapping_sub(1);
            }
            Direction::Up => {
                self.y = self.y.wrapping_add(1);
            }
            Direction::North => {
                self.z = self.z.wrapping_sub(1);
            }
            Direction::South => {
                self.z = self.z.wrapping_add(1);
            }
            Direction::West => {
                self.x = self.x.wrapping_sub(1);
            }
            Direction::East => {
                self.x = self.x.wrapping_add(1);
            }
        }
        self
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
    #[inline]
    pub const fn get_opposite(self) -> Self {
        match self {
            Self::Down => Self::Up,
            Self::Up => Self::Down,
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }

    #[inline]
    pub const fn get_clock_wise(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
            Self::Up => Self::Up,
            Self::Down => Self::Down,
        }
    }

    #[inline]
    pub const fn get_counter_clock_wise(self) -> Self {
        match self {
            Self::North => Self::West,
            Self::West => Self::South,
            Self::South => Self::East,
            Self::East => Self::North,
            Self::Up => Self::Up,
            Self::Down => Self::Down,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectionPlane {
    Horizontal,
}

impl DirectionPlane {
    pub fn get_random_direction(self, random: &mut impl RandomSource) -> Direction {
        match self {
            Self::Horizontal => {
                const DIRS: [Direction; 4] = [
                    Direction::North,
                    Direction::East,
                    Direction::South,
                    Direction::West,
                ];
                let idx = random.next_int(DIRS.len() as i32);
                DIRS[idx as usize]
            }
        }
    }
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
    fn next_float(&mut self) -> f32;
}

fn shuffle<T>(list: &mut [T], random: &mut impl RandomSource) {
    let mut i: i32 = list.len() as i32;
    while i > 1 {
        let j: i32 = random.next_int(i);
        let i1 = (i.wrapping_sub(1)) as usize;
        let j1 = j as usize;
        list.swap(i1, j1);
        i = i.wrapping_sub(1);
    }
}

fn to_shuffled_list<T: Clone>(items: &[T], random: &mut impl RandomSource) -> Vec<T> {
    let mut list: Vec<T> = items.to_vec();
    shuffle(&mut list, random);
    list
}

fn get_random<T: Copy>(items: &[T], random: &mut impl RandomSource) -> T {
    let idx = random.next_int(items.len() as i32);
    items[idx as usize]
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CoralClawFeature;

impl CoralClawFeature {
    #[inline]
    pub const fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.CoralClawFeature#placeFeature
    pub fn place_feature<S, R>(
        &self,
        random: &mut R,
        pos: BlockPos,
        state: &S,
        place_coral_block: &mut impl FnMut(&mut R, BlockPos, &S) -> bool,
    ) -> bool
    where
        S: Clone,
        R: RandomSource,
    {
        if !place_coral_block(random, pos, state) {
            return false;
        }

        let dir4: Direction = DirectionPlane::Horizontal.get_random_direction(random);
        let len: i32 = random.next_int(2).wrapping_add(2);

        let shuffled_dirs: [Direction; 3] =
            [dir4, dir4.get_clock_wise(), dir4.get_counter_clock_wise()];
        let directions: Vec<Direction> = to_shuffled_list(&shuffled_dirs, random);

        let mut limit: usize = len as usize;
        if limit > directions.len() {
            limit = directions.len();
        }

        let mut idx_dir: usize = 0;
        while idx_dir < limit {
            let dir8: Direction = directions[idx_dir];
            let mut pos9: MutableBlockPos = pos.mutable();
            let count10: i32 = random.next_int(2).wrapping_add(1);
            pos9.move_dir(dir8);

            let dir11: Direction;
            let count12: i32;
            if dir8 == dir4 {
                dir11 = dir4;
                count12 = random.next_int(3).wrapping_add(2);
            } else {
                pos9.move_dir(Direction::Up);
                let choices: [Direction; 2] = [dir8, Direction::Up];
                dir11 = get_random(&choices, random);
                count12 = random.next_int(3).wrapping_add(3);
            }

            let mut i16: i32 = 0;
            while i16 < count10 && place_coral_block(random, pos9.immutable(), state) {
                pos9.move_dir(dir11);
                i16 = i16.wrapping_add(1);
            }

            pos9.move_dir(dir11.get_opposite());
            pos9.move_dir(Direction::Up);

            let mut i17: i32 = 0;
            while i17 < count12 {
                pos9.move_dir(dir4);
                if !place_coral_block(random, pos9.immutable(), state) {
                    break;
                }

                if random.next_float() < 0.25_f32 {
                    pos9.move_dir(Direction::Up);
                }

                i17 = i17.wrapping_add(1);
            }

            idx_dir = idx_dir.wrapping_add(1);
        }

        true
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
        fn next_int(&mut self, bound: i32) -> i32 {
            let v = if self.int_idx < self.ints.len() {
                self.ints[self.int_idx]
            } else {
                0
            };
            self.int_idx += 1;
            if bound <= 0 {
                0
            } else {
                ((v % bound) + bound) % bound
            }
        }

        fn next_float(&mut self) -> f32 {
            let v = if self.float_idx < self.floats.len() {
                self.floats[self.float_idx]
            } else {
                0.0_f32
            };
            self.float_idx += 1;
            v
        }
    }

    #[test]
    fn returns_false_when_initial_placement_fails() {
        let feature = CoralClawFeature::new();
        let mut random = SeqRandom::new(vec![], vec![]);
        let pos = BlockPos::new(1, 64, 2);
        let state = 42_u8;
        let mut calls: i32 = 0;
        let result = feature.place_feature(&mut random, pos, &state, &mut |_r, _p, _s| {
            calls = calls.wrapping_add(1);
            false
        });
        assert!(!result);
        assert_eq!(calls, 1);
    }

    #[test]
    fn places_coral_and_returns_true_when_initial_succeeds() {
        let feature = CoralClawFeature::new();
        let mut random = SeqRandom::new(vec![0; 128], vec![0.5_f32; 128]);
        let pos = BlockPos::new(0, 64, 0);
        let state = 1_u8;
        let mut placed: Vec<BlockPos> = Vec::new();
        let result = feature.place_feature(&mut random, pos, &state, &mut |_r, p, _s| {
            placed.push(p);
            true
        });
        assert!(result);
        assert!(!placed.is_empty());
        assert_eq!(placed[0], pos);
    }

    #[test]
    fn handles_negative_coordinates() {
        let feature = CoralClawFeature::new();
        let mut random = SeqRandom::new(vec![0; 128], vec![0.5_f32; 128]);
        let pos = BlockPos::new(-5, -10, -20);
        let state = 7_i32;
        let mut placed: Vec<BlockPos> = Vec::new();
        let result = feature.place_feature(&mut random, pos, &state, &mut |_r, p, _s| {
            placed.push(p);
            true
        });
        assert!(result);
        assert_eq!(placed[0], pos);
        assert!(placed.iter().all(|p| p.y <= pos.y.wrapping_add(32)));
    }

    #[test]
    fn mutable_block_pos_move_wraps_on_overflow() {
        let mut p1 = MutableBlockPos::new(i32::MAX, 0, 0);
        p1.move_dir(Direction::East);
        assert_eq!(p1.x, i32::MIN);

        let mut p2 = MutableBlockPos::new(i32::MIN, 0, 0);
        p2.move_dir(Direction::West);
        assert_eq!(p2.x, i32::MAX);

        let mut p3 = MutableBlockPos::new(0, i32::MAX, 0);
        p3.move_dir(Direction::Up);
        assert_eq!(p3.y, i32::MIN);

        let mut p4 = MutableBlockPos::new(0, i32::MIN, 0);
        p4.move_dir(Direction::Down);
        assert_eq!(p4.y, i32::MAX);

        let mut p5 = MutableBlockPos::new(0, 0, i32::MAX);
        p5.move_dir(Direction::South);
        assert_eq!(p5.z, i32::MIN);
    }

    #[test]
    fn float_threshold_is_strictly_less_than_quarter() {
        let feature = CoralClawFeature::new();
        let base_ints = vec![0; 256];

        let mut placed_low: Vec<BlockPos> = Vec::new();
        let mut random_low = SeqRandom::new(base_ints.clone(), vec![0.0_f32; 256]);
        let res_low = feature.place_feature(
            &mut random_low,
            BlockPos::new(0, 64, 0),
            &1_u8,
            &mut |_r, p, _s| {
                placed_low.push(p);
                true
            },
        );

        let mut placed_edge: Vec<BlockPos> = Vec::new();
        let mut random_edge = SeqRandom::new(base_ints, vec![0.25_f32; 256]);
        let res_edge = feature.place_feature(
            &mut random_edge,
            BlockPos::new(0, 64, 0),
            &1_u8,
            &mut |_r, p, _s| {
                placed_edge.push(p);
                true
            },
        );

        assert!(res_low);
        assert!(res_edge);
        assert_ne!(placed_low, placed_edge);
        assert_eq!(0.25_f32.to_bits(), 0x3E80_0000);
        assert!(0.249_999_99_f32 < 0.25_f32);
        assert!(!(0.25_f32 < 0.25_f32));
    }

    #[test]
    fn second_loop_breaks_on_placement_failure_but_feature_still_succeeds() {
        let feature = CoralClawFeature::new();
        let mut random = SeqRandom::new(vec![0; 256], vec![0.5_f32; 256]);
        let pos = BlockPos::new(10, 64, 10);
        let mut call_count: i32 = 0;
        let result = feature.place_feature(&mut random, pos, &9_u8, &mut |_r, _p, _s| {
            call_count = call_count.wrapping_add(1);
            call_count != 3
        });
        assert!(result);
        assert!(call_count >= 3);
    }
}
