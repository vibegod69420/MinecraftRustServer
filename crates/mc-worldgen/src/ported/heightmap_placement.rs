//! Port of `net.minecraft.world.level.levelgen.placement.HeightmapPlacement`.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    x: i32,
    y: i32,
    z: i32,
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

#[derive(Debug, Default)]
pub struct RandomSourceStub;

impl RandomSource for RandomSourceStub {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeightmapTypes {
    WorldSurfaceWg,
    WorldSurface,
    OceanFloorWg,
    OceanFloor,
    MotionBlocking,
    MotionBlockingNoLeaves,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementModifierType {
    Heightmap,
}

pub struct PlacementContext {
    min_y: i32,
    height_fn: Box<dyn Fn(HeightmapTypes, i32, i32) -> i32>,
}

impl PlacementContext {
    pub fn new(
        min_y: i32,
        height_fn: impl Fn(HeightmapTypes, i32, i32) -> i32 + 'static,
    ) -> Self {
        Self {
            min_y,
            height_fn: Box::new(height_fn),
        }
    }

    pub fn get_height(&self, heightmap: HeightmapTypes, x: i32, z: i32) -> i32 {
        (self.height_fn)(heightmap, x, z)
    }

    pub fn get_min_y(&self) -> i32 {
        self.min_y
    }
}

/// Port of `net.minecraft.world.level.levelgen.placement.HeightmapPlacement`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeightmapPlacement {
    heightmap: HeightmapTypes,
}

impl HeightmapPlacement {
    // JAVA: net.minecraft.world.level.levelgen.placement.HeightmapPlacement#HeightmapPlacement
    fn new(heightmap: HeightmapTypes) -> Self {
        Self { heightmap }
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.HeightmapPlacement#onHeightmap
    pub fn on_heightmap(heightmap: HeightmapTypes) -> Self {
        Self::new(heightmap)
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.HeightmapPlacement#getPositions
    pub fn get_positions(
        &self,
        ctx: &PlacementContext,
        _random: &mut dyn RandomSource,
        pos: BlockPos,
    ) -> Vec<BlockPos> {
        let x = pos.get_x();
        let z = pos.get_z();
        let height = ctx.get_height(self.heightmap, x, z);
        if height > ctx.get_min_y() {
            vec![BlockPos::new(x, height, z)]
        } else {
            Vec::new()
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.HeightmapPlacement#type
    pub fn type_(&self) -> PlacementModifierType {
        PlacementModifierType::Heightmap
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_position_when_height_above_min_y() {
        let ctx = PlacementContext::new(-64, |_hm, _x, _z| 70);
        let mut rng = RandomSourceStub;
        let placement = HeightmapPlacement::on_heightmap(HeightmapTypes::WorldSurfaceWg);

        let result = placement.get_positions(&ctx, &mut rng, BlockPos::new(10, 0, 20));
        assert_eq!(result, vec![BlockPos::new(10, 70, 20)]);
    }

    #[test]
    fn returns_empty_when_height_at_or_below_min_y() {
        let mut rng = RandomSourceStub;
        let placement = HeightmapPlacement::on_heightmap(HeightmapTypes::OceanFloor);

        let ctx_eq = PlacementContext::new(64, |_hm, _x, _z| 64);
        assert!(placement
            .get_positions(&ctx_eq, &mut rng, BlockPos::new(0, 0, 0))
            .is_empty());

        let ctx_below = PlacementContext::new(64, |_hm, _x, _z| 63);
        assert!(placement
            .get_positions(&ctx_below, &mut rng, BlockPos::new(1, 2, 3))
            .is_empty());
    }

    #[test]
    fn uses_input_x_z_and_selected_heightmap() {
        let ctx = PlacementContext::new(i32::MIN, |hm, x, z| {
            assert_eq!(hm, HeightmapTypes::MotionBlockingNoLeaves);
            x + z
        });
        let mut rng = RandomSourceStub;
        let placement = HeightmapPlacement::on_heightmap(HeightmapTypes::MotionBlockingNoLeaves);

        let result = placement.get_positions(&ctx, &mut rng, BlockPos::new(-5, 999, 12));
        assert_eq!(result, vec![BlockPos::new(-5, 7, 12)]);
    }

    #[test]
    fn handles_negative_and_extreme_coordinates() {
        let ctx = PlacementContext::new(i32::MIN, |_hm, x, z| {
            if x == i32::MIN && z == i32::MAX {
                i32::MAX
            } else {
                -100
            }
        });
        let mut rng = RandomSourceStub;
        let placement = HeightmapPlacement::on_heightmap(HeightmapTypes::WorldSurface);

        let extreme =
            placement.get_positions(&ctx, &mut rng, BlockPos::new(i32::MIN, 0, i32::MAX));
        assert_eq!(
            extreme,
            vec![BlockPos::new(i32::MIN, i32::MAX, i32::MAX)]
        );

        let negative = placement.get_positions(&ctx, &mut rng, BlockPos::new(-1, -2, -3));
        assert_eq!(negative, vec![BlockPos::new(-1, -100, -3)]);
    }

    #[test]
    fn type_returns_heightmap() {
        let placement = HeightmapPlacement::on_heightmap(HeightmapTypes::OceanFloorWg);
        assert_eq!(placement.type_(), PlacementModifierType::Heightmap);
    }
}
