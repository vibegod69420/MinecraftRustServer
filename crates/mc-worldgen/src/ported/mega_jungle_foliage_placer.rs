//! Port of `net.minecraft.world.level.levelgen.feature.foliageplacers.MegaJungleFoliagePlacer`.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#<init>(int,int,int)
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#offset(int,int,int)
    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
        Self {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }

    // JAVA: net.minecraft.core.BlockPos#above(int)
    pub fn above(&self, dy: i32) -> Self {
        self.offset(0, dy, 0)
    }
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntProvider {
    Constant(i32),
}

impl IntProvider {
    // JAVA: net.minecraft.util.valueproviders.IntProvider#sample
    pub fn sample(&self, _random: &mut dyn RandomSource) -> i32 {
        match self {
            IntProvider::Constant(v) => *v,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TreeConfiguration;

pub trait LevelSimulatedReader {}

pub trait FoliageSetter {
    fn set(&mut self, pos: BlockPos);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoliagePlacerType {
    MegaJungleFoliagePlacer,
}

#[derive(Debug, Clone)]
pub struct MegaJungleFoliagePlacer {
    pub radius: IntProvider,
    pub offset: IntProvider,
    pub height: i32,
}

impl MegaJungleFoliagePlacer {
    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.MegaJungleFoliagePlacer#<init>
    pub fn new(radius: IntProvider, offset: IntProvider, height: i32) -> Self {
        Self {
            radius,
            offset,
            height,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.MegaJungleFoliagePlacer#type
    pub fn foliage_placer_type(&self) -> FoliagePlacerType {
        FoliagePlacerType::MegaJungleFoliagePlacer
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacer#shouldSkipLocationSigned
    fn should_skip_location_signed(
        &self,
        random: &mut dyn RandomSource,
        local_x: i32,
        local_y: i32,
        local_z: i32,
        range: i32,
        large: bool,
    ) -> bool {
        let (dx, dz) = if large {
            (
                local_x
                    .wrapping_abs()
                    .min(local_x.wrapping_sub(1).wrapping_abs()),
                local_z
                    .wrapping_abs()
                    .min(local_z.wrapping_sub(1).wrapping_abs()),
            )
        } else {
            (local_x.wrapping_abs(), local_z.wrapping_abs())
        };
        self.should_skip_location(random, dx, local_y, dz, range, large)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacer#placeLeavesRow
    pub fn place_leaves_row(
        &self,
        _level: &dyn LevelSimulatedReader,
        setter: &mut dyn FoliageSetter,
        random: &mut dyn RandomSource,
        _config: &TreeConfiguration,
        pos: BlockPos,
        range: i32,
        local_y: i32,
        double_trunk: bool,
    ) {
        let i = if double_trunk { 1_i32 } else { 0_i32 };
        let end = range.wrapping_add(i);
        let mut j = range.wrapping_neg();
        while j <= end {
            let mut k = range.wrapping_neg();
            while k <= end {
                if !self.should_skip_location_signed(random, j, local_y, k, range, double_trunk) {
                    setter.set(pos.offset(j, local_y, k));
                }
                k = k.wrapping_add(1);
            }
            j = j.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.MegaJungleFoliagePlacer#createFoliage
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
        offset: i32,
    ) {
        let v = if attachment.double_trunk() {
            foliage_height
        } else {
            1_i32.wrapping_add(random.next_int(2))
        };

        let bottom = offset.wrapping_sub(v);
        let mut y = offset;
        while y >= bottom {
            let radius = foliage_radius
                .wrapping_add(attachment.radius_offset())
                .wrapping_add(1)
                .wrapping_sub(y);
            self.place_leaves_row(
                level,
                setter,
                random,
                config,
                attachment.pos(),
                radius,
                y,
                attachment.double_trunk(),
            );
            y = y.wrapping_sub(1);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.MegaJungleFoliagePlacer#foliageHeight
    pub fn foliage_height(
        &self,
        _random: &mut dyn RandomSource,
        _height: i32,
        _config: &TreeConfiguration,
    ) -> i32 {
        self.height
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.MegaJungleFoliagePlacer#shouldSkipLocation
    pub fn should_skip_location(
        &self,
        _random: &mut dyn RandomSource,
        local_x: i32,
        _local_y: i32,
        local_z: i32,
        range: i32,
        _large: bool,
    ) -> bool {
        if local_x.wrapping_add(local_z) >= 7 {
            return true;
        }
        local_x
            .wrapping_mul(local_x)
            .wrapping_add(local_z.wrapping_mul(local_z))
            > range.wrapping_mul(range)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    struct TestRandom;

    impl RandomSource for TestRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            0
        }
    }

    struct FixedRandom(i32);

    impl RandomSource for FixedRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            self.0
        }
    }

    struct CollectingSetter {
        positions: BTreeSet<(i32, i32, i32)>,
    }

    impl CollectingSetter {
        fn new() -> Self {
            Self {
                positions: BTreeSet::new(),
            }
        }
    }

    impl FoliageSetter for CollectingSetter {
        fn set(&mut self, pos: BlockPos) {
            self.positions.insert((pos.x, pos.y, pos.z));
        }
    }

    struct DummyLevel;
    impl LevelSimulatedReader for DummyLevel {}

    #[test]
    fn mega_jungle_skip_when_sum_ge_7() {
        let placer =
            MegaJungleFoliagePlacer::new(IntProvider::Constant(0), IntProvider::Constant(0), 2);
        let mut rng = TestRandom;
        assert!(placer.should_skip_location(&mut rng, 4, 0, 3, 10, false));
        assert!(placer.should_skip_location(&mut rng, 3, 0, 4, 10, false));
        assert!(!placer.should_skip_location(&mut rng, 3, 0, 3, 10, false));
    }

    #[test]
    fn mega_jungle_skip_by_squared_distance_and_negative() {
        let placer =
            MegaJungleFoliagePlacer::new(IntProvider::Constant(0), IntProvider::Constant(0), 2);
        let mut rng = TestRandom;
        assert!(placer.should_skip_location(&mut rng, 2, 0, 2, 2, false));
        assert!(!placer.should_skip_location(&mut rng, 1, 0, 1, 2, false));
        assert!(placer.should_skip_location(&mut rng, -4, 0, -4, 5, false));
        assert!(!placer.should_skip_location(&mut rng, -1, 0, -1, 2, false));
    }

    #[test]
    fn mega_jungle_double_trunk_uses_foliage_height() {
        let placer =
            MegaJungleFoliagePlacer::new(IntProvider::Constant(0), IntProvider::Constant(0), 2);
        let mut rng = TestRandom;
        let level = DummyLevel;
        let config = TreeConfiguration;
        let attachment = FoliageAttachment::new(BlockPos::new(0, 10, 0), 0, true);
        let mut setter = CollectingSetter::new();
        placer.create_foliage(&level, &mut setter, &mut rng, &config, 0, &attachment, 3, 2, 0);
        let has_layer_minus3 = setter.positions.iter().any(|&(_, y, _)| y == 7);
        assert!(has_layer_minus3);
    }

    #[test]
    fn mega_jungle_non_double_trunk_uses_random() {
        let placer =
            MegaJungleFoliagePlacer::new(IntProvider::Constant(0), IntProvider::Constant(0), 2);
        let level = DummyLevel;
        let config = TreeConfiguration;
        let attachment = FoliageAttachment::new(BlockPos::new(0, 10, 0), 0, false);

        let mut rng0 = FixedRandom(0);
        let mut setter0 = CollectingSetter::new();
        placer.create_foliage(&level, &mut setter0, &mut rng0, &config, 0, &attachment, 5, 2, 0);
        let ys0: BTreeSet<i32> = setter0.positions.iter().map(|p| p.1).collect();
        assert!(ys0.contains(&10));
        assert!(ys0.contains(&9));
        assert!(!ys0.iter().any(|&y| y < 9));

        let mut rng1 = FixedRandom(1);
        let mut setter1 = CollectingSetter::new();
        placer.create_foliage(&level, &mut setter1, &mut rng1, &config, 0, &attachment, 5, 2, 0);
        let ys1: BTreeSet<i32> = setter1.positions.iter().map(|p| p.1).collect();
        assert!(ys1.contains(&8));
    }

    #[test]
    fn mega_jungle_foliage_height_and_overflow() {
        let placer =
            MegaJungleFoliagePlacer::new(IntProvider::Constant(1), IntProvider::Constant(1), 7);
        let mut rng = TestRandom;
        let config = TreeConfiguration;
        assert_eq!(placer.foliage_height(&mut rng, 5, &config), 7);

        let x = i32::MAX;
        let z = 1_i32;
        let sum = x.wrapping_add(z);
        assert!(sum < 7);
        let _ = placer.should_skip_location(&mut rng, x, 0, z, 10, false);
    }
}
