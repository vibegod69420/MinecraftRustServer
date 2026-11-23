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
    FancyFoliagePlacer,
}

#[derive(Debug)]
pub struct FancyFoliagePlacer {
    pub radius: Box<dyn IntProvider>,
    pub offset: Box<dyn IntProvider>,
    pub height: i32,
}

impl FancyFoliagePlacer {
    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FancyFoliagePlacer#<init>
    pub fn new(radius: Box<dyn IntProvider>, offset: Box<dyn IntProvider>, height: i32) -> Self {
        Self {
            radius,
            offset,
            height,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FancyFoliagePlacer#type
    pub fn foliage_placer_type(&self) -> FoliagePlacerType {
        FoliagePlacerType::FancyFoliagePlacer
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.BlobFoliagePlacer#foliageHeight
    pub fn foliage_height(
        &self,
        _random: &mut dyn RandomSource,
        _height: i32,
        _config: &dyn TreeConfiguration,
    ) -> i32 {
        self.height
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
        // Simplified for self-contained port: real implementation checks validity and block state.
        setter.set(pos);
        true
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FancyFoliagePlacer#createFoliage
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
        let mut yy: i32 = offset;
        let min_y: i32 = offset.wrapping_sub(foliage_height);
        while yy >= min_y {
            let extra: i32 = if yy != offset && yy != min_y { 1 } else { 0 };
            let leaf_radius: i32 = foliage_radius.wrapping_add(extra);
            self.place_leaves_row(
                level,
                setter,
                random,
                config,
                attachment.pos(),
                leaf_radius,
                yy,
                attachment.double_trunk(),
            );
            yy = yy.wrapping_sub(1);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FancyFoliagePlacer#shouldSkipLocation
    pub fn should_skip_location(
        &self,
        _random: &mut dyn RandomSource,
        local_x: i32,
        _local_y: i32,
        local_z: i32,
        range: i32,
        _large: bool,
    ) -> bool {
        let dx: f32 = (local_x as f32) + 0.5_f32;
        let dz: f32 = (local_z as f32) + 0.5_f32;
        let dist_sq: f32 = (dx * dx) + (dz * dz);
        let range_sq: f32 = range.wrapping_mul(range) as f32;
        dist_sq > range_sq
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn should_skip_location_circle_check() {
        let placer = FancyFoliagePlacer::new(Box::new(FixedInt(0)), Box::new(FixedInt(0)), 4);
        let mut rng = SeqRandom::new(vec![]);
        assert!(!placer.should_skip_location(&mut rng, 1, 0, 0, 2, false));
        assert!(placer.should_skip_location(&mut rng, 2, 0, 0, 2, false));
    }

    #[test]
    fn should_skip_location_float_bits() {
        let placer = FancyFoliagePlacer::new(Box::new(FixedInt(0)), Box::new(FixedInt(0)), 4);
        let mut rng = SeqRandom::new(vec![]);
        let dx = 1.5_f32;
        let dist_sq = (dx * dx) + (dx * dx);
        assert_eq!(dist_sq.to_bits(), 4.5_f32.to_bits());
        assert!(placer.should_skip_location(&mut rng, 1, 0, 1, 2, false));
    }

    #[test]
    fn should_skip_location_range_overflow() {
        let placer = FancyFoliagePlacer::new(Box::new(FixedInt(0)), Box::new(FixedInt(0)), 4);
        let mut rng = SeqRandom::new(vec![]);
        let range: i32 = 46341;
        let range_sq_f32 = range.wrapping_mul(range) as f32;
        let skip = placer.should_skip_location(&mut rng, 0, 0, 0, range, false);
        assert_eq!(skip, 0.5_f32 > range_sq_f32);
    }

    #[test]
    fn should_skip_location_signed_large_and_negative() {
        let placer = FancyFoliagePlacer::new(Box::new(FixedInt(0)), Box::new(FixedInt(0)), 4);
        let mut rng = SeqRandom::new(vec![]);
        assert!(!placer.should_skip_location_signed(&mut rng, 1, 0, 1, 1, true));
        assert!(placer.should_skip_location_signed(&mut rng, 2, 0, 0, 1, true));
        assert!(placer.should_skip_location_signed(&mut rng, -2, 0, 0, 1, false));
        assert!(!placer.should_skip_location_signed(&mut rng, -1, 0, 0, 2, false));
    }

    #[test]
    fn create_foliage_radius_pattern() {
        let placer = FancyFoliagePlacer::new(Box::new(FixedInt(0)), Box::new(FixedInt(0)), 4);
        let mut rng = SeqRandom::new(vec![]);
        let level = NullLevel;
        let config = NullConfig;
        let mut setter = CollectSetter::default();
        let attachment = FoliageAttachment::new(BlockPos::new(0, 10, 0), 0, false);
        placer.create_foliage(&level, &mut setter, &mut rng, &config, 0, &attachment, 2, 2, 3);
        let count_y = |y: i32| setter.positions.iter().filter(|p| p.y == 10 + y).count();
        let top = count_y(3);
        let mid = count_y(2);
        let bot = count_y(1);
        assert!(mid > top, "middle row should be wider: top={top}, mid={mid}, bot={bot}");
        assert_eq!(top, bot, "top and bottom should have same radius");
    }

    #[test]
    fn foliage_height_and_radius() {
        let placer = FancyFoliagePlacer::new(Box::new(FixedInt(2)), Box::new(FixedInt(1)), 7);
        let mut rng = SeqRandom::new(vec![]);
        let config = NullConfig;
        assert_eq!(placer.foliage_height(&mut rng, 0, &config), 7);
        assert_eq!(placer.foliage_radius(&mut rng, 0), 3);
    }
}
