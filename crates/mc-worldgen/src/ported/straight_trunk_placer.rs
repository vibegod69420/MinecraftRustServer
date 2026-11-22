use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#new
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#below
    pub fn below(&self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_sub(1),
            z: self.z,
        }
    }

    // JAVA: net.minecraft.core.BlockPos#above
    pub fn above(&self, dy: i32) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(dy),
            z: self.z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockState(pub u32);

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
    fn next_float(&mut self) -> f32;
}

pub trait LevelSimulatedReader {
    fn is_air(&self, pos: BlockPos) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeConfiguration {
    pub trunk_state: BlockState,
    pub dirt_state: BlockState,
}

impl Default for TreeConfiguration {
    fn default() -> Self {
        Self {
            trunk_state: BlockState(1),
            dirt_state: BlockState(2),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoliageAttachment {
    pub pos: BlockPos,
    pub radius_offset: i32,
    pub double_trunk: bool,
}

impl FoliageAttachment {
    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacer$FoliageAttachment#<init>
    pub fn new(pos: BlockPos, radius_offset: i32, double_trunk: bool) -> Self {
        Self {
            pos,
            radius_offset,
            double_trunk,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrunkPlacerTypeId {
    StraightTrunkPlacer,
}

impl fmt::Display for TrunkPlacerTypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrunkPlacerTypeId::StraightTrunkPlacer => write!(f, "straight_trunk_placer"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StraightTrunkPlacer {
    pub base_height: i32,
    pub height_rand_a: i32,
    pub height_rand_b: i32,
}

impl StraightTrunkPlacer {
    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.StraightTrunkPlacer#<init>
    pub fn new(base_height: i32, height_rand_a: i32, height_rand_b: i32) -> Self {
        Self {
            base_height,
            height_rand_a,
            height_rand_b,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.StraightTrunkPlacer#type
    pub fn type_(&self) -> TrunkPlacerTypeId {
        TrunkPlacerTypeId::StraightTrunkPlacer
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.TrunkPlacer#setDirtAt
    pub fn set_dirt_at(
        _level: &dyn LevelSimulatedReader,
        block_setter: &mut dyn FnMut(BlockPos, BlockState),
        _random: &mut dyn RandomSource,
        pos: BlockPos,
        config: &TreeConfiguration,
    ) {
        block_setter(pos, config.dirt_state);
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.TrunkPlacer#placeLog
    pub fn place_log(
        &self,
        _level: &dyn LevelSimulatedReader,
        block_setter: &mut dyn FnMut(BlockPos, BlockState),
        _random: &mut dyn RandomSource,
        pos: BlockPos,
        config: &TreeConfiguration,
    ) -> bool {
        block_setter(pos, config.trunk_state);
        true
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.TrunkPlacer#getTreeHeight
    pub fn get_tree_height(&self, random: &mut dyn RandomSource) -> i32 {
        self.base_height
            .wrapping_add(random.next_int(self.height_rand_a.wrapping_add(1)))
            .wrapping_add(random.next_int(self.height_rand_b.wrapping_add(1)))
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.StraightTrunkPlacer#placeTrunk
    pub fn place_trunk(
        &self,
        level: &dyn LevelSimulatedReader,
        block_setter: &mut dyn FnMut(BlockPos, BlockState),
        random: &mut dyn RandomSource,
        free_tree_height: i32,
        pos: BlockPos,
        config: &TreeConfiguration,
    ) -> Vec<FoliageAttachment> {
        Self::set_dirt_at(level, block_setter, random, pos.below(), config);

        let mut i: i32 = 0;
        while i < free_tree_height {
            self.place_log(level, block_setter, random, pos.above(i), config);
            i = i.wrapping_add(1);
        }

        vec![FoliageAttachment::new(
            pos.above(free_tree_height),
            0,
            false,
        )]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLevel;
    impl LevelSimulatedReader for MockLevel {
        fn is_air(&self, _pos: BlockPos) -> bool {
            true
        }
    }

    struct MockRandom;
    impl RandomSource for MockRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            0
        }
        fn next_float(&mut self) -> f32 {
            0.0
        }
    }

    #[test]
    fn test_place_trunk_basic() {
        let placer = StraightTrunkPlacer::new(5, 2, 0);
        let level = MockLevel;
        let mut random = MockRandom;
        let mut placed: Vec<(BlockPos, BlockState)> = Vec::new();
        let mut setter = |p: BlockPos, s: BlockState| placed.push((p, s));
        let config = TreeConfiguration::default();
        let pos = BlockPos::new(10, 64, 20);

        let attachments = placer.place_trunk(&level, &mut setter, &mut random, 5, pos, &config);

        assert_eq!(placed[0], (BlockPos::new(10, 63, 20), config.dirt_state));
        for i in 0..5i32 {
            assert_eq!(
                placed[(1 + i) as usize],
                (BlockPos::new(10, 64 + i, 20), config.trunk_state)
            );
        }
        assert_eq!(placed.len(), 6);
        assert_eq!(attachments.len(), 1);
        assert_eq!(
            attachments[0],
            FoliageAttachment::new(BlockPos::new(10, 69, 20), 0, false)
        );
    }

    #[test]
    fn test_place_trunk_zero_height() {
        let placer = StraightTrunkPlacer::new(0, 0, 0);
        let level = MockLevel;
        let mut random = MockRandom;
        let mut placed: Vec<(BlockPos, BlockState)> = Vec::new();
        let mut setter = |p: BlockPos, s: BlockState| placed.push((p, s));
        let config = TreeConfiguration::default();
        let pos = BlockPos::new(0, 10, 0);

        let attachments = placer.place_trunk(&level, &mut setter, &mut random, 0, pos, &config);

        assert_eq!(placed.len(), 1);
        assert_eq!(placed[0].0, BlockPos::new(0, 9, 0));
        assert_eq!(attachments[0].pos, BlockPos::new(0, 10, 0));
    }

    #[test]
    fn test_place_trunk_negative_height() {
        let placer = StraightTrunkPlacer::new(1, 0, 0);
        let level = MockLevel;
        let mut random = MockRandom;
        let mut placed: Vec<(BlockPos, BlockState)> = Vec::new();
        let mut setter = |p: BlockPos, s: BlockState| placed.push((p, s));
        let config = TreeConfiguration::default();
        let pos = BlockPos::new(0, 5, 0);

        let attachments = placer.place_trunk(&level, &mut setter, &mut random, -1, pos, &config);

        assert_eq!(placed.len(), 1);
        assert_eq!(attachments[0].pos, pos.above(-1));
        assert_eq!(attachments[0].pos.y, 4);
    }

    #[test]
    fn test_place_trunk_y_overflow() {
        let placer = StraightTrunkPlacer::new(1, 0, 0);
        let level = MockLevel;
        let mut random = MockRandom;
        let mut placed: Vec<(BlockPos, BlockState)> = Vec::new();
        let mut setter = |p: BlockPos, s: BlockState| placed.push((p, s));
        let config = TreeConfiguration::default();
        let pos = BlockPos::new(0, i32::MAX, 0);

        let attachments = placer.place_trunk(&level, &mut setter, &mut random, 1, pos, &config);

        assert_eq!(placed[0].0.y, i32::MAX.wrapping_sub(1));
        assert_eq!(placed[1].0.y, i32::MAX);
        assert_eq!(attachments[0].pos.y, i32::MAX.wrapping_add(1));
    }

    #[test]
    fn test_type_constructor_and_tree_height() {
        struct SeqRandom {
            vals: [i32; 2],
            idx: usize,
        }
        impl RandomSource for SeqRandom {
            fn next_int(&mut self, _bound: i32) -> i32 {
                let v = self.vals[self.idx];
                self.idx += 1;
                v
            }
            fn next_float(&mut self) -> f32 {
                0.0
            }
        }

        let placer = StraightTrunkPlacer::new(4, 1, 2);
        assert_eq!(placer.base_height, 4);
        assert_eq!(placer.height_rand_a, 1);
        assert_eq!(placer.height_rand_b, 2);
        assert_eq!(placer.type_(), TrunkPlacerTypeId::StraightTrunkPlacer);

        let mut random = SeqRandom { vals: [1, 2], idx: 0 };
        assert_eq!(placer.get_tree_height(&mut random), 7);
    }

    #[test]
    fn test_get_tree_height_wrapping() {
        struct FixedRandom(i32);
        impl RandomSource for FixedRandom {
            fn next_int(&mut self, _bound: i32) -> i32 {
                self.0
            }
            fn next_float(&mut self) -> f32 {
                0.0
            }
        }

        let placer = StraightTrunkPlacer::new(i32::MAX, 0, 0);
        let mut random = FixedRandom(1);
        assert_eq!(
            placer.get_tree_height(&mut random),
            i32::MAX.wrapping_add(1).wrapping_add(1)
        );
    }
}
