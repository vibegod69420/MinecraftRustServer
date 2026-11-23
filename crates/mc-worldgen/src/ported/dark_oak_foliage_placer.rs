//! Port of `net.minecraft.world.level.levelgen.feature.foliageplacers.DarkOakFoliagePlacer` (1.21.11).

use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Equivalent to Java `BlockPos.above(int)`.
    pub const fn above(self, dy: i32) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(dy),
            z: self.z,
        }
    }

    /// Equivalent to Java `BlockPos.offset(int, int, int)` / mutable `setWithOffset`.
    pub const fn offset(self, dx: i32, dy: i32, dz: i32) -> Self {
        Self {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoliageAttachment {
    pos: BlockPos,
    radius_offset: i32,
    double_trunk: bool,
}

impl FoliageAttachment {
    pub const fn new(pos: BlockPos, radius_offset: i32, double_trunk: bool) -> Self {
        Self {
            pos,
            radius_offset,
            double_trunk,
        }
    }

    pub const fn pos(&self) -> BlockPos {
        self.pos
    }

    pub const fn radius_offset(&self) -> i32 {
        self.radius_offset
    }

    pub const fn double_trunk(&self) -> bool {
        self.double_trunk
    }
}

pub trait RandomSource {
    fn next_boolean(&mut self) -> bool;
}

pub trait IntProvider: Debug {
    fn sample(&self, random: &mut dyn RandomSource) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstantInt(pub i32);

impl IntProvider for ConstantInt {
    fn sample(&self, _random: &mut dyn RandomSource) -> i32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TreeConfiguration;

pub trait LevelSimulatedReader {
    fn is_valid_tree_pos(&self, pos: BlockPos) -> bool;
}

pub trait FoliageSetter {
    fn set_leaf(&mut self, pos: BlockPos);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoliagePlacerType {
    DarkOakFoliagePlacer,
}

// JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacer#tryPlaceLeaf
fn try_place_leaf(
    level: &dyn LevelSimulatedReader,
    setter: &mut dyn FoliageSetter,
    _random: &mut dyn RandomSource,
    _config: &TreeConfiguration,
    pos: BlockPos,
) -> bool {
    if !level.is_valid_tree_pos(pos) {
        return false;
    }
    setter.set_leaf(pos);
    true
}

#[derive(Debug)]
pub struct DarkOakFoliagePlacer {
    pub radius: Box<dyn IntProvider>,
    pub offset: Box<dyn IntProvider>,
}

impl DarkOakFoliagePlacer {
    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.DarkOakFoliagePlacer#<init>
    pub fn new(radius: Box<dyn IntProvider>, offset: Box<dyn IntProvider>) -> Self {
        Self { radius, offset }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.DarkOakFoliagePlacer#type
    pub fn foliage_placer_type(&self) -> FoliagePlacerType {
        FoliagePlacerType::DarkOakFoliagePlacer
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacer#shouldSkipLocationSigned
    fn base_should_skip_location_signed(
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
        level: &dyn LevelSimulatedReader,
        setter: &mut dyn FoliageSetter,
        random: &mut dyn RandomSource,
        config: &TreeConfiguration,
        pos: BlockPos,
        range: i32,
        y_offset: i32,
        double_trunk: bool,
    ) {
        let i: i32 = if double_trunk { 1 } else { 0 };
        let end = range.wrapping_add(i);
        let mut j = 0i32.wrapping_sub(range);
        while j <= end {
            let mut k = 0i32.wrapping_sub(range);
            while k <= end {
                if !self.should_skip_location_signed(random, j, y_offset, k, range, double_trunk) {
                    let leaf_pos = pos.offset(j, y_offset, k);
                    try_place_leaf(level, setter, random, config, leaf_pos);
                }
                k = k.wrapping_add(1);
            }
            j = j.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.DarkOakFoliagePlacer#createFoliage
    pub fn create_foliage(
        &self,
        level: &dyn LevelSimulatedReader,
        setter: &mut dyn FoliageSetter,
        random: &mut dyn RandomSource,
        config: &TreeConfiguration,
        _max_free_tree_height: i32,
        attachment: &FoliageAttachment,
        _foliage_height: i32,
        foliage_radius: i32,
        offset: i32,
    ) {
        let pos = attachment.pos().above(offset);
        let double_trunk = attachment.double_trunk();
        if double_trunk {
            self.place_leaves_row(
                level,
                setter,
                random,
                config,
                pos,
                foliage_radius.wrapping_add(2),
                -1,
                double_trunk,
            );
            self.place_leaves_row(
                level,
                setter,
                random,
                config,
                pos,
                foliage_radius.wrapping_add(3),
                0,
                double_trunk,
            );
            self.place_leaves_row(
                level,
                setter,
                random,
                config,
                pos,
                foliage_radius.wrapping_add(2),
                1,
                double_trunk,
            );
            if random.next_boolean() {
                self.place_leaves_row(
                    level,
                    setter,
                    random,
                    config,
                    pos,
                    foliage_radius,
                    2,
                    double_trunk,
                );
            }
        } else {
            self.place_leaves_row(
                level,
                setter,
                random,
                config,
                pos,
                foliage_radius.wrapping_add(2),
                -1,
                double_trunk,
            );
            self.place_leaves_row(
                level,
                setter,
                random,
                config,
                pos,
                foliage_radius.wrapping_add(1),
                0,
                double_trunk,
            );
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.DarkOakFoliagePlacer#foliageHeight
    pub fn foliage_height(
        &self,
        _random: &mut dyn RandomSource,
        _height: i32,
        _config: &TreeConfiguration,
    ) -> i32 {
        4
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.DarkOakFoliagePlacer#shouldSkipLocationSigned
    pub fn should_skip_location_signed(
        &self,
        random: &mut dyn RandomSource,
        local_x: i32,
        local_y: i32,
        local_z: i32,
        range: i32,
        large: bool,
    ) -> bool {
        if local_y != 0
            || !large
            || (local_x != range.wrapping_neg() && local_x < range)
            || (local_z != range.wrapping_neg() && local_z < range)
        {
            self.base_should_skip_location_signed(
                random, local_x, local_y, local_z, range, large,
            )
        } else {
            true
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.DarkOakFoliagePlacer#shouldSkipLocation
    pub fn should_skip_location(
        &self,
        _random: &mut dyn RandomSource,
        local_x: i32,
        local_y: i32,
        local_z: i32,
        range: i32,
        large: bool,
    ) -> bool {
        if local_y == -1 && !large {
            return local_x == range && local_z == range;
        }
        if local_y == 1 {
            return local_x.wrapping_add(local_z)
                > range.wrapping_mul(2).wrapping_sub(2);
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SeqRandom {
        bools: Vec<bool>,
        idx: usize,
    }

    impl SeqRandom {
        fn new(bools: Vec<bool>) -> Self {
            Self { bools, idx: 0 }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_boolean(&mut self) -> bool {
            let v = self.bools.get(self.idx).copied().unwrap_or(false);
            self.idx += 1;
            v
        }
    }

    struct MockLevel;

    impl LevelSimulatedReader for MockLevel {
        fn is_valid_tree_pos(&self, _pos: BlockPos) -> bool {
            true
        }
    }

    #[derive(Default)]
    struct MockSetter {
        positions: Vec<BlockPos>,
    }

    impl FoliageSetter for MockSetter {
        fn set_leaf(&mut self, pos: BlockPos) {
            self.positions.push(pos);
        }
    }

    fn placer() -> DarkOakFoliagePlacer {
        DarkOakFoliagePlacer::new(Box::new(ConstantInt(0)), Box::new(ConstantInt(0)))
    }

    #[test]
    fn foliage_height_is_four() {
        let p = placer();
        let mut rng = SeqRandom::new(vec![]);
        let cfg = TreeConfiguration;
        assert_eq!(p.foliage_height(&mut rng, 0, &cfg), 4);
        assert_eq!(p.foliage_height(&mut rng, 10, &cfg), 4);
    }

    #[test]
    fn should_skip_location_rules() {
        let p = placer();
        let mut rng = SeqRandom::new(vec![]);
        // y == -1, not large: only far corner (range, range) is skipped
        assert!(p.should_skip_location(&mut rng, 2, -1, 2, 2, false));
        assert!(!p.should_skip_location(&mut rng, 1, -1, 2, 2, false));
        assert!(!p.should_skip_location(&mut rng, 2, -1, 2, 2, true));
        // y == 1: skip when local_x + local_z > range * 2 - 2
        assert!(p.should_skip_location(&mut rng, 2, 1, 2, 2, false)); // 4 > 2
        assert!(!p.should_skip_location(&mut rng, 1, 1, 1, 2, false)); // 2 > 2 is false
        // other y: false
        assert!(!p.should_skip_location(&mut rng, 5, 0, 5, 2, false));
    }

    #[test]
    fn should_skip_location_signed_dark_oak_corner_trim() {
        let p = placer();
        let mut rng = SeqRandom::new(vec![]);
        let range = 3;
        // y==0, large: extreme corners are skipped
        assert!(p.should_skip_location_signed(&mut rng, -3, 0, -3, range, true));
        assert!(p.should_skip_location_signed(&mut rng, 3, 0, 3, range, true));
        assert!(p.should_skip_location_signed(&mut rng, -3, 0, 3, range, true));
        // inner positions delegate to base -> should_skip_location (y==0 => false)
        assert!(!p.should_skip_location_signed(&mut rng, 0, 0, 0, range, true));
        // y!=0 delegates to base; y==1 with large computes dx=min(abs(-3),abs(-4))=3, dz=3, 6>4 => skip
        assert!(p.should_skip_location_signed(&mut rng, -3, 1, -3, range, true));
    }

    #[test]
    fn create_foliage_double_and_single_trunk() {
        let p = placer();
        let level = MockLevel;
        let cfg = TreeConfiguration;

        // Double trunk, next_boolean = true -> extra top row
        let mut rng = SeqRandom::new(vec![true]);
        let mut setter = MockSetter::default();
        let attachment = FoliageAttachment::new(BlockPos::new(0, 10, 0), 0, true);
        p.create_foliage(&level, &mut setter, &mut rng, &cfg, 5, &attachment, 4, 1, 0);
        assert!(!setter.positions.is_empty());
        for pos in &setter.positions {
            assert!(pos.y >= 9 && pos.y <= 12, "unexpected y {}", pos.y);
        }

        // Single trunk
        let mut rng2 = SeqRandom::new(vec![]);
        let mut setter2 = MockSetter::default();
        let attachment2 = FoliageAttachment::new(BlockPos::new(1, 5, -2), 0, false);
        p.create_foliage(&level, &mut setter2, &mut rng2, &cfg, 5, &attachment2, 4, 0, 1);
        assert!(!setter2.positions.is_empty());
        for pos in &setter2.positions {
            assert!(pos.y == 5 || pos.y == 6, "unexpected y {}", pos.y);
        }
    }

    #[test]
    fn negative_overflow_and_float_bits() {
        let p = placer();
        let mut rng = SeqRandom::new(vec![]);

        // Negative inputs should not panic
        let _ = p.should_skip_location(&mut rng, -1, 1, -2, -3, false);
        let _ = p.should_skip_location_signed(&mut rng, i32::MIN, 0, i32::MAX, 2, true);

        // Overflow edge cases in arithmetic
        let _ = p.should_skip_location(&mut rng, i32::MAX, 1, i32::MAX, i32::MAX, false);

        // BlockPos wrapping semantics
        let pos = BlockPos::new(0, i32::MAX, 0).above(1);
        assert_eq!(pos.y, i32::MIN);
        let pos2 = BlockPos::new(5, 0, -7).above(-1);
        assert_eq!(pos2, BlockPos::new(5, -1, -7));

        // Float bit patterns preserved
        let f_bits: u32 = 0xFFC0_0001;
        assert_eq!(f32::from_bits(f_bits).to_bits(), f_bits);
        let d_bits: u64 = 0xFFF8_0000_0000_0001;
        assert_eq!(f64::from_bits(d_bits).to_bits(), d_bits);
    }
}
