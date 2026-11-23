#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

pub trait RandomSource {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeightmapType {
    WorldSurfaceWg,
    WorldSurface,
    OceanFloorWg,
    OceanFloor,
    MotionBlocking,
    MotionBlockingNoLeaves,
}

pub trait PlacementContext {
    fn get_height(&self, heightmap: HeightmapType, x: i32, z: i32) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceRelativeThresholdFilter {
    pub heightmap: HeightmapType,
    pub min_inclusive: i32,
    pub max_inclusive: i32,
}

impl SurfaceRelativeThresholdFilter {
    // JAVA: net.minecraft.world.level.levelgen.placement.SurfaceRelativeThresholdFilter#<init>
    fn new(heightmap: HeightmapType, min_inclusive: i32, max_inclusive: i32) -> Self {
        Self {
            heightmap,
            min_inclusive,
            max_inclusive,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.SurfaceRelativeThresholdFilter#of
    pub fn of(heightmap: HeightmapType, min_inclusive: i32, max_inclusive: i32) -> Self {
        Self::new(heightmap, min_inclusive, max_inclusive)
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.SurfaceRelativeThresholdFilter#shouldPlace
    pub fn should_place(
        &self,
        ctx: &impl PlacementContext,
        _random: &mut impl RandomSource,
        pos: &BlockPos,
    ) -> bool {
        let height = ctx.get_height(self.heightmap, pos.get_x(), pos.get_z()) as i64;
        let min_y = height + (self.min_inclusive as i64);
        let max_y = height + (self.max_inclusive as i64);
        let y = pos.get_y() as i64;
        min_y <= y && y <= max_y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyRandom;
    impl RandomSource for DummyRandom {}

    struct FixedHeightContext {
        height: i32,
    }

    impl PlacementContext for FixedHeightContext {
        fn get_height(&self, _heightmap: HeightmapType, _x: i32, _z: i32) -> i32 {
            self.height
        }
    }

    struct RecordingContext {
        height: i32,
        last_type: std::cell::Cell<Option<HeightmapType>>,
        last_x: std::cell::Cell<i32>,
        last_z: std::cell::Cell<i32>,
    }

    impl RecordingContext {
        fn new(height: i32) -> Self {
            Self {
                height,
                last_type: std::cell::Cell::new(None),
                last_x: std::cell::Cell::new(0),
                last_z: std::cell::Cell::new(0),
            }
        }
    }

    impl PlacementContext for RecordingContext {
        fn get_height(&self, heightmap: HeightmapType, x: i32, z: i32) -> i32 {
            self.last_type.set(Some(heightmap));
            self.last_x.set(x);
            self.last_z.set(z);
            self.height
        }
    }

    #[test]
    fn should_place_inclusive_bounds() {
        let filter = SurfaceRelativeThresholdFilter::of(HeightmapType::WorldSurfaceWg, -2, 3);
        let ctx = FixedHeightContext { height: 100 };
        let mut rng = DummyRandom;
        assert!(filter.should_place(&ctx, &mut rng, &BlockPos::new(0, 98, 0)));
        assert!(filter.should_place(&ctx, &mut rng, &BlockPos::new(0, 103, 0)));
        assert!(!filter.should_place(&ctx, &mut rng, &BlockPos::new(0, 97, 0)));
        assert!(!filter.should_place(&ctx, &mut rng, &BlockPos::new(0, 104, 0)));
    }

    #[test]
    fn should_place_queries_heightmap_with_pos_xz() {
        let filter = SurfaceRelativeThresholdFilter::of(HeightmapType::OceanFloor, 0, 0);
        let ctx = RecordingContext::new(64);
        let mut rng = DummyRandom;
        assert!(filter.should_place(&ctx, &mut rng, &BlockPos::new(12, 64, -7)));
        assert_eq!(ctx.last_type.get(), Some(HeightmapType::OceanFloor));
        assert_eq!(ctx.last_x.get(), 12);
        assert_eq!(ctx.last_z.get(), -7);
    }

    #[test]
    fn should_place_handles_negative_height_and_offsets() {
        let filter = SurfaceRelativeThresholdFilter::of(HeightmapType::MotionBlocking, -5, -1);
        let ctx = FixedHeightContext { height: -10 };
        let mut rng = DummyRandom;
        assert!(filter.should_place(&ctx, &mut rng, &BlockPos::new(0, -15, 0)));
        assert!(filter.should_place(&ctx, &mut rng, &BlockPos::new(0, -11, 0)));
        assert!(!filter.should_place(&ctx, &mut rng, &BlockPos::new(0, -16, 0)));
        assert!(!filter.should_place(&ctx, &mut rng, &BlockPos::new(0, -10, 0)));
    }

    #[test]
    fn should_place_uses_i64_for_extreme_bounds() {
        let filter = SurfaceRelativeThresholdFilter::of(HeightmapType::WorldSurface, i32::MIN, i32::MAX);
        let ctx = FixedHeightContext { height: 0 };
        let mut rng = DummyRandom;
        assert!(filter.should_place(&ctx, &mut rng, &BlockPos::new(0, i32::MIN, 0)));
        assert!(filter.should_place(&ctx, &mut rng, &BlockPos::new(0, i32::MAX, 0)));
    }
}
