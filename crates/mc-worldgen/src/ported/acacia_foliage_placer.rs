use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#<init>
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#above
    pub fn above(&self, dy: i32) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(dy),
            z: self.z,
        }
    }

    // JAVA: net.minecraft.core.BlockPos#offset
    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
        Self {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

pub trait IntProvider: Debug {
    fn sample(&self, random: &mut dyn RandomSource) -> i32;
}

#[derive(Debug, Clone, Copy)]
pub struct ConstantInt(pub i32);

impl IntProvider for ConstantInt {
    fn sample(&self, _random: &mut dyn RandomSource) -> i32 {
        self.0
    }
}

pub trait LevelSimulatedReader {}
pub trait TreeConfiguration {}

pub trait FoliageSetter {
    fn set(&mut self, pos: BlockPos);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoliageAttachment {
    pub pos: BlockPos,
    pub radius_offset: i32,
    pub double_trunk: bool,
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
pub enum FoliagePlacerType {
    AcaciaFoliagePlacer,
}

#[derive(Debug)]
pub struct AcaciaFoliagePlacer {
    pub radius: Box<dyn IntProvider>,
    pub offset: Box<dyn IntProvider>,
}

impl AcaciaFoliagePlacer {
    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.AcaciaFoliagePlacer#<init>
    pub fn new(radius: Box<dyn IntProvider>, offset: Box<dyn IntProvider>) -> Self {
        Self { radius, offset }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.AcaciaFoliagePlacer#type
    pub fn foliage_placer_type(&self) -> FoliagePlacerType {
        FoliagePlacerType::AcaciaFoliagePlacer
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacer#foliageRadius
    pub fn foliage_radius(&self, random: &mut dyn RandomSource, _base_height: i32) -> i32 {
        self.radius
            .sample(random)
            .wrapping_add(self.offset.sample(random))
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacer#placeLeavesRow
    pub fn place_leaves_row(
        &self,
        level: &dyn LevelSimulatedReader,
        setter: &mut dyn FoliageSetter,
        random: &mut dyn RandomSource,
        config: &dyn TreeConfiguration,
        pos: BlockPos,
        range: i32,
        local_y: i32,
        large: bool,
    ) {
        let i: i32 = if large { 1 } else { 0 };
        let mut j: i32 = 0i32.wrapping_sub(range);
        while j <= range.wrapping_add(i) {
            let mut k: i32 = 0i32.wrapping_sub(range);
            while k <= range.wrapping_add(i) {
                if !self.should_skip_location_signed(random, j, local_y, k, range, large) {
                    let leaf_pos = pos.offset(j, local_y, k);
                    Self::try_place_leaf(level, setter, random, config, leaf_pos);
                }
                k = k.wrapping_add(1);
            }
            j = j.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacer#shouldSkipLocationSigned
    pub fn should_skip_location_signed(
        &self,
        random: &mut dyn RandomSource,
        local_x: i32,
        local_y: i32,
        local_z: i32,
        range: i32,
        large: bool,
    ) -> bool {
        let (dx, dz) = if large {
            let ax = local_x.wrapping_abs();
            let ax1 = local_x.wrapping_sub(1).wrapping_abs();
            let az = local_z.wrapping_abs();
            let az1 = local_z.wrapping_sub(1).wrapping_abs();
            (ax.min(ax1), az.min(az1))
        } else {
            (local_x.wrapping_abs(), local_z.wrapping_abs())
        };
        self.should_skip_location(random, dx, local_y, dz, range, large)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacer#tryPlaceLeaf
    pub fn try_place_leaf(
        _level: &dyn LevelSimulatedReader,
        setter: &mut dyn FoliageSetter,
        _random: &mut dyn RandomSource,
        _config: &dyn TreeConfiguration,
        pos: BlockPos,
    ) -> bool {
        setter.set(pos);
        true
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.AcaciaFoliagePlacer#createFoliage
    pub fn create_foliage(
        &self,
        level: &dyn LevelSimulatedReader,
        setter: &mut dyn FoliageSetter,
        random: &mut dyn RandomSource,
        config: &dyn TreeConfiguration,
        _max_free_tree_height: i32,
        attachment: &FoliageAttachment,
        foliage_height: i32,
        foliage_radius: i32,
        offset: i32,
    ) {
        let double_trunk = attachment.double_trunk();
        let top_pos = attachment.pos().above(offset);
        self.place_leaves_row(
            level,
            setter,
            random,
            config,
            top_pos,
            foliage_radius.wrapping_add(attachment.radius_offset()),
            (-1i32).wrapping_sub(foliage_height),
            double_trunk,
        );
        self.place_leaves_row(
            level,
            setter,
            random,
            config,
            top_pos,
            foliage_radius.wrapping_sub(1),
            0i32.wrapping_sub(foliage_height),
            double_trunk,
        );
        self.place_leaves_row(
            level,
            setter,
            random,
            config,
            top_pos,
            foliage_radius
                .wrapping_add(attachment.radius_offset())
                .wrapping_sub(1),
            0,
            double_trunk,
        );
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.AcaciaFoliagePlacer#foliageHeight
    pub fn foliage_height(
        &self,
        _random: &mut dyn RandomSource,
        _height: i32,
        _config: &dyn TreeConfiguration,
    ) -> i32 {
        0
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.AcaciaFoliagePlacer#shouldSkipLocation
    pub fn should_skip_location(
        &self,
        _random: &mut dyn RandomSource,
        local_x: i32,
        local_y: i32,
        local_z: i32,
        range: i32,
        _large: bool,
    ) -> bool {
        if local_y == 0 {
            (local_x > 1 || local_z > 1) && local_x != 0 && local_z != 0
        } else {
            local_x == range && local_z == range && range > 0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    struct SeqRandom {
        values: Vec<i32>,
        idx: usize,
    }

    impl SeqRandom {
        fn new(values: Vec<i32>) -> Self {
            Self { values, idx: 0 }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            let v = if self.idx < self.values.len() {
                self.values[self.idx]
            } else {
                0
            };
            self.idx += 1;
            ((v % bound) + bound) % bound
        }
    }

    #[derive(Debug)]
    struct FixedInt(i32);
    impl IntProvider for FixedInt {
        fn sample(&self, _random: &mut dyn RandomSource) -> i32 {
            self.0
        }
    }

    struct NullLevel;
    impl LevelSimulatedReader for NullLevel {}
    struct NullConfig;
    impl TreeConfiguration for NullConfig {}

    #[derive(Default)]
    struct CollectSetter {
        positions: Vec<BlockPos>,
    }
    impl FoliageSetter for CollectSetter {
        fn set(&mut self, pos: BlockPos) {
            self.positions.push(pos);
        }
    }

    #[test]
    fn should_skip_location_y_zero() {
        let placer = AcaciaFoliagePlacer::new(Box::new(FixedInt(0)), Box::new(FixedInt(0)));
        let mut rng = SeqRandom::new(vec![]);
        assert!(placer.should_skip_location(&mut rng, 2, 0, 2, 3, false));
        assert!(!placer.should_skip_location(&mut rng, 1, 0, 1, 3, false));
        assert!(!placer.should_skip_location(&mut rng, 0, 0, 2, 3, false));
        assert!(!placer.should_skip_location(&mut rng, 2, 0, 0, 3, false));
    }

    #[test]
    fn should_skip_location_y_nonzero() {
        let placer = AcaciaFoliagePlacer::new(Box::new(FixedInt(0)), Box::new(FixedInt(0)));
        let mut rng = SeqRandom::new(vec![]);
        assert!(placer.should_skip_location(&mut rng, 2, 1, 2, 2, false));
        assert!(!placer.should_skip_location(&mut rng, 1, 1, 2, 2, false));
        assert!(!placer.should_skip_location(&mut rng, 0, -1, 0, 0, false));
    }

    #[test]
    fn foliage_height_is_zero() {
        let placer = AcaciaFoliagePlacer::new(Box::new(FixedInt(0)), Box::new(FixedInt(0)));
        let mut rng = SeqRandom::new(vec![]);
        let config = NullConfig;
        assert_eq!(placer.foliage_height(&mut rng, 5, &config), 0);
        assert_eq!(placer.foliage_height(&mut rng, -10, &config), 0);
    }

    #[test]
    fn create_foliage_places_three_rows() {
        let placer = AcaciaFoliagePlacer::new(Box::new(FixedInt(2)), Box::new(FixedInt(0)));
        let mut rng = SeqRandom::new(vec![]);
        let level = NullLevel;
        let config = NullConfig;
        let mut setter = CollectSetter::default();
        let attachment = FoliageAttachment::new(BlockPos::new(0, 5, 0), 1, false);
        placer.create_foliage(&level, &mut setter, &mut rng, &config, 0, &attachment, 3, 2, 2);
        let ys: HashSet<i32> = setter.positions.iter().map(|p| p.y).collect();
        assert!(ys.contains(&3), "expected row at y=3, got {ys:?}");
        assert!(ys.contains(&4), "expected row at y=4, got {ys:?}");
        assert!(ys.contains(&7), "expected row at y=7, got {ys:?}");
    }

    #[test]
    fn create_foliage_negative_values_no_panic() {
        let placer = AcaciaFoliagePlacer::new(Box::new(FixedInt(0)), Box::new(FixedInt(0)));
        let mut rng = SeqRandom::new(vec![]);
        let level = NullLevel;
        let config = NullConfig;
        let mut setter = CollectSetter::default();
        let attachment = FoliageAttachment::new(BlockPos::new(0, 0, 0), -1, false);
        placer.create_foliage(&level, &mut setter, &mut rng, &config, 0, &attachment, -2, -1, 0);
        let _ = setter.positions.len();
    }

    #[test]
    fn foliage_radius_sums_providers() {
        let placer = AcaciaFoliagePlacer::new(Box::new(FixedInt(i32::MAX)), Box::new(FixedInt(1)));
        let mut rng = SeqRandom::new(vec![]);
        assert_eq!(placer.foliage_radius(&mut rng, 0), i32::MIN);
    }
}
