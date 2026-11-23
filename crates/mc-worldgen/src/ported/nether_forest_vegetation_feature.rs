use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#below()
    pub fn below(&self) -> Self {
        Self::new(self.x, self.y.wrapping_sub(1), self.z)
    }

    // JAVA: net.minecraft.core.BlockPos#offset(int,int,int)
    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
        Self::new(
            self.x.wrapping_add(dx),
            self.y.wrapping_add(dy),
            self.z.wrapping_add(dz),
        )
    }
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

pub trait VegetationBlockState: Clone {
    fn is_nylium(&self) -> bool;
    fn can_survive<L: WorldGenLevel<BlockState = Self> + ?Sized>(&self, level: &L, pos: BlockPos) -> bool;
}

pub trait BlockStateProvider {
    type State: VegetationBlockState;
    fn get_state<R: RandomSource + ?Sized>(&self, random: &mut R, pos: BlockPos) -> Self::State;
}

pub trait WorldGenLevel {
    type BlockState: VegetationBlockState;
    fn get_block_state(&self, pos: BlockPos) -> Self::BlockState;
    fn is_empty_block(&self, pos: BlockPos) -> bool;
    fn set_block(&mut self, pos: BlockPos, state: Self::BlockState, flags: i32) -> bool;
    fn get_min_y(&self) -> i32;
    fn get_max_y(&self) -> i32;
}

pub struct NetherForestVegetationConfig<P> {
    pub state_provider: P,
    pub spread_width: i32,
    pub spread_height: i32,
}

pub struct NetherForestVegetationFeature;

impl NetherForestVegetationFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.NetherForestVegetationFeature#<init>(com.mojang.serialization.Codec)
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.NetherForestVegetationFeature#place(net.minecraft.world.level.levelgen.feature.FeaturePlaceContext)
    pub fn place<L, R, P>(
        &self,
        level: &mut L,
        origin: BlockPos,
        random: &mut R,
        config: &NetherForestVegetationConfig<P>,
    ) -> bool
    where
        L: WorldGenLevel + ?Sized,
        R: RandomSource + ?Sized,
        P: BlockStateProvider<State = L::BlockState>,
    {
        let below_state = level.get_block_state(origin.below());
        if !below_state.is_nylium() {
            return false;
        }

        let y = origin.y;
        let min_y = level.get_min_y();
        let max_y = level.get_max_y();
        if y >= min_y.wrapping_add(1) && y.wrapping_add(1) <= max_y {
            let mut placed_count: i32 = 0;
            let attempts = config.spread_width.wrapping_mul(config.spread_width);
            let mut i: i32 = 0;
            while i < attempts {
                let dx = random
                    .next_int(config.spread_width)
                    .wrapping_sub(random.next_int(config.spread_width));
                let dy = random
                    .next_int(config.spread_height)
                    .wrapping_sub(random.next_int(config.spread_height));
                let dz = random
                    .next_int(config.spread_width)
                    .wrapping_sub(random.next_int(config.spread_width));
                let pos = origin.offset(dx, dy, dz);
                let state = config.state_provider.get_state(random, pos);
                if level.is_empty_block(pos) && pos.y > min_y && state.can_survive(level, pos) {
                    level.set_block(pos, state, 2);
                    placed_count = placed_count.wrapping_add(1);
                }
                i = i.wrapping_add(1);
            }

            placed_count > 0
        } else {
            false
        }
    }
}

impl Default for NetherForestVegetationFeature {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct TestState {
        kind: u8, // 0 = air, 1 = nylium, 2 = vegetation, 3 = other
    }

    impl TestState {
        const AIR: Self = Self { kind: 0 };
        const NYLIUM: Self = Self { kind: 1 };
        const VEGETATION: Self = Self { kind: 2 };
        const OTHER: Self = Self { kind: 3 };
    }

    impl VegetationBlockState for TestState {
        fn is_nylium(&self) -> bool {
            self.kind == 1
        }

        fn can_survive<L: WorldGenLevel<BlockState = Self> + ?Sized>(
            &self,
            _level: &L,
            _pos: BlockPos,
        ) -> bool {
            self.kind == 2
        }
    }

    struct FixedProvider;

    impl BlockStateProvider for FixedProvider {
        type State = TestState;
        fn get_state<R: RandomSource + ?Sized>(&self, _random: &mut R, _pos: BlockPos) -> Self::State {
            TestState::VEGETATION
        }
    }

    struct SeqRandom {
        values: Vec<i32>,
        idx: usize,
    }

    impl SeqRandom {
        fn new(values: Vec<i32>) -> Self {
            Self { values, idx: 0 }
        }
        fn zeros() -> Self {
            Self::new(vec![0])
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            let v = self.values[self.idx % self.values.len()];
            self.idx += 1;
            v.rem_euclid(bound)
        }
    }

    struct MockWorld {
        blocks: HashMap<(i32, i32, i32), TestState>,
        min_y: i32,
        max_y: i32,
    }

    impl MockWorld {
        fn new(min_y: i32, max_y: i32) -> Self {
            Self {
                blocks: HashMap::new(),
                min_y,
                max_y,
            }
        }

        fn set(&mut self, pos: BlockPos, state: TestState) {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
        }

        fn get(&self, pos: BlockPos) -> TestState {
            self.blocks
                .get(&(pos.x, pos.y, pos.z))
                .copied()
                .unwrap_or(TestState::AIR)
        }
    }

    impl WorldGenLevel for MockWorld {
        type BlockState = TestState;

        fn get_block_state(&self, pos: BlockPos) -> Self::BlockState {
            self.get(pos)
        }

        fn is_empty_block(&self, pos: BlockPos) -> bool {
            self.get(pos) == TestState::AIR
        }

        fn set_block(&mut self, pos: BlockPos, state: Self::BlockState, _flags: i32) -> bool {
            self.set(pos, state);
            true
        }

        fn get_min_y(&self) -> i32 {
            self.min_y
        }

        fn get_max_y(&self) -> i32 {
            self.max_y
        }
    }

    #[test]
    fn rejects_when_below_is_not_nylium() {
        let mut world = MockWorld::new(-64, 320);
        let origin = BlockPos::new(0, 64, 0);
        world.set(origin.below(), TestState::OTHER);
        let mut random = SeqRandom::zeros();
        let config = NetherForestVegetationConfig {
            state_provider: FixedProvider,
            spread_width: 3,
            spread_height: 2,
        };
        assert!(!NetherForestVegetationFeature::new().place(
            &mut world,
            origin,
            &mut random,
            &config
        ));
    }

    #[test]
    fn rejects_when_origin_y_out_of_bounds() {
        let mut world = MockWorld::new(0, 10);
        let origin = BlockPos::new(0, 0, 0);
        world.set(origin.below(), TestState::NYLIUM);
        let mut random = SeqRandom::zeros();
        let config = NetherForestVegetationConfig {
            state_provider: FixedProvider,
            spread_width: 3,
            spread_height: 2,
        };
        assert!(!NetherForestVegetationFeature::new().place(
            &mut world,
            origin,
            &mut random,
            &config
        ));
    }

    #[test]
    fn places_vegetation_on_valid_nylium() {
        let mut world = MockWorld::new(-64, 320);
        let origin = BlockPos::new(5, 64, -2);
        world.set(origin.below(), TestState::NYLIUM);
        let mut random = SeqRandom::zeros();
        let config = NetherForestVegetationConfig {
            state_provider: FixedProvider,
            spread_width: 3,
            spread_height: 2,
        };
        assert!(NetherForestVegetationFeature::new().place(
            &mut world,
            origin,
            &mut random,
            &config
        ));
        assert_eq!(world.get(origin), TestState::VEGETATION);
    }

    #[test]
    fn does_not_place_below_min_y_plus_one() {
        let mut world = MockWorld::new(0, 128);
        let origin = BlockPos::new(0, 10, 0);
        world.set(origin.below(), TestState::NYLIUM);
        // spread_height gives dy = 1-2 = -1 when sequence produces 1,2,... so pos.y becomes 9
        // but also try to force pos.y == 0 by using large negative offset: dy = 0-5 in height 6
        let mut random = SeqRandom::new(vec![0, 0, 0, 5, 0, 0]);
        let config = NetherForestVegetationConfig {
            state_provider: FixedProvider,
            spread_width: 1,
            spread_height: 6,
        };
        let placed = NetherForestVegetationFeature::new().place(&mut world, origin, &mut random, &config);
        // pos.y == 5 is > min_y (0), so it can place; ensure no placement at y <= min_y
        if placed {
            assert!(world.get(BlockPos::new(0, 5, 0)) == TestState::VEGETATION);
        }
        assert_ne!(world.get(BlockPos::new(0, 0, 0)), TestState::VEGETATION);
    }

    #[test]
    fn returns_false_when_no_position_can_survive() {
        #[derive(Clone, Copy)]
        struct UnsurvivableProvider;
        impl BlockStateProvider for UnsurvivableProvider {
            type State = TestState;
            fn get_state<R: RandomSource + ?Sized>(&self, _random: &mut R, _pos: BlockPos) -> Self::State {
                TestState::OTHER
            }
        }

        let mut world = MockWorld::new(-64, 320);
        let origin = BlockPos::new(0, 64, 0);
        world.set(origin.below(), TestState::NYLIUM);
        let mut random = SeqRandom::zeros();
        let config = NetherForestVegetationConfig {
            state_provider: UnsurvivableProvider,
            spread_width: 2,
            spread_height: 2,
        };
        assert!(!NetherForestVegetationFeature::new().place(
            &mut world,
            origin,
            &mut random,
            &config
        ));
    }
}
