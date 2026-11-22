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
}

pub trait RandomSourceLike {
    fn next_int(&mut self, bound: i32) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightmapType {
    OceanFloor,
    WorldSurface,
}

pub trait PlacementContextLike {
    fn get_height(&self, heightmap: HeightmapType, x: i32, z: i32) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementModifierType {
    SurfaceWaterDepthFilter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceWaterDepthFilter {
    max_water_depth: i32,
}

impl SurfaceWaterDepthFilter {
    // JAVA: net.minecraft.world.level.levelgen.placement.SurfaceWaterDepthFilter#<init>
    fn new(max_water_depth: i32) -> Self {
        Self { max_water_depth }
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.SurfaceWaterDepthFilter#forMaxDepth
    pub fn for_max_depth(max_water_depth: i32) -> Self {
        Self::new(max_water_depth)
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.SurfaceWaterDepthFilter#shouldPlace
    pub fn should_place<C: PlacementContextLike, R: RandomSourceLike>(
        &self,
        ctx: &C,
        _random: &mut R,
        pos: BlockPos,
    ) -> bool {
        let ocean_floor = ctx.get_height(HeightmapType::OceanFloor, pos.x, pos.z);
        let world_surface = ctx.get_height(HeightmapType::WorldSurface, pos.x, pos.z);
        world_surface.wrapping_sub(ocean_floor) <= self.max_water_depth
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.SurfaceWaterDepthFilter#type
    pub fn get_type(&self) -> PlacementModifierType {
        PlacementModifierType::SurfaceWaterDepthFilter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCtx {
        ocean: i32,
        surface: i32,
    }

    impl PlacementContextLike for MockCtx {
        fn get_height(&self, h: HeightmapType, _x: i32, _z: i32) -> i32 {
            match h {
                HeightmapType::OceanFloor => self.ocean,
                HeightmapType::WorldSurface => self.surface,
            }
        }
    }

    struct Rng;
    impl RandomSourceLike for Rng {
        fn next_int(&mut self, _bound: i32) -> i32 {
            0
        }
    }

    #[test]
    fn allows_when_depth_within_max() {
        let f = SurfaceWaterDepthFilter::for_max_depth(2);
        let ctx = MockCtx { ocean: 60, surface: 62 };
        assert!(f.should_place(&ctx, &mut Rng, BlockPos::new(0, 0, 0)));
    }

    #[test]
    fn rejects_when_depth_exceeds_max() {
        let f = SurfaceWaterDepthFilter::for_max_depth(1);
        let ctx = MockCtx { ocean: 60, surface: 62 };
        assert!(!f.should_place(&ctx, &mut Rng, BlockPos::new(1, 0, 2)));
    }

    #[test]
    fn equal_depth_is_allowed() {
        let f = SurfaceWaterDepthFilter::for_max_depth(0);
        let ctx = MockCtx { ocean: 63, surface: 63 };
        assert!(f.should_place(&ctx, &mut Rng, BlockPos::new(-5, 0, 10)));
    }

    #[test]
    fn negative_coordinates_and_depths() {
        let f = SurfaceWaterDepthFilter::for_max_depth(-1);
        let ctx = MockCtx { ocean: 10, surface: 5 };
        // 5 - 10 = -5 <= -1
        assert!(f.should_place(&ctx, &mut Rng, BlockPos::new(-100, 64, -200)));
    }

    #[test]
    fn wrapping_sub_overflow_behavior() {
        let f = SurfaceWaterDepthFilter::for_max_depth(0);
        let ctx = MockCtx { ocean: 1, surface: i32::MIN };
        // i32::MIN.wrapping_sub(1) == i32::MAX
        assert!(!f.should_place(&ctx, &mut Rng, BlockPos::new(0, 0, 0)));
    }

    #[test]
    fn type_is_surface_water_depth_filter() {
        let f = SurfaceWaterDepthFilter::for_max_depth(0);
        assert_eq!(f.get_type(), PlacementModifierType::SurfaceWaterDepthFilter);
    }
}
