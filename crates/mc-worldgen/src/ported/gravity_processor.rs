use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HeightmapType {
    WorldSurfaceWg,
    WorldSurface,
    OceanFloorWg,
    OceanFloor,
    MotionBlocking,
    MotionBlockingNoLeaves,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#<init>(int,int,int)
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.Vec3i#getX
    pub fn get_x(&self) -> i32 {
        self.x
    }

    // JAVA: net.minecraft.core.Vec3i#getY
    pub fn get_y(&self) -> i32 {
        self.y
    }

    // JAVA: net.minecraft.core.Vec3i#getZ
    pub fn get_z(&self) -> i32 {
        self.z
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BlockState {
    pub id: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NbtTag;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructureBlockInfo {
    pos: BlockPos,
    state: BlockState,
    nbt: Option<NbtTag>,
}

impl StructureBlockInfo {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate$StructureBlockInfo#<init>
    pub fn new(pos: BlockPos, state: BlockState, nbt: Option<NbtTag>) -> Self {
        Self { pos, state, nbt }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate$StructureBlockInfo#pos
    pub fn pos(&self) -> BlockPos {
        self.pos
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate$StructureBlockInfo#state
    pub fn state(&self) -> &BlockState {
        &self.state
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate$StructureBlockInfo#nbt
    pub fn nbt(&self) -> Option<&NbtTag> {
        self.nbt.as_ref()
    }
}

#[derive(Clone, Debug, Default)]
pub struct StructurePlaceSettings;

pub trait LevelReader {
    fn get_height(&self, heightmap: HeightmapType, x: i32, z: i32) -> i32;
    fn is_server_level(&self) -> bool;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructureProcessorType {
    Gravity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GravityProcessor {
    heightmap: HeightmapType,
    offset: i32,
}

impl GravityProcessor {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.GravityProcessor#<init>
    pub fn new(heightmap: HeightmapType, offset: i32) -> Self {
        Self { heightmap, offset }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.GravityProcessor#processBlock
    pub fn process_block(
        &self,
        level: &dyn LevelReader,
        _block_pos: BlockPos,
        _pivot: BlockPos,
        original: &StructureBlockInfo,
        current: &StructureBlockInfo,
        _settings: &StructurePlaceSettings,
    ) -> Option<StructureBlockInfo> {
        let heightmap_type = if level.is_server_level() {
            if self.heightmap == HeightmapType::WorldSurfaceWg {
                HeightmapType::WorldSurface
            } else if self.heightmap == HeightmapType::OceanFloorWg {
                HeightmapType::OceanFloor
            } else {
                self.heightmap
            }
        } else {
            self.heightmap
        };

        let current_pos = current.pos();
        let height = level
            .get_height(heightmap_type, current_pos.get_x(), current_pos.get_z())
            .wrapping_add(self.offset);
        let original_y = original.pos().get_y();
        Some(StructureBlockInfo::new(
            BlockPos::new(
                current_pos.get_x(),
                height.wrapping_add(original_y),
                current_pos.get_z(),
            ),
            *current.state(),
            current.nbt().cloned(),
        ))
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.GravityProcessor#getType
    pub fn get_type(&self) -> StructureProcessorType {
        StructureProcessorType::Gravity
    }

    pub fn heightmap(&self) -> HeightmapType {
        self.heightmap
    }

    pub fn offset(&self) -> i32 {
        self.offset
    }
}

impl fmt::Display for HeightmapType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            HeightmapType::WorldSurfaceWg => "WORLD_SURFACE_WG",
            HeightmapType::WorldSurface => "WORLD_SURFACE",
            HeightmapType::OceanFloorWg => "OCEAN_FLOOR_WG",
            HeightmapType::OceanFloor => "OCEAN_FLOOR",
            HeightmapType::MotionBlocking => "MOTION_BLOCKING",
            HeightmapType::MotionBlockingNoLeaves => "MOTION_BLOCKING_NO_LEAVES",
        };
        f.write_str(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockLevel {
        server: bool,
        heights: HashMap<(HeightmapType, i32, i32), i32>,
    }

    impl MockLevel {
        fn new(server: bool) -> Self {
            Self {
                server,
                heights: HashMap::new(),
            }
        }

        fn with_height(mut self, hm: HeightmapType, x: i32, z: i32, h: i32) -> Self {
            self.heights.insert((hm, x, z), h);
            self
        }
    }

    impl LevelReader for MockLevel {
        fn get_height(&self, heightmap: HeightmapType, x: i32, z: i32) -> i32 {
            *self.heights.get(&(heightmap, x, z)).unwrap_or(&0)
        }

        fn is_server_level(&self) -> bool {
            self.server
        }
    }

    fn info(x: i32, y: i32, z: i32, state_id: u32) -> StructureBlockInfo {
        StructureBlockInfo::new(
            BlockPos::new(x, y, z),
            BlockState { id: state_id },
            None,
        )
    }

    #[test]
    fn server_remaps_world_surface_wg() {
        let level = MockLevel::new(true).with_height(HeightmapType::WorldSurface, 10, 20, 64);
        let proc = GravityProcessor::new(HeightmapType::WorldSurfaceWg, 0);
        let original = info(0, 5, 0, 1);
        let current = info(10, 0, 20, 2);
        let out = proc
            .process_block(
                &level,
                BlockPos::default(),
                BlockPos::default(),
                &original,
                &current,
                &StructurePlaceSettings,
            )
            .unwrap();
        assert_eq!(out.pos(), BlockPos::new(10, 69, 20));
        assert_eq!(out.state().id, 2);
    }

    #[test]
    fn server_remaps_ocean_floor_wg() {
        let level = MockLevel::new(true).with_height(HeightmapType::OceanFloor, 1, 2, 30);
        let proc = GravityProcessor::new(HeightmapType::OceanFloorWg, -1);
        let original = info(0, 3, 0, 7);
        let current = info(1, 0, 2, 7);
        let out = proc
            .process_block(
                &level,
                BlockPos::default(),
                BlockPos::default(),
                &original,
                &current,
                &StructurePlaceSettings,
            )
            .unwrap();
        assert_eq!(out.pos(), BlockPos::new(1, 32, 2)); // (30 + -1) + 3
    }

    #[test]
    fn non_server_keeps_wg_type() {
        let level = MockLevel::new(false).with_height(HeightmapType::WorldSurfaceWg, 5, 6, 10);
        let proc = GravityProcessor::new(HeightmapType::WorldSurfaceWg, 2);
        let original = info(0, 1, 0, 3);
        let current = info(5, 0, 6, 3);
        let out = proc
            .process_block(
                &level,
                BlockPos::default(),
                BlockPos::default(),
                &original,
                &current,
                &StructurePlaceSettings,
            )
            .unwrap();
        assert_eq!(out.pos(), BlockPos::new(5, 13, 6));
    }

    #[test]
    fn server_non_wg_type_unchanged() {
        let level = MockLevel::new(true).with_height(HeightmapType::MotionBlocking, -3, 7, 50);
        let proc = GravityProcessor::new(HeightmapType::MotionBlocking, 0);
        let original = info(0, -2, 0, 9);
        let current = info(-3, 100, 7, 9);
        let out = proc
            .process_block(
                &level,
                BlockPos::default(),
                BlockPos::default(),
                &original,
                &current,
                &StructurePlaceSettings,
            )
            .unwrap();
        assert_eq!(out.pos(), BlockPos::new(-3, 48, 7));
    }

    #[test]
    fn wrapping_integer_addition() {
        let level = MockLevel::new(false).with_height(HeightmapType::WorldSurface, 0, 0, i32::MAX);
        let proc = GravityProcessor::new(HeightmapType::WorldSurface, 1);
        let original = info(0, 1, 0, 1);
        let current = info(0, 0, 0, 1);
        let out = proc
            .process_block(
                &level,
                BlockPos::default(),
                BlockPos::default(),
                &original,
                &current,
                &StructurePlaceSettings,
            )
            .unwrap();
        // height = i32::MAX.wrapping_add(1) = i32::MIN, then + original_y(1) = i32::MIN+1
        assert_eq!(out.pos().get_y(), i32::MIN.wrapping_add(1));
    }

    #[test]
    fn get_type_is_gravity() {
        let proc = GravityProcessor::new(HeightmapType::WorldSurface, 0);
        assert_eq!(proc.get_type(), StructureProcessorType::Gravity);
    }
}
