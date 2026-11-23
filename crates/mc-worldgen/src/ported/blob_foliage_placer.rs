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
    pub const BLOB_FOLIAGE_PLACER: Self = Self {
        id: "blob_foliage_placer",
    };
}

#[derive(Debug, Clone)]
pub struct BlobFoliagePlacer {
    pub radius: IntProvider,
    pub offset: IntProvider,
    pub height: i32,
}

impl BlobFoliagePlacer {
    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.BlobFoliagePlacer#<init>
    pub fn new(radius: IntProvider, offset: IntProvider, height: i32) -> Self {
        Self {
            radius,
            offset,
            height,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.BlobFoliagePlacer#type
    pub fn placer_type(&self) -> FoliagePlacerType {
        FoliagePlacerType::BLOB_FOLIAGE_PLACER
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.BlobFoliagePlacer#createFoliage
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
        let bound = offset_y.wrapping_sub(foliage_height);
        let mut y = offset_y;
        while y >= bound {
            let radius = foliage_radius
                .wrapping_add(attachment.radius_offset())
                .wrapping_sub(1)
                .wrapping_sub(y / 2)
                .max(0);
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
            if y == bound {
                break;
            }
            y = y.wrapping_sub(1);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.BlobFoliagePlacer#foliageHeight
    pub fn foliage_height(
        &self,
        _random: &mut impl RandomSource,
        _height: i32,
        _config: &TreeConfiguration,
    ) -> i32 {
        self.height
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.BlobFoliagePlacer#shouldSkipLocation
    pub fn should_skip_location(
        &self,
        random: &mut impl RandomSource,
        local_x: i32,
        local_y: i32,
        local_z: i32,
        range: i32,
        _large: bool,
    ) -> bool {
        local_x == range && local_z == range && (random.next_int(2) == 0 || local_y == 0)
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

#[cfg(test)]
mod tests {
    use super::*;
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
    fn foliage_height_returns_configured() {
        let p = BlobFoliagePlacer::new(IntProvider::constant(2), IntProvider::constant(0), 3);
        let mut r = SeqRandom::new(Vec::new());
        assert_eq!(p.foliage_height(&mut r, 10, &TreeConfiguration), 3);
    }

    #[test]
    fn should_skip_location_corner_logic() {
        let p = BlobFoliagePlacer::new(IntProvider::constant(2), IntProvider::constant(0), 3);
        let mut r = SeqRandom::new(vec![0]);
        assert!(p.should_skip_location(&mut r, 2, 1, 2, 2, false));
        let mut r = SeqRandom::new(vec![1]);
        assert!(!p.should_skip_location(&mut r, 2, 1, 2, 2, false));
        let mut r = SeqRandom::new(vec![1]);
        assert!(p.should_skip_location(&mut r, 2, 0, 2, 2, false));
        let mut r = SeqRandom::new(vec![0]);
        assert!(!p.should_skip_location(&mut r, 1, 1, 2, 2, false));
        assert_eq!(r.idx, 0);
    }

    #[test]
    fn create_foliage_blob_shape() {
        let p = BlobFoliagePlacer::new(IntProvider::constant(0), IntProvider::constant(0), 3);
        let mut setter = TestSetter::default();
        let mut random = SeqRandom::new(vec![1; 512]);
        let attachment = FoliageAttachment::new(BlockPos::new(0, 10, 0), 0, false);
        p.create_foliage(
            &LevelSimulatedReader,
            &mut setter,
            &mut random,
            &TreeConfiguration,
            10,
            &attachment,
            3,
            2,
            0,
        );
        // y=0,r=1:8 (corner at local_y==0 skipped), y=-1,r=1:9, y=-2,r=2:25, y=-3,r=2:25
        assert_eq!(setter.positions.len(), 67);
    }

    #[test]
    fn create_foliage_negative_height_no_rows() {
        let p = BlobFoliagePlacer::new(IntProvider::constant(0), IntProvider::constant(0), -1);
        let mut setter = TestSetter::default();
        let mut random = SeqRandom::new(Vec::new());
        let attachment = FoliageAttachment::new(BlockPos::new(0, 5, 0), 0, false);
        p.create_foliage(
            &LevelSimulatedReader,
            &mut setter,
            &mut random,
            &TreeConfiguration,
            10,
            &attachment,
            -1,
            2,
            0,
        );
        assert!(setter.positions.is_empty());
    }

    #[test]
    fn radius_wrapping_and_placer_type() {
        let p = BlobFoliagePlacer::new(IntProvider::constant(0), IntProvider::constant(0), 0);
        assert_eq!(p.placer_type(), FoliagePlacerType::BLOB_FOLIAGE_PLACER);
        let mut setter = TestSetter::default();
        let mut random = SeqRandom::new(vec![1; 16]);
        let attachment = FoliageAttachment::new(BlockPos::new(0, 0, 0), 2, false);
        p.create_foliage(
            &LevelSimulatedReader,
            &mut setter,
            &mut random,
            &TreeConfiguration,
            0,
            &attachment,
            0,
            i32::MAX,
            0,
        );
        // radius wraps to i32::MIN, max(...,0)=0; range 0 at local_y==0 is skipped
        assert!(setter.positions.is_empty());
        assert_eq!(random.idx, 1);
    }
}
