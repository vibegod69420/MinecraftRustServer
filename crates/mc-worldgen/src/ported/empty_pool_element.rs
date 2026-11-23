use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vec3i {
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BoundingBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Rotation {
    #[default]
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Projection {
    TerrainMatching,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructurePoolElementType {
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LiquidSettings;

#[derive(Debug, Default)]
pub struct StructureTemplateManager;

#[derive(Debug, Default)]
pub struct WorldGenLevel;

#[derive(Debug, Default)]
pub struct StructureManager;

#[derive(Debug, Default)]
pub struct ChunkGenerator;

#[derive(Debug, Default)]
pub struct RandomSource;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JigsawBlockInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmptyPoolElement {
    projection: Projection,
}

impl EmptyPoolElement {
    pub const INSTANCE: Self = Self {
        projection: Projection::TerrainMatching,
    };

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.EmptyPoolElement#<init>
    fn new() -> Self {
        Self {
            projection: Projection::TerrainMatching,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.EmptyPoolElement#getSize
    pub fn get_size(&self, _template_manager: &StructureTemplateManager, _rotation: Rotation) -> Vec3i {
        Vec3i::ZERO
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.EmptyPoolElement#getShuffledJigsawBlocks
    pub fn get_shuffled_jigsaw_blocks(
        &self,
        _template_manager: &StructureTemplateManager,
        _pos: BlockPos,
        _rotation: Rotation,
        _random: &RandomSource,
    ) -> Vec<JigsawBlockInfo> {
        Vec::new()
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.EmptyPoolElement#getBoundingBox
    pub fn get_bounding_box(
        &self,
        _template_manager: &StructureTemplateManager,
        _pos: BlockPos,
        _rotation: Rotation,
    ) -> BoundingBox {
        panic!("Invalid call to EmptyPoolElement.getBoundingBox, filter me!");
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.EmptyPoolElement#place
    pub fn place(
        &self,
        _template_manager: &StructureTemplateManager,
        _level: &mut WorldGenLevel,
        _structure_manager: &StructureManager,
        _chunk_generator: &ChunkGenerator,
        _pos: BlockPos,
        _pos2: BlockPos,
        _rotation: Rotation,
        _bounding_box: &BoundingBox,
        _random: &RandomSource,
        _liquid_settings: LiquidSettings,
        _keep_jigsaws: bool,
    ) -> bool {
        true
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.pools.EmptyPoolElement#getType
    pub fn get_type(&self) -> StructurePoolElementType {
        StructurePoolElementType::Empty
    }

    /// Returns the projection used by this element.
    pub fn projection(&self) -> Projection {
        self.projection
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.pools.EmptyPoolElement#toString
impl fmt::Display for EmptyPoolElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Empty")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_and_type() {
        let el = EmptyPoolElement::INSTANCE;
        assert_eq!(el.get_type(), StructurePoolElementType::Empty);
        assert_eq!(el.projection(), Projection::TerrainMatching);
        assert_eq!(el, EmptyPoolElement::new());
    }

    #[test]
    fn get_size_is_zero() {
        let size = EmptyPoolElement::INSTANCE.get_size(&StructureTemplateManager, Rotation::None);
        assert_eq!(size, Vec3i::ZERO);
    }

    #[test]
    fn shuffled_jigsaw_blocks_empty_with_extreme_positions() {
        let blocks = EmptyPoolElement::INSTANCE.get_shuffled_jigsaw_blocks(
            &StructureTemplateManager,
            BlockPos {
                x: i32::MIN,
                y: -64,
                z: i32::MAX,
            },
            Rotation::Clockwise180,
            &RandomSource,
        );
        assert!(blocks.is_empty());
    }

    #[test]
    #[should_panic(expected = "Invalid call to EmptyPoolElement.getBoundingBox, filter me!")]
    fn get_bounding_box_panics() {
        let _ = EmptyPoolElement::INSTANCE.get_bounding_box(
            &StructureTemplateManager,
            BlockPos::default(),
            Rotation::Counterclockwise90,
        );
    }

    #[test]
    fn place_returns_true_with_negative_inputs() {
        let mut level = WorldGenLevel;
        let ok = EmptyPoolElement::INSTANCE.place(
            &StructureTemplateManager,
            &mut level,
            &StructureManager,
            &ChunkGenerator,
            BlockPos {
                x: i32::MIN,
                y: -1,
                z: i32::MAX,
            },
            BlockPos { x: -30000000, y: 320, z: -30000000 },
            Rotation::Clockwise90,
            &BoundingBox {
                min_x: i32::MIN,
                min_y: i32::MIN,
                min_z: i32::MIN,
                max_x: i32::MAX,
                max_y: i32::MAX,
                max_z: i32::MAX,
            },
            &RandomSource,
            LiquidSettings,
            true,
        );
        assert!(ok);
    }

    #[test]
    fn to_string_is_empty() {
        assert_eq!(EmptyPoolElement::INSTANCE.to_string(), "Empty");
    }
}
