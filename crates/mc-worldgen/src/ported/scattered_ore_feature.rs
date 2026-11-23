//! Port of `net.minecraft.world.level.levelgen.feature.ScatteredOreFeature`.

use std::collections::HashMap;

const MAX_DIST_FROM_ORIGIN: i32 = 7;

/// Minimal random source interface matching Java `RandomSource` behavior used here.
pub trait RandomSource {
    /// Equivalent to Java `RandomSource.nextInt(int)`. `bound` must be positive.
    fn next_int(&mut self, bound: i32) -> i32;
    /// Equivalent to Java `RandomSource.nextFloat()`, returning a value in [0.0, 1.0).
    fn next_float(&mut self) -> f32;
}

/// Minimal block position.
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
}

/// Minimal mutable block position.
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

    /// Equivalent to Java `BlockPos.MutableBlockPos#setWithOffset`.
    pub fn set_with_offset(&mut self, pos: BlockPos, dx: i32, dy: i32, dz: i32) -> &mut Self {
        self.x = pos.x.wrapping_add(dx);
        self.y = pos.y.wrapping_add(dy);
        self.z = pos.z.wrapping_add(dz);
        self
    }

    pub const fn as_pos(self) -> BlockPos {
        BlockPos {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl From<MutableBlockPos> for BlockPos {
    fn from(value: MutableBlockPos) -> Self {
        value.as_pos()
    }
}

/// Minimal block state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BlockState {
    pub id: u32,
    pub is_air: bool,
}

impl BlockState {
    pub const fn new(id: u32, is_air: bool) -> Self {
        Self { id, is_air }
    }
}

/// Minimal world generation level interface.
pub trait WorldGenLevel {
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool;
}

/// Minimal target block state matching `OreConfiguration.TargetBlockState`.
pub struct TargetBlockState {
    pub test: fn(BlockState, &mut dyn RandomSource) -> bool,
    pub state: BlockState,
}

impl std::fmt::Debug for TargetBlockState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TargetBlockState")
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

/// Minimal ore configuration matching fields used by `ScatteredOreFeature`.
#[derive(Debug)]
pub struct OreConfiguration {
    pub target_states: Vec<TargetBlockState>,
    pub size: i32,
    pub discard_chance_on_air_exposure: f32,
}

/// Minimal feature place context.
pub struct FeaturePlaceContext<'a> {
    pub level: &'a mut dyn WorldGenLevel,
    pub random: &'a mut dyn RandomSource,
    pub config: &'a OreConfiguration,
    pub origin: BlockPos,
}

// JAVA: java.lang.Math#round(float)
fn java_math_round_f32(a: f32) -> i32 {
    let int_bits = a.to_bits() as i32;
    let biased_exp = (int_bits & 0x7F80_0000u32 as i32) >> 23;
    let shift = (24_i32 - 2 + 127) - biased_exp;
    if (shift & -32) == 0 {
        let mut r = (int_bits & 0x007F_FFFFu32 as i32) | 0x0080_0000u32 as i32;
        if int_bits < 0 {
            r = r.wrapping_neg();
        }
        ((r >> shift).wrapping_add(1)) >> 1
    } else {
        // Matches Java (int) float cast semantics for NaN/Inf/out-of-range.
        a as i32
    }
}

// JAVA: net.minecraft.world.level.levelgen.feature.OreFeature#shouldSkipAirCheck
fn should_skip_air_check(random: &mut dyn RandomSource, chance: f32) -> bool {
    if chance <= 0.0f32 {
        true
    } else if chance >= 1.0f32 {
        false
    } else {
        random.next_float() >= chance
    }
}

// JAVA: net.minecraft.world.level.levelgen.feature.OreFeature#isAdjacentToAir
fn is_adjacent_to_air(get_block_state: &dyn Fn(BlockPos) -> BlockState, pos: BlockPos) -> bool {
    const OFFSETS: [(i32, i32, i32); 6] = [
        (0, -1, 0), // DOWN
        (0, 1, 0),  // UP
        (0, 0, -1), // NORTH
        (0, 0, 1),  // SOUTH
        (-1, 0, 0), // WEST
        (1, 0, 0),  // EAST
    ];
    for (dx, dy, dz) in OFFSETS {
        let neighbor = BlockPos::new(
            pos.x.wrapping_add(dx),
            pos.y.wrapping_add(dy),
            pos.z.wrapping_add(dz),
        );
        if get_block_state(neighbor).is_air {
            return true;
        }
    }
    false
}

// JAVA: net.minecraft.world.level.levelgen.feature.OreFeature#canPlaceOre
pub fn can_place_ore(
    state: BlockState,
    get_block_state: &dyn Fn(BlockPos) -> BlockState,
    random: &mut dyn RandomSource,
    config: &OreConfiguration,
    target: &TargetBlockState,
    pos: &MutableBlockPos,
) -> bool {
    if !(target.test)(state, random) {
        return false;
    }
    if should_skip_air_check(random, config.discard_chance_on_air_exposure) {
        true
    } else {
        !is_adjacent_to_air(get_block_state, pos.as_pos())
    }
}

/// Port of `net.minecraft.world.level.levelgen.feature.ScatteredOreFeature`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ScatteredOreFeature;

impl ScatteredOreFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.ScatteredOreFeature#ScatteredOreFeature
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.ScatteredOreFeature#place
    pub fn place(&self, ctx: &mut FeaturePlaceContext<'_>) -> bool {
        let level: &mut dyn WorldGenLevel = &mut *ctx.level;
        let random: &mut dyn RandomSource = &mut *ctx.random;
        let config = ctx.config;
        let origin = ctx.origin;

        let count = random.next_int(config.size.wrapping_add(1));
        let mut pos = MutableBlockPos::new();

        let mut i: i32 = 0;
        while i < count {
            let spread = if i < MAX_DIST_FROM_ORIGIN {
                i
            } else {
                MAX_DIST_FROM_ORIGIN
            };
            self.offset_target_pos(&mut pos, random, origin, spread);
            let state = level.get_block_state(pos.as_pos());

            for target in &config.target_states {
                let can_place = can_place_ore(
                    state,
                    &|p: BlockPos| level.get_block_state(p),
                    random,
                    config,
                    target,
                    &pos,
                );
                if can_place {
                    level.set_block(pos.as_pos(), target.state, 2);
                    break;
                }
            }

            i = i.wrapping_add(1);
        }

        true
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.ScatteredOreFeature#offsetTargetPos
    fn offset_target_pos(
        &self,
        pos: &mut MutableBlockPos,
        random: &mut dyn RandomSource,
        origin: BlockPos,
        max_dist: i32,
    ) {
        let x = self.get_random_placement_in_one_axis_relative_to_origin(random, max_dist);
        let y = self.get_random_placement_in_one_axis_relative_to_origin(random, max_dist);
        let z = self.get_random_placement_in_one_axis_relative_to_origin(random, max_dist);
        pos.set_with_offset(origin, x, y, z);
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.ScatteredOreFeature#getRandomPlacementInOneAxisRelativeToOrigin
    fn get_random_placement_in_one_axis_relative_to_origin(
        &self,
        random: &mut dyn RandomSource,
        max_dist: i32,
    ) -> i32 {
        let a = random.next_float();
        let b = random.next_float();
        java_math_round_f32((a - b) * (max_dist as f32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct SeqRandom {
        ints: std::vec::IntoIter<i32>,
        floats: std::vec::IntoIter<f32>,
        int_bounds: Vec<i32>,
    }

    impl SeqRandom {
        fn new(ints: Vec<i32>, floats: Vec<f32>) -> Self {
            Self {
                ints: ints.into_iter(),
                floats: floats.into_iter(),
                int_bounds: Vec::new(),
            }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            assert!(bound > 0, "bound must be positive");
            self.int_bounds.push(bound);
            self.ints.next().expect("exhausted ints")
        }

        fn next_float(&mut self) -> f32 {
            self.floats.next().expect("exhausted floats")
        }
    }

    struct MockLevel {
        states: HashMap<(i32, i32, i32), BlockState>,
        placed: Vec<(BlockPos, BlockState, i32)>,
        default_state: BlockState,
    }

    impl MockLevel {
        fn new(default_state: BlockState) -> Self {
            Self {
                states: HashMap::new(),
                placed: Vec::new(),
                default_state,
            }
        }
    }

    impl WorldGenLevel for MockLevel {
        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            self.states
                .get(&(pos.x, pos.y, pos.z))
                .copied()
                .unwrap_or(self.default_state)
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool {
            self.placed.push((pos, state, flags));
            self.states.insert((pos.x, pos.y, pos.z), state);
            true
        }
    }

    #[test]
    fn round_f32_matches_java_ties_and_signs() {
        assert_eq!(java_math_round_f32(0.0f32), 0);
        assert_eq!(java_math_round_f32(0.5f32), 1);
        assert_eq!(java_math_round_f32(-0.5f32), 0); // Java ties toward +inf
        assert_eq!(java_math_round_f32(-1.5f32), -1);
        assert_eq!(java_math_round_f32(-2.5f32), -2);
        assert_eq!(java_math_round_f32(f32::NAN), 0);
        assert_eq!(java_math_round_f32(f32::INFINITY), i32::MAX);
        assert_eq!(java_math_round_f32(f32::NEG_INFINITY), i32::MIN);
        assert_eq!(java_math_round_f32(f32::from_bits(0x8000_0000)), 0); // -0.0
    }

    #[test]
    fn random_placement_uses_float_subtraction_order_and_java_rounding() {
        let feature = ScatteredOreFeature::new();
        let mut random = SeqRandom::new(vec![], vec![0.75f32, 0.25f32]);
        // (0.75 - 0.25) * 3 = 1.5, Java round -> 2
        let v = feature.get_random_placement_in_one_axis_relative_to_origin(&mut random, 3);
        assert_eq!(v, 2);

        let mut random2 = SeqRandom::new(vec![], vec![0.25f32, 0.75f32]);
        // (0.25 - 0.75) * 3 = -1.5, Java round -> -1
        let v2 = feature.get_random_placement_in_one_axis_relative_to_origin(&mut random2, 3);
        assert_eq!(v2, -1);

        let mut random3 = SeqRandom::new(vec![], vec![0.1f32, 0.9f32]);
        // (0.1 - 0.9) * 7 = -5.6, Java round -> -6
        let v3 = feature.get_random_placement_in_one_axis_relative_to_origin(&mut random3, 7);
        assert_eq!(v3, -6);
    }

    #[test]
    fn place_uses_size_plus_one_and_clamps_spread_at_7() {
        let feature = ScatteredOreFeature::new();
        let default_state = BlockState::new(0, true);
        let ore_state = BlockState::new(1, false);
        let mut level = MockLevel::new(default_state);

        let mut floats = Vec::new();
        // 9 placements * 3 axes * 2 floats; (1.0 - 0.0) * spread = spread
        for _ in 0..(9 * 3) {
            floats.push(1.0f32);
            floats.push(0.0f32);
        }
        let mut random = SeqRandom::new(vec![9], floats); // nextInt(11) -> 9

        let config = OreConfiguration {
            target_states: vec![TargetBlockState {
                test: |_s, _r| true,
                state: ore_state,
            }],
            size: 10,
            discard_chance_on_air_exposure: 0.0,
        };
        let origin = BlockPos::new(100, 200, 300);
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            random: &mut random,
            config: &config,
            origin,
        };

        assert!(feature.place(&mut ctx));
        drop(ctx);

        assert_eq!(random.int_bounds, vec![11]);
        assert_eq!(level.placed.len(), 9);
        let expected_spreads = [0_i32, 1, 2, 3, 4, 5, 6, 7, 7];
        for (idx, (pos, state, flags)) in level.placed.iter().enumerate() {
            let s = expected_spreads[idx];
            assert_eq!(*pos, BlockPos::new(100 + s, 200 + s, 300 + s));
            assert_eq!(*state, ore_state);
            assert_eq!(*flags, 2);
        }
    }

    #[test]
    fn place_respects_target_test_and_air_exposure() {
        let feature = ScatteredOreFeature::new();
        let stone = BlockState::new(1, false);
        let air = BlockState::new(0, true);
        let ore = BlockState::new(2, false);

        // Case 1: discard_chance 1.0 and adjacent air -> rejected
        let mut level = MockLevel::new(stone);
        level.states.insert((1, 0, 0), air);
        let mut random = SeqRandom::new(vec![1], vec![0.5, 0.5, 0.5, 0.5, 0.5, 0.5]);
        let config = OreConfiguration {
            target_states: vec![
                TargetBlockState {
                    test: |s, _r| s.id == 999,
                    state: ore,
                },
                TargetBlockState {
                    test: |s, _r| s.id == 1,
                    state: ore,
                },
            ],
            size: 1,
            discard_chance_on_air_exposure: 1.0,
        };
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            random: &mut random,
            config: &config,
            origin: BlockPos::new(0, 0, 0),
        };
        assert!(feature.place(&mut ctx));
        drop(ctx);
        assert!(level.placed.is_empty());

        // Case 2: discard_chance 0.0 skips air check -> placed
        let mut level2 = MockLevel::new(stone);
        level2.states.insert((1, 0, 0), air);
        let mut random2 = SeqRandom::new(vec![1], vec![0.5, 0.5, 0.5, 0.5, 0.5, 0.5]);
        let config2 = OreConfiguration {
            target_states: vec![TargetBlockState {
                test: |s, _r| s.id == 1,
                state: ore,
            }],
            size: 1,
            discard_chance_on_air_exposure: 0.0,
        };
        let mut ctx2 = FeaturePlaceContext {
            level: &mut level2,
            random: &mut random2,
            config: &config2,
            origin: BlockPos::new(0, 0, 0),
        };
        assert!(feature.place(&mut ctx2));
        drop(ctx2);
        assert_eq!(level2.placed.len(), 1);
        assert_eq!(level2.placed[0].1, ore);
        assert_eq!(level2.placed[0].2, 2);
    }

    #[test]
    fn offset_uses_wrapping_add_for_negative_and_overflow_coordinates() {
        let feature = ScatteredOreFeature::new();
        let origin = BlockPos::new(i32::MIN, -10, i32::MAX);
        let mut pos = MutableBlockPos::new();
        let mut random = SeqRandom::new(vec![], vec![1.0, 0.0, 1.0, 0.0, 1.0, 0.0]);
        feature.offset_target_pos(&mut pos, &mut random, origin, 1);
        assert_eq!(pos.x, i32::MIN.wrapping_add(1));
        assert_eq!(pos.y, -9);
        assert_eq!(pos.z, i32::MAX.wrapping_add(1));
    }
}
