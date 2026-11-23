#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#<init>(int,int,int)
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.Vec3i#getX
    pub fn get_x(&self) -> i32 {
        self.x
    }

    // JAVA: net.minecraft.core.Vec3i#getZ
    pub fn get_z(&self) -> i32 {
        self.z
    }

    // JAVA: net.minecraft.core.BlockPos#above()
    pub fn above(&self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(1),
            z: self.z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoubleBlockHalf {
    Upper,
    Lower,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockState {
    Water,
    Seagrass,
    TallSeagrass(DoubleBlockHalf),
    Other,
}

impl BlockState {
    // JAVA: net.minecraft.world.level.block.state.BlockBehaviour.BlockStateBase#is
    pub fn is_water(&self) -> bool {
        matches!(self, Self::Water)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightmapType {
    OceanFloor,
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
    fn next_double(&mut self) -> f64;
}

pub trait WorldGenLevel {
    fn get_height(&self, heightmap: HeightmapType, x: i32, z: i32) -> i32;
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool;
    fn can_survive(&self, state: BlockState, pos: BlockPos) -> bool;
}

#[derive(Debug, Clone, Copy)]
pub struct ProbabilityFeatureConfiguration {
    pub probability: f32,
}

pub struct SeagrassFeature;

impl SeagrassFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.SeagrassFeature#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.SeagrassFeature#place
    pub fn place<L: WorldGenLevel, R: RandomSource>(
        &self,
        level: &mut L,
        random: &mut R,
        origin: BlockPos,
        config: ProbabilityFeatureConfiguration,
    ) -> bool {
        let mut placed = false;
        let dx = random.next_int(8).wrapping_sub(random.next_int(8));
        let dz = random.next_int(8).wrapping_sub(random.next_int(8));
        let x = origin.get_x().wrapping_add(dx);
        let z = origin.get_z().wrapping_add(dz);
        let y = level.get_height(HeightmapType::OceanFloor, x, z);
        let pos = BlockPos::new(x, y, z);
        if level.get_block_state(pos).is_water() {
            let is_tall = random.next_double() < config.probability as f64;
            let state = if is_tall {
                BlockState::TallSeagrass(DoubleBlockHalf::Lower)
            } else {
                BlockState::Seagrass
            };
            if level.can_survive(state, pos) {
                if is_tall {
                    let upper_state = BlockState::TallSeagrass(DoubleBlockHalf::Upper);
                    let above_pos = pos.above();
                    if level.get_block_state(above_pos).is_water() {
                        level.set_block(pos, state, 2);
                        level.set_block(above_pos, upper_state, 2);
                    }
                } else {
                    level.set_block(pos, state, 2);
                }

                placed = true;
            }
        }

        placed
    }
}

impl Default for SeagrassFeature {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockRandom {
        ints: Vec<i32>,
        doubles: Vec<f64>,
        int_idx: usize,
        double_idx: usize,
    }

    impl MockRandom {
        fn new(ints: Vec<i32>, doubles: Vec<f64>) -> Self {
            Self {
                ints,
                doubles,
                int_idx: 0,
                double_idx: 0,
            }
        }
    }

    impl RandomSource for MockRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            let v = self.ints[self.int_idx % self.ints.len()];
            self.int_idx += 1;
            v
        }

        fn next_double(&mut self) -> f64 {
            let v = self.doubles[self.double_idx % self.doubles.len()];
            self.double_idx += 1;
            v
        }
    }

    struct MockLevel {
        blocks: HashMap<(i32, i32, i32), BlockState>,
        ocean_floor: i32,
        survivable: bool,
    }

    impl MockLevel {
        fn new(ocean_floor: i32) -> Self {
            Self {
                blocks: HashMap::new(),
                ocean_floor,
                survivable: true,
            }
        }

        fn set(&mut self, x: i32, y: i32, z: i32, state: BlockState) {
            self.blocks.insert((x, y, z), state);
        }

        fn get(&self, x: i32, y: i32, z: i32) -> BlockState {
            self.blocks
                .get(&(x, y, z))
                .copied()
                .unwrap_or(BlockState::Other)
        }
    }

    impl WorldGenLevel for MockLevel {
        fn get_height(&self, _heightmap: HeightmapType, _x: i32, _z: i32) -> i32 {
            self.ocean_floor
        }

        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            self.get(pos.x, pos.y, pos.z)
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState, _flags: i32) -> bool {
            self.set(pos.x, pos.y, pos.z, state);
            true
        }

        fn can_survive(&self, _state: BlockState, _pos: BlockPos) -> bool {
            self.survivable
        }
    }

    #[test]
    fn places_seagrass_when_water_and_survivable() {
        let mut level = MockLevel::new(5);
        level.set(10, 5, 20, BlockState::Water);
        let mut random = MockRandom::new(vec![0, 0, 0, 0], vec![0.5]);
        let config = ProbabilityFeatureConfiguration { probability: 0.3 };
        let placed = SeagrassFeature::new().place(
            &mut level,
            &mut random,
            BlockPos::new(10, 0, 20),
            config,
        );
        assert!(placed);
        assert_eq!(level.get(10, 5, 20), BlockState::Seagrass);
    }

    #[test]
    fn places_tall_seagrass_when_upper_is_water() {
        let mut level = MockLevel::new(5);
        level.set(10, 5, 20, BlockState::Water);
        level.set(10, 6, 20, BlockState::Water);
        let mut random = MockRandom::new(vec![0, 0, 0, 0], vec![0.0]);
        let config = ProbabilityFeatureConfiguration { probability: 1.0 };
        let placed = SeagrassFeature::new().place(
            &mut level,
            &mut random,
            BlockPos::new(10, 0, 20),
            config,
        );
        assert!(placed);
        assert_eq!(
            level.get(10, 5, 20),
            BlockState::TallSeagrass(DoubleBlockHalf::Lower)
        );
        assert_eq!(
            level.get(10, 6, 20),
            BlockState::TallSeagrass(DoubleBlockHalf::Upper)
        );
    }

    #[test]
    fn tall_seagrass_returns_true_even_if_upper_blocked() {
        let mut level = MockLevel::new(5);
        level.set(10, 5, 20, BlockState::Water);
        let mut random = MockRandom::new(vec![0, 0, 0, 0], vec![0.0]);
        let config = ProbabilityFeatureConfiguration { probability: 1.0 };
        let placed = SeagrassFeature::new().place(
            &mut level,
            &mut random,
            BlockPos::new(10, 0, 20),
            config,
        );
        assert!(placed);
        assert_eq!(level.get(10, 5, 20), BlockState::Water);
        assert_eq!(level.get(10, 6, 20), BlockState::Other);
    }

    #[test]
    fn returns_false_when_not_water_or_not_survivable() {
        let mut level = MockLevel::new(5);
        level.set(10, 5, 20, BlockState::Other);
        let mut random = MockRandom::new(vec![0, 0, 0, 0], vec![0.0]);
        let config = ProbabilityFeatureConfiguration { probability: 1.0 };
        let placed = SeagrassFeature::new().place(
            &mut level,
            &mut random,
            BlockPos::new(10, 0, 20),
            config,
        );
        assert!(!placed);

        let mut level2 = MockLevel::new(5);
        level2.set(10, 5, 20, BlockState::Water);
        level2.survivable = false;
        let mut random2 = MockRandom::new(vec![0, 0, 0, 0], vec![0.0]);
        let placed2 = SeagrassFeature::new().place(
            &mut level2,
            &mut random2,
            BlockPos::new(10, 0, 20),
            config,
        );
        assert!(!placed2);
    }

    #[test]
    fn negative_offset_and_wrapping_coordinates() {
        let mut level = MockLevel::new(-10);
        let x = i32::MAX.wrapping_add(-6); // 1 - 7
        let z = (-20i32).wrapping_add(-3); // 2 - 5
        level.set(x, -10, z, BlockState::Water);
        let mut random = MockRandom::new(vec![1, 7, 2, 5], vec![0.9]);
        let config = ProbabilityFeatureConfiguration { probability: 0.5 };
        let placed = SeagrassFeature::new().place(
            &mut level,
            &mut random,
            BlockPos::new(i32::MAX, 0, -20),
            config,
        );
        assert!(placed);
        assert_eq!(level.get(x, -10, z), BlockState::Seagrass);
    }

    #[test]
    fn probability_comparison_uses_double_semantics() {
        let mut level = MockLevel::new(0);
        level.set(0, 0, 0, BlockState::Water);
        let mut random = MockRandom::new(vec![0, 0, 0, 0], vec![0.5]);
        let config = ProbabilityFeatureConfiguration { probability: 0.5 };
        let placed = SeagrassFeature::new().place(
            &mut level,
            &mut random,
            BlockPos::new(0, 0, 0),
            config,
        );
        assert!(placed);
        // 0.5 < 0.5f32 as f64 is false, so seagrass (not tall)
        assert_eq!(level.get(0, 0, 0), BlockState::Seagrass);
    }
}
