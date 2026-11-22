//! Port of `net.minecraft.world.level.levelgen.feature.foliageplacers.BushFoliagePlacer` (1.21.11).

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

    pub fn above(&self, dy: i32) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(dy),
            z: self.z,
        }
    }
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IntProvider {
    pub value: i32,
}

impl IntProvider {
    pub fn constant(value: i32) -> Self {
        Self { value }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TreeConfiguration;

pub trait LevelSimulatedReader {}

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
pub struct LeavesRow {
    pub pos: BlockPos,
    pub radius: i32,
    pub y: i32,
    pub double_trunk: bool,
}

#[derive(Debug, Default, Clone)]
pub struct FoliageSetter {
    pub rows: Vec<LeavesRow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoliagePlacerType {
    BushFoliagePlacer,
    FancyFoliagePlacer,
    AcaciaFoliagePlacer,
    MegaJungleFoliagePlacer,
}

#[derive(Debug, Clone)]
pub struct BushFoliagePlacer {
    pub radius: IntProvider,
    pub offset: IntProvider,
    pub height: i32,
}

impl BushFoliagePlacer {
    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.BushFoliagePlacer#<init>
    pub fn new(radius: IntProvider, offset: IntProvider, height: i32) -> Self {
        Self {
            radius,
            offset,
            height,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.BushFoliagePlacer#type
    pub fn foliage_placer_type(&self) -> FoliagePlacerType {
        FoliagePlacerType::BushFoliagePlacer
    }

    pub fn place_leaves_row(
        &self,
        _level: &dyn LevelSimulatedReader,
        setter: &mut FoliageSetter,
        _random: &mut dyn RandomSource,
        _config: &TreeConfiguration,
        pos: BlockPos,
        radius: i32,
        y: i32,
        double_trunk: bool,
    ) {
        setter.rows.push(LeavesRow {
            pos,
            radius,
            y,
            double_trunk,
        });
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.BushFoliagePlacer#createFoliage
    pub fn create_foliage(
        &self,
        level: &dyn LevelSimulatedReader,
        setter: &mut FoliageSetter,
        random: &mut dyn RandomSource,
        config: &TreeConfiguration,
        _max_free_tree_height: i32,
        attachment: &FoliageAttachment,
        foliage_height: i32,
        foliage_radius: i32,
        offset: i32,
    ) {
        let mut y = offset;
        let min_y = offset.wrapping_sub(foliage_height);
        while y >= min_y {
            let radius = foliage_radius
                .wrapping_add(attachment.radius_offset())
                .wrapping_sub(1)
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

    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.BushFoliagePlacer#shouldSkipLocation
    pub fn should_skip_location(
        &self,
        random: &mut dyn RandomSource,
        local_x: i32,
        _local_y: i32,
        local_z: i32,
        range: i32,
        _large: bool,
    ) -> bool {
        local_x == range && local_z == range && random.next_int(2) == 0
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
        fn next_int(&mut self, _bound: i32) -> i32 {
            let v = self.values[self.idx % self.values.len()];
            self.idx += 1;
            v
        }
    }

    struct NoopLevel;
    impl LevelSimulatedReader for NoopLevel {}

    #[test]
    fn type_is_bush() {
        let placer = BushFoliagePlacer::new(IntProvider::default(), IntProvider::default(), 0);
        assert_eq!(
            placer.foliage_placer_type(),
            FoliagePlacerType::BushFoliagePlacer
        );
    }

    #[test]
    fn should_skip_corner_only_when_random_zero() {
        let placer = BushFoliagePlacer::new(IntProvider::default(), IntProvider::default(), 0);
        let mut rng0 = SeqRandom::new(vec![0]);
        let mut rng1 = SeqRandom::new(vec![1]);
        assert!(placer.should_skip_location(&mut rng0, 2, 0, 2, 2, false));
        assert!(!placer.should_skip_location(&mut rng1, 2, 0, 2, 2, false));
    }

    #[test]
    fn should_not_skip_non_corner() {
        let placer = BushFoliagePlacer::new(IntProvider::default(), IntProvider::default(), 0);
        let mut rng = SeqRandom::new(vec![0]);
        assert!(!placer.should_skip_location(&mut rng, 1, 0, 2, 2, false));
        assert!(!placer.should_skip_location(&mut rng, 2, 0, 1, 2, false));
        assert!(!placer.should_skip_location(&mut rng, -2, 0, 2, 2, false));
    }

    #[test]
    fn create_foliage_rows_match_java_formula() {
        let placer = BushFoliagePlacer::new(IntProvider::default(), IntProvider::default(), 2);
        let mut setter = FoliageSetter::default();
        let mut rng = SeqRandom::new(vec![0]);
        let config = TreeConfiguration;
        let level = NoopLevel;
        let attachment = FoliageAttachment::new(BlockPos::new(10, 20, 30), 0, false);
        placer.create_foliage(&level, &mut setter, &mut rng, &config, 5, &attachment, 1, 3, 2);
        assert_eq!(setter.rows.len(), 2);
        assert_eq!(
            setter.rows[0],
            LeavesRow {
                pos: BlockPos::new(10, 20, 30),
                radius: 0, // 3 + 0 - 1 - 2
                y: 2,
                double_trunk: false,
            }
        );
        assert_eq!(
            setter.rows[1],
            LeavesRow {
                pos: BlockPos::new(10, 20, 30),
                radius: 1, // 3 + 0 - 1 - 1
                y: 1,
                double_trunk: false,
            }
        );
    }

    #[test]
    fn create_foliage_with_negative_radius_offset() {
        let placer = BushFoliagePlacer::new(IntProvider::default(), IntProvider::default(), 0);
        let mut setter = FoliageSetter::default();
        let mut rng = SeqRandom::new(vec![0]);
        let attachment = FoliageAttachment::new(BlockPos::new(0, 0, 0), -1, true);
        placer.create_foliage(
            &NoopLevel,
            &mut setter,
            &mut rng,
            &TreeConfiguration,
            0,
            &attachment,
            0,
            2,
            0,
        );
        assert_eq!(setter.rows.len(), 1);
        assert_eq!(setter.rows[0].radius, 0); // 2 + (-1) - 1 - 0
        assert!(setter.rows[0].double_trunk);
    }
}
