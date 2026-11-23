//! Port of `net.minecraft.world.level.levelgen.feature.foliageplacers.RandomSpreadFoliagePlacer`.
//!
//! Self-contained module with minimal local stubs for unported Minecraft types.

use std::fmt::Debug;

// ---------------------------------------------------------------------------
// Minimal stubs for external Minecraft types
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#new (constructor)
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#mutable
    pub fn mutable(self) -> MutableBlockPos {
        MutableBlockPos {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }

    // JAVA: net.minecraft.core.BlockPos#above(int)
    pub fn above(self, dy: i32) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(dy),
            z: self.z,
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
    // JAVA: net.minecraft.core.BlockPos$MutableBlockPos#setWithOffset
    pub fn set_with_offset(&mut self, pos: BlockPos, dx: i32, dy: i32, dz: i32) -> &mut Self {
        self.x = pos.x.wrapping_add(dx);
        self.y = pos.y.wrapping_add(dy);
        self.z = pos.z.wrapping_add(dz);
        self
    }

    // JAVA: net.minecraft.core.BlockPos$MutableBlockPos#immutable (equivalent conversion)
    pub fn immutable(self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }
}

pub trait RandomSource {
    // JAVA: net.minecraft.util.RandomSource#nextInt(int)
    fn next_int(&mut self, bound: i32) -> i32;
    // JAVA: net.minecraft.util.RandomSource#nextBoolean
    fn next_boolean(&mut self) -> bool;
}

pub trait IntProvider: Debug {
    // JAVA: net.minecraft.util.valueproviders.IntProvider#sample
    fn sample(&self, random: &mut dyn RandomSource) -> i32;
}

#[derive(Clone, Debug, Default)]
pub struct TreeConfiguration;

pub trait LevelSimulatedReader {
    // JAVA: net.minecraft.world.level.levelgen.feature.TreeFeature#validTreePos (abstracted)
    fn is_valid_tree_pos(&self, pos: BlockPos) -> bool;
}

pub trait FoliageSetter {
    /// Abstracts `config.foliageProvider.getState(...)` + `setter.set(...)`.
    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacer$FoliageSetter#set
    fn set(&mut self, pos: BlockPos, random: &mut dyn RandomSource, config: &TreeConfiguration);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoliageAttachment {
    pos: BlockPos,
    radius_offset: i32,
    double_trunk: bool,
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

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacer$FoliageAttachment#pos
    pub fn pos(&self) -> BlockPos {
        self.pos
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacer$FoliageAttachment#radiusOffset
    pub fn radius_offset(&self) -> i32 {
        self.radius_offset
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacer$FoliageAttachment#doubleTrunk
    pub fn double_trunk(&self) -> bool {
        self.double_trunk
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoliagePlacerType {
    RandomSpreadFoliagePlacer,
}

// JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacer#tryPlaceLeaf
pub fn try_place_leaf(
    level: &dyn LevelSimulatedReader,
    setter: &mut dyn FoliageSetter,
    random: &mut dyn RandomSource,
    config: &TreeConfiguration,
    pos: BlockPos,
) -> bool {
    if !level.is_valid_tree_pos(pos) {
        return false;
    }
    setter.set(pos, random, config);
    true
}

// ---------------------------------------------------------------------------
// RandomSpreadFoliagePlacer
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct RandomSpreadFoliagePlacer {
    pub radius: Box<dyn IntProvider>,
    pub offset: Box<dyn IntProvider>,
    pub foliage_height: Box<dyn IntProvider>,
    pub leaf_placement_attempts: i32,
}

impl RandomSpreadFoliagePlacer {
    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.RandomSpreadFoliagePlacer#<init>
    pub fn new(
        radius: Box<dyn IntProvider>,
        offset: Box<dyn IntProvider>,
        foliage_height: Box<dyn IntProvider>,
        leaf_placement_attempts: i32,
    ) -> Self {
        Self {
            radius,
            offset,
            foliage_height,
            leaf_placement_attempts,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.RandomSpreadFoliagePlacer#type
    pub fn foliage_placer_type(&self) -> FoliagePlacerType {
        FoliagePlacerType::RandomSpreadFoliagePlacer
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.RandomSpreadFoliagePlacer#createFoliage
    pub fn create_foliage(
        &self,
        level: &dyn LevelSimulatedReader,
        setter: &mut dyn FoliageSetter,
        random: &mut dyn RandomSource,
        config: &TreeConfiguration,
        _max_free_tree_height: i32,
        attachment: &FoliageAttachment,
        foliage_height: i32,
        foliage_radius: i32,
        _offset: i32,
    ) {
        let pos = attachment.pos();
        let mut mutable = pos.mutable();

        let mut i = 0i32;
        while i < self.leaf_placement_attempts {
            let dx = random
                .next_int(foliage_radius)
                .wrapping_sub(random.next_int(foliage_radius));
            let dy = random
                .next_int(foliage_height)
                .wrapping_sub(random.next_int(foliage_height));
            let dz = random
                .next_int(foliage_radius)
                .wrapping_sub(random.next_int(foliage_radius));
            mutable.set_with_offset(pos, dx, dy, dz);
            try_place_leaf(level, setter, random, config, mutable.immutable());
            i = i.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.RandomSpreadFoliagePlacer#foliageHeight
    pub fn foliage_height(&self, random: &mut dyn RandomSource, _height: i32, _config: &TreeConfiguration) -> i32 {
        self.foliage_height.sample(random)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.RandomSpreadFoliagePlacer#shouldSkipLocation
    pub fn should_skip_location(
        &self,
        _random: &mut dyn RandomSource,
        _local_x: i32,
        _local_y: i32,
        _local_z: i32,
        _range: i32,
        _large: bool,
    ) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    #[derive(Debug)]
    struct SeqRandom {
        ints: RefCell<VecDeque<i32>>,
        bools: RefCell<VecDeque<bool>>,
    }

    impl SeqRandom {
        fn new(ints: Vec<i32>, bools: Vec<bool>) -> Self {
            Self {
                ints: RefCell::new(VecDeque::from(ints)),
                bools: RefCell::new(VecDeque::from(bools)),
            }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            let v = self.ints.borrow_mut().pop_front().unwrap_or(0);
            if bound <= 0 {
                0
            } else {
                v.rem_euclid(bound)
            }
        }

        fn next_boolean(&mut self) -> bool {
            self.bools.borrow_mut().pop_front().unwrap_or(false)
        }
    }

    #[derive(Debug)]
    struct ConstInt(i32);
    impl IntProvider for ConstInt {
        fn sample(&self, _random: &mut dyn RandomSource) -> i32 {
            self.0
        }
    }

    struct AlwaysValidLevel;
    impl LevelSimulatedReader for AlwaysValidLevel {
        fn is_valid_tree_pos(&self, _pos: BlockPos) -> bool {
            true
        }
    }

    #[derive(Default)]
    struct CollectSetter {
        positions: Vec<BlockPos>,
    }
    impl FoliageSetter for CollectSetter {
        fn set(&mut self, pos: BlockPos, _random: &mut dyn RandomSource, _config: &TreeConfiguration) {
            self.positions.push(pos);
        }
    }

    #[test]
    fn foliage_height_samples_provider() {
        let placer = RandomSpreadFoliagePlacer::new(
            Box::new(ConstInt(2)),
            Box::new(ConstInt(0)),
            Box::new(ConstInt(7)),
            3,
        );
        let mut rng = SeqRandom::new(vec![], vec![]);
        assert_eq!(placer.foliage_height(&mut rng, 10, &TreeConfiguration), 7);
        assert_eq!(
            placer.foliage_placer_type(),
            FoliagePlacerType::RandomSpreadFoliagePlacer
        );
    }

    #[test]
    fn should_skip_location_always_false() {
        let placer = RandomSpreadFoliagePlacer::new(
            Box::new(ConstInt(1)),
            Box::new(ConstInt(0)),
            Box::new(ConstInt(1)),
            0,
        );
        let mut rng = SeqRandom::new(vec![], vec![]);
        assert!(!placer.should_skip_location(&mut rng, 0, 0, 0, 0, false));
        assert!(!placer.should_skip_location(&mut rng, -5, 10, i32::MAX, i32::MIN, true));
    }

    #[test]
    fn create_foliage_places_attempts_with_wrapping_offsets() {
        // 1 attempt, radius=3, height=4 -> 6 nextInt calls: dx1,dx2,dy1,dy2,dz1,dz2
        let placer = RandomSpreadFoliagePlacer::new(
            Box::new(ConstInt(2)),
            Box::new(ConstInt(0)),
            Box::new(ConstInt(4)),
            1,
        );
        let mut rng = SeqRandom::new(vec![2, 1, 3, 0, 0, 2], vec![]);
        let mut setter = CollectSetter::default();
        let level = AlwaysValidLevel;
        let attachment = FoliageAttachment::new(BlockPos::new(10, 20, 30), 0, false);
        placer.create_foliage(
            &level,
            &mut setter,
            &mut rng,
            &TreeConfiguration,
            5,
            &attachment,
            4,
            3,
            0,
        );
        // dx = 2-1 = 1, dy = 3-0 = 3, dz = 0-2 = -2
        assert_eq!(setter.positions, vec![BlockPos::new(11, 23, 28)]);
    }

    #[test]
    fn create_foliage_handles_negative_and_overflow_coordinates() {
        let placer = RandomSpreadFoliagePlacer::new(
            Box::new(ConstInt(1)),
            Box::new(ConstInt(0)),
            Box::new(ConstInt(1)),
            1,
        );
        // radius=2, height=2: offsets 1-0=1, 0-1=-1, 1-0=1
        let mut rng = SeqRandom::new(vec![1, 0, 0, 1, 1, 0], vec![]);
        let mut setter = CollectSetter::default();
        let level = AlwaysValidLevel;
        let base = BlockPos::new(i32::MAX, i32::MIN, -1);
        let attachment = FoliageAttachment::new(base, 0, false);
        placer.create_foliage(
            &level,
            &mut setter,
            &mut rng,
            &TreeConfiguration,
            0,
            &attachment,
            2,
            2,
            0,
        );
        assert_eq!(
            setter.positions,
            vec![BlockPos::new(
                i32::MAX.wrapping_add(1),
                i32::MIN.wrapping_sub(1),
                0
            )]
        );
    }

    #[test]
    fn try_place_leaf_respects_level_validity() {
        struct NeverValid;
        impl LevelSimulatedReader for NeverValid {
            fn is_valid_tree_pos(&self, _pos: BlockPos) -> bool {
                false
            }
        }
        let mut rng = SeqRandom::new(vec![], vec![]);
        let mut setter = CollectSetter::default();
        let ok = try_place_leaf(
            &NeverValid,
            &mut setter,
            &mut rng,
            &TreeConfiguration,
            BlockPos::new(1, 2, 3),
        );
        assert!(!ok);
        assert!(setter.positions.is_empty());
    }
}
