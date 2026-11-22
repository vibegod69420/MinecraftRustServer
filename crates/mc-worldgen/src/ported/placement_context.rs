#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BlockState(pub i32);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CarvingMask(pub Vec<u64>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightmapType {
    OceanFloor,
    WorldSurface,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacedFeature {
    pub id: u32,
}

pub trait WorldGenLevelLike {
    fn get_height(&self, heightmap: HeightmapType, x: i32, z: i32) -> i32;
    fn get_or_create_carving_mask(&mut self, chunk_pos: ChunkPos) -> CarvingMask;
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
    fn get_min_y(&self) -> i32;
}

pub trait ChunkGeneratorLike {}

#[derive(Debug)]
pub struct PlacementContext<L, G> {
    level: L,
    generator: G,
    top_feature: Option<PlacedFeature>,
}

impl<L, G> PlacementContext<L, G> {
    // JAVA: net.minecraft.world.level.levelgen.placement.PlacementContext#<init>
    pub fn new(level: L, generator: G, top_feature: Option<PlacedFeature>) -> Self {
        Self {
            level,
            generator,
            top_feature,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.PlacementContext#topFeature
    pub fn top_feature(&self) -> Option<&PlacedFeature> {
        self.top_feature.as_ref()
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.PlacementContext#generator
    pub fn generator(&self) -> &G {
        &self.generator
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.PlacementContext#getLevel
    pub fn get_level(&self) -> &L {
        &self.level
    }
}

impl<L: WorldGenLevelLike, G> PlacementContext<L, G> {
    // JAVA: net.minecraft.world.level.levelgen.placement.PlacementContext#getHeight
    pub fn get_height(&self, heightmap: HeightmapType, x: i32, z: i32) -> i32 {
        self.level.get_height(heightmap, x, z)
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.PlacementContext#getCarvingMask
    pub fn get_carving_mask(&mut self, chunk_pos: ChunkPos) -> CarvingMask {
        self.level.get_or_create_carving_mask(chunk_pos)
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.PlacementContext#getBlockState
    pub fn get_block_state(&self, pos: BlockPos) -> BlockState {
        self.level.get_block_state(pos)
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.PlacementContext#getMinY
    pub fn get_min_y(&self) -> i32 {
        self.level.get_min_y()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLevel {
        min_y: i32,
    }

    impl WorldGenLevelLike for MockLevel {
        fn get_height(&self, h: HeightmapType, x: i32, _z: i32) -> i32 {
            match h {
                HeightmapType::OceanFloor => x,
                HeightmapType::WorldSurface => x.wrapping_add(2),
            }
        }

        fn get_or_create_carving_mask(&mut self, pos: ChunkPos) -> CarvingMask {
            CarvingMask(vec![pos.x as u64, pos.z as u64])
        }

        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            BlockState(pos.y)
        }

        fn get_min_y(&self) -> i32 {
            self.min_y
        }
    }

    #[derive(Debug)]
    struct MockGen;
    impl ChunkGeneratorLike for MockGen {}

    #[test]
    fn delegates_height_and_min_y() {
        let ctx = PlacementContext::new(MockLevel { min_y: -64 }, MockGen, None);
        assert_eq!(ctx.get_height(HeightmapType::OceanFloor, 10, 20), 10);
        assert_eq!(ctx.get_height(HeightmapType::WorldSurface, 10, 20), 12);
        assert_eq!(ctx.get_min_y(), -64);
    }

    #[test]
    fn block_state_and_top_feature() {
        let feat = PlacedFeature { id: 7 };
        let ctx = PlacementContext::new(MockLevel { min_y: 0 }, MockGen, Some(feat.clone()));
        assert_eq!(
            ctx.get_block_state(BlockPos { x: 1, y: 42, z: 3 }),
            BlockState(42)
        );
        assert_eq!(ctx.top_feature(), Some(&feat));
    }

    #[test]
    fn carving_mask_generator_and_level() {
        let mut ctx = PlacementContext::new(MockLevel { min_y: 0 }, MockGen, None);
        let mask = ctx.get_carving_mask(ChunkPos { x: 3, z: -2 });
        assert_eq!(mask, CarvingMask(vec![3u64, (-2i32) as u64]));
        let _g: &MockGen = ctx.generator();
        let _l: &MockLevel = ctx.get_level();
    }

    #[test]
    fn negative_coordinates_and_wrapping_height() {
        let ctx = PlacementContext::new(MockLevel { min_y: i32::MIN }, MockGen, None);
        assert_eq!(
            ctx.get_height(HeightmapType::WorldSurface, i32::MAX, 0),
            i32::MAX.wrapping_add(2)
        );
        assert_eq!(ctx.get_min_y(), i32::MIN);
    }
}
