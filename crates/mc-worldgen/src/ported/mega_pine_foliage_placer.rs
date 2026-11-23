#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn get_z(&self) -> i32 {
        self.z
    }
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntProvider {
    value: i32,
}

impl IntProvider {
    pub fn constant(value: i32) -> Self {
        Self { value }
    }

    pub fn sample(&self, _random: &mut impl RandomSource) -> i32 {
        self.value
    }
}

#[derive(Debug, Clone, Default)]
pub struct TreeConfiguration;

#[derive(Debug, Clone, Default)]
pub struct LevelSimulatedReader;

pub trait FoliageSetter {
    fn set_leaf(&mut self, pos: BlockPos);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoliageAttachment {
    pos: BlockPos,
    radius_offset: i32,
    double_trunk: bool,
}

impl FoliageAttachment {
    pub fn new(pos: BlockPos, radius_offset: i32, double_trunk: bool) -> Self {
        Self {
            pos,
            radius_offset,
            double_trunk,
        }
    }

    pub fn pos(&self) -> BlockPos {
        self.pos
    }

    pub fn radius_offset(&self) -> i32 {
        self.radius_offset
    }

    pub fn double_trunk(&self) -> bool {
        self.double_trunk
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoliagePlacerType {
    pub id: &'static str,
}

impl FoliagePlacerType {
    pub const MEGA_PINE_FOLIAGE_PLACER: Self = Self {
        id: "mega_pine_foliage_placer",
    };
}

#[derive(Debug, Clone)]
pub struct MegaPineFoliagePlacer {
    pub radius: IntProvider,
    pub offset: IntProvider,
    pub crown_height: IntProvider,
}

impl MegaPineFoliagePlacer {
    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.MegaPineFoliagePlacer#<init>
    pub fn new(radius: IntProvider, offset: IntProvider, crown_height: IntProvider) -> Self {
        Self {
            radius,
            offset,
            crown_height,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.MegaPineFoliagePlacer#type
    pub fn placer_type(&self) -> FoliagePlacerType {
        FoliagePlacerType::MEGA_PINE_FOLIAGE_PLACER
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.MegaPineFoliagePlacer#createFoliage
    pub fn create_foliage(
        &self,
        level: &LevelSimulatedReader,
        setter: &mut impl FoliageSetter,
        random: &mut impl RandomSource,
        config: &TreeConfiguration,
        _max_free_tree_height: i32,
        attachment: &FoliageAttachment,
        foliage_height: i32,
        foliage_radius: i32,
        offset_y: i32,
    ) {
        let pos = attachment.pos();
        let mut prev_radius: i32 = 0;
        let start = pos
            .get_y()
            .wrapping_sub(foliage_height)
            .wrapping_add(offset_y);
        let end = pos.get_y().wrapping_add(offset_y);
        let mut y = start;
        while y <= end {
            let rel_y = pos.get_y().wrapping_sub(y);
            let f = ((rel_y as f32) / (foliage_height as f32)) * 3.5f32;
            let radius_base = foliage_radius
                .wrapping_add(attachment.radius_offset())
                .wrapping_add(mth_floor_f32(f));
            let radius = if rel_y > 0 && radius_base == prev_radius && (y & 1) == 0 {
                radius_base.wrapping_add(1)
            } else {
                radius_base
            };
            self.place_leaves_row(
                level,
                setter,
                random,
                config,
                BlockPos::new(pos.get_x(), y, pos.get_z()),
                radius,
                0,
                attachment.double_trunk(),
            );
            prev_radius = radius_base;
            if y == end {
                break;
            }
            y = y.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.MegaPineFoliagePlacer#foliageHeight
    pub fn foliage_height(
        &self,
        random: &mut impl RandomSource,
        _height: i32,
        _config: &TreeConfiguration,
    ) -> i32 {
        self.crown_height.sample(random)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.MegaPineFoliagePlacer#shouldSkipLocation
    pub fn should_skip_location(
        &self,
        _random: &mut impl RandomSource,
        local_x: i32,
        _local_y: i32,
        local_z: i32,
        range: i32,
        _large: bool,
    ) -> bool {
        let sum_xz = local_x.wrapping_add(local_z);
        if sum_xz >= 7 {
            return true;
        }
        let dist_sq = local_x
            .wrapping_mul(local_x)
            .wrapping_add(local_z.wrapping_mul(local_z));
        let range_sq = range.wrapping_mul(range);
        dist_sq > range_sq
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacer#shouldSkipLocationSigned
    fn should_skip_location_signed(
        &self,
        random: &mut impl RandomSource,
        local_x: i32,
        local_y: i32,
        local_z: i32,
        range: i32,
        large: bool,
    ) -> bool {
        let offset = if large { 1i32 } else { 0i32 };
        let ax = if local_x >= 0 {
            local_x.wrapping_add(offset)
        } else {
            local_x
        };
        let az = if local_z >= 0 {
            local_z.wrapping_add(offset)
        } else {
            local_z
        };
        self.should_skip_location(random, ax, local_y, az, range, large)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacer#placeLeavesRow
    fn place_leaves_row(
        &self,
        level: &LevelSimulatedReader,
        setter: &mut impl FoliageSetter,
        random: &mut impl RandomSource,
        config: &TreeConfiguration,
        pos: BlockPos,
        range: i32,
        y_offset: i32,
        double_trunk: bool,
    ) {
        let i = if double_trunk { 1i32 } else { 0i32 };
        let end = range.wrapping_add(i);
        let mut j = range.wrapping_neg();
        while j <= end {
            let mut k = range.wrapping_neg();
            while k <= end {
                if !self.should_skip_location_signed(random, j, y_offset, k, range, double_trunk) {
                    let leaf_pos = BlockPos::new(
                        pos.x.wrapping_add(j),
                        pos.y.wrapping_add(y_offset),
                        pos.z.wrapping_add(k),
                    );
                    try_place_leaf(level, setter, random, config, leaf_pos);
                }
                k = k.wrapping_add(1);
            }
            j = j.wrapping_add(1);
        }
    }
}

// JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacer#tryPlaceLeaf
fn try_place_leaf(
    _level: &LevelSimulatedReader,
    setter: &mut impl FoliageSetter,
    _random: &mut impl RandomSource,
    _config: &TreeConfiguration,
    pos: BlockPos,
) -> bool {
    setter.set_leaf(pos);
    true
}

// JAVA: net.minecraft.util.Mth#floor
fn mth_floor_f32(value: f32) -> i32 {
    let i = value as i32;
    if value < i as f32 {
        i.wrapping_sub(1)
    } else {
        i
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::vec::Vec;

    struct SeqRandom {
        ints: Vec<i32>,
        idx: usize,
    }

    impl SeqRandom {
        fn new(ints: Vec<i32>) -> Self {
            Self { ints, idx: 0 }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            let v = if self.idx < self.ints.len() {
                self.ints[self.idx]
            } else {
                0
            };
            self.idx += 1;
            v.rem_euclid(bound)
        }
    }

    #[derive(Default)]
    struct TestSetter {
        positions: Vec<BlockPos>,
    }

    impl FoliageSetter for TestSetter {
        fn set_leaf(&mut self, pos: BlockPos) {
            self.positions.push(pos);
        }
    }

    #[test]
    fn foliage_height_samples_crown_height() {
        let p = MegaPineFoliagePlacer::new(
            IntProvider::constant(0),
            IntProvider::constant(0),
            IntProvider::constant(5),
        );
        let mut r = SeqRandom::new(Vec::new());
        assert_eq!(p.foliage_height(&mut r, 10, &TreeConfiguration), 5);
    }

    #[test]
    fn should_skip_location_logic() {
        let p = MegaPineFoliagePlacer::new(
            IntProvider::constant(0),
            IntProvider::constant(0),
            IntProvider::constant(5),
        );
        let mut r = SeqRandom::new(Vec::new());
        assert!(p.should_skip_location(&mut r, 3, 0, 4, 10, false));
        assert!(p.should_skip_location(&mut r, 2, 0, 2, 2, false));
        assert!(!p.should_skip_location(&mut r, 1, 0, 1, 2, false));
    }

    #[test]
    fn create_foliage_basic_shape() {
        let p = MegaPineFoliagePlacer::new(
            IntProvider::constant(0),
            IntProvider::constant(0),
            IntProvider::constant(5),
        );
        let mut setter = TestSetter::default();
        let mut random = SeqRandom::new(Vec::new());
        let attachment = FoliageAttachment::new(BlockPos::new(0, 10, 0), 0, false);
        p.create_foliage(
            &LevelSimulatedReader,
            &mut setter,
            &mut random,
            &TreeConfiguration,
            10,
            &attachment,
            3,
            0,
            0,
        );
        assert_eq!(setter.positions.len(), 48);
        let mut by_y: BTreeMap<i32, usize> = BTreeMap::new();
        for pos in &setter.positions {
            *by_y.entry(pos.y).or_insert(0) += 1;
        }
        assert_eq!(by_y.get(&7), Some(&29));
        assert_eq!(by_y.get(&8), Some(&13));
        assert_eq!(by_y.get(&9), Some(&5));
        assert_eq!(by_y.get(&10), Some(&1));
    }

    #[test]
    fn create_foliage_even_y_radius_bump() {
        let p = MegaPineFoliagePlacer::new(
            IntProvider::constant(0),
            IntProvider::constant(0),
            IntProvider::constant(5),
        );
        let mut setter = TestSetter::default();
        let mut random = SeqRandom::new(Vec::new());
        let attachment = FoliageAttachment::new(BlockPos::new(0, 10, 0), 0, false);
        p.create_foliage(
            &LevelSimulatedReader,
            &mut setter,
            &mut random,
            &TreeConfiguration,
            10,
            &attachment,
            6,
            0,
            0,
        );
        let mut by_y: BTreeMap<i32, usize> = BTreeMap::new();
        for pos in &setter.positions {
            *by_y.entry(pos.y).or_insert(0) += 1;
        }
        assert_eq!(by_y.get(&6), Some(&29));
        assert_eq!(by_y.get(&8), Some(&13));
    }

    #[test]
    fn float_operation_order_and_floor() {
        let v = ((2i32 as f32) / (3i32 as f32)) * 3.5f32;
        assert_eq!(v.to_bits(), 2.3333335f32.to_bits());
        assert_eq!(mth_floor_f32(v), 2);
        assert_eq!(mth_floor_f32(-0.5f32), -1);
        assert_eq!(mth_floor_f32(f32::NAN), 0);
    }

    #[test]
    fn should_skip_location_wrapping_and_negative_height() {
        let p = MegaPineFoliagePlacer::new(
            IntProvider::constant(0),
            IntProvider::constant(0),
            IntProvider::constant(1),
        );
        let mut r = SeqRandom::new(Vec::new());
        // sum wraps to i32::MIN; dist_sq wraps: i32::MAX*i32::MAX=1, +1=2, range_sq=1 => skip
        assert!(p.should_skip_location(&mut r, i32::MAX, 0, 1, 1, false));

        let mut setter = TestSetter::default();
        let attachment = FoliageAttachment::new(BlockPos::new(0, 10, 0), 0, false);
        p.create_foliage(
            &LevelSimulatedReader,
            &mut setter,
            &mut SeqRandom::new(Vec::new()),
            &TreeConfiguration,
            10,
            &attachment,
            -1,
            0,
            0,
        );
        assert!(setter.positions.is_empty());
    }
}
