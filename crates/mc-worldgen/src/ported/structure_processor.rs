/// Minimal block position stub.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Minimal representation of `StructureTemplate.StructureBlockInfo`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureBlockInfo {
    pub pos: BlockPos,
    pub state_id: i32,
    pub nbt_tag: Option<String>,
}

/// Minimal structure place settings stub.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StructurePlaceSettings;

/// Minimal level reader stub.
pub trait LevelReader {}

/// Minimal server level accessor stub.
pub trait ServerLevelAccessor: LevelReader {}

/// Minimal structure processor type stub.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructureProcessorType(pub &'static str);

/// Port of `net.minecraft.world.level.levelgen.structure.templatesystem.StructureProcessor`.
pub trait StructureProcessor {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructureProcessor#processBlock
    fn process_block(
        &self,
        _level: &dyn LevelReader,
        _offset: BlockPos,
        _pos: BlockPos,
        _original_info: StructureBlockInfo,
        rel_info: StructureBlockInfo,
        _settings: &StructurePlaceSettings,
    ) -> Option<StructureBlockInfo> {
        Some(rel_info)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructureProcessor#getType
    fn get_type(&self) -> StructureProcessorType;

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructureProcessor#finalizeProcessing
    fn finalize_processing(
        &self,
        _level: &dyn ServerLevelAccessor,
        _offset: BlockPos,
        _pos: BlockPos,
        _original_list: Vec<StructureBlockInfo>,
        rel_list: Vec<StructureBlockInfo>,
        _settings: &StructurePlaceSettings,
    ) -> Vec<StructureBlockInfo> {
        rel_list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyProcessor;
    impl StructureProcessor for DummyProcessor {
        fn get_type(&self) -> StructureProcessorType {
            StructureProcessorType("dummy")
        }
    }

    struct DummyLevel;
    impl LevelReader for DummyLevel {}
    impl ServerLevelAccessor for DummyLevel {}

    fn info(x: i32, y: i32, z: i32, state_id: i32, nbt_tag: Option<&str>) -> StructureBlockInfo {
        StructureBlockInfo {
            pos: BlockPos { x, y, z },
            state_id,
            nbt_tag: nbt_tag.map(|s| s.to_string()),
        }
    }

    #[test]
    fn process_block_returns_rel_info_by_default() {
        let p = DummyProcessor;
        let level = DummyLevel;
        let original = info(1, 2, 3, 5, None);
        let rel = info(4, 5, 6, 7, Some("nbt"));
        let result = p.process_block(
            &level,
            BlockPos { x: 0, y: 0, z: 0 },
            BlockPos { x: 10, y: 20, z: 30 },
            original,
            rel.clone(),
            &StructurePlaceSettings,
        );
        assert_eq!(result, Some(rel));
    }

    #[test]
    fn process_block_with_negative_and_extreme_positions() {
        let p = DummyProcessor;
        let level = DummyLevel;
        let original = info(i32::MIN, -1, i32::MAX, -1, None);
        let rel = info(-100, 0, 200, 42, None);
        let result = p.process_block(
            &level,
            BlockPos {
                x: -5,
                y: -10,
                z: -15,
            },
            BlockPos {
                x: i32::MIN,
                y: i32::MAX,
                z: 0,
            },
            original,
            rel.clone(),
            &StructurePlaceSettings,
        );
        assert_eq!(result, Some(rel));
    }

    #[test]
    fn finalize_processing_returns_rel_list_by_default() {
        let p = DummyProcessor;
        let level = DummyLevel;
        let rel_list = vec![info(-1, -64, 100, 0, None), info(7, 8, 9, -2, Some("t"))];
        let result = p.finalize_processing(
            &level,
            BlockPos { x: 0, y: 0, z: 0 },
            BlockPos { x: 1, y: 2, z: 3 },
            vec![info(0, 0, 0, 1, None)],
            rel_list.clone(),
            &StructurePlaceSettings,
        );
        assert_eq!(result, rel_list);
    }

    #[test]
    fn finalize_processing_empty_list() {
        let p = DummyProcessor;
        let level = DummyLevel;
        let result = p.finalize_processing(
            &level,
            BlockPos { x: 0, y: 0, z: 0 },
            BlockPos { x: 0, y: 0, z: 0 },
            vec![],
            vec![],
            &StructurePlaceSettings,
        );
        assert!(result.is_empty());
    }

    #[test]
    fn get_type_returns_configured_type() {
        let p = DummyProcessor;
        assert_eq!(p.get_type(), StructureProcessorType("dummy"));
    }
}
