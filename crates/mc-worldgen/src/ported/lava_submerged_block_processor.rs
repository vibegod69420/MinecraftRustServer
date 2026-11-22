#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    Air,
    Lava,
    Stone,
    Slab,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoxelShape {
    pub is_full_block: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub kind: BlockKind,
    pub is_full_block_shape: bool,
}

impl BlockState {
    // JAVA: net.minecraft.world.level.block.state.BlockState#is(net.minecraft.world.level.block.Block)
    pub fn is(&self, block: BlockKind) -> bool {
        self.kind == block
    }

    // JAVA: net.minecraft.world.level.block.state.BlockState#getShape
    pub fn get_shape(&self, _level: &dyn LevelReader, _pos: &BlockPos) -> VoxelShape {
        VoxelShape {
            is_full_block: self.is_full_block_shape,
        }
    }

    // JAVA: net.minecraft.world.level.block.Block#defaultBlockState
    pub fn lava_default() -> Self {
        Self {
            kind: BlockKind::Lava,
            is_full_block_shape: true,
        }
    }
}

pub struct Block;

impl Block {
    // JAVA: net.minecraft.world.level.block.Block#isShapeFullBlock
    pub fn is_shape_full_block(shape: &VoxelShape) -> bool {
        shape.is_full_block
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureBlockInfo {
    pub pos: BlockPos,
    pub state: BlockState,
    pub nbt: Option<Vec<u8>>,
}

pub trait LevelReader {
    fn get_block_state(&self, pos: &BlockPos) -> BlockState;
}

#[derive(Debug, Clone, Default)]
pub struct StructurePlaceSettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureProcessorType {
    LavaSubmergedBlock,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LavaSubmergedBlockProcessor;

impl LavaSubmergedBlockProcessor {
    pub const INSTANCE: Self = Self;

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.LavaSubmergedBlockProcessor#processBlock
    pub fn process_block(
        &self,
        level: &dyn LevelReader,
        _pos1: &BlockPos,
        _pos2: &BlockPos,
        _info3: &StructureBlockInfo,
        info4: &StructureBlockInfo,
        _settings: &StructurePlaceSettings,
    ) -> Option<StructureBlockInfo> {
        let pos6 = info4.pos;
        let is_lava = level.get_block_state(&pos6).is(BlockKind::Lava);
        if is_lava && !Block::is_shape_full_block(&info4.state.get_shape(level, &pos6)) {
            Some(StructureBlockInfo {
                pos: pos6,
                state: BlockState::lava_default(),
                nbt: info4.nbt.clone(),
            })
        } else {
            Some(info4.clone())
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.LavaSubmergedBlockProcessor#getType
    pub fn get_type(&self) -> StructureProcessorType {
        StructureProcessorType::LavaSubmergedBlock
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLevel {
        state: BlockState,
    }

    impl LevelReader for MockLevel {
        fn get_block_state(&self, _pos: &BlockPos) -> BlockState {
            self.state.clone()
        }
    }

    fn pos(x: i32, y: i32, z: i32) -> BlockPos {
        BlockPos { x, y, z }
    }

    fn info(p: BlockPos, kind: BlockKind, full: bool, nbt: Option<Vec<u8>>) -> StructureBlockInfo {
        StructureBlockInfo {
            pos: p,
            state: BlockState {
                kind,
                is_full_block_shape: full,
            },
            nbt,
        }
    }

    #[test]
    fn replaces_non_full_block_submerged_in_lava() {
        let proc = LavaSubmergedBlockProcessor;
        let level = MockLevel {
            state: BlockState {
                kind: BlockKind::Lava,
                is_full_block_shape: true,
            },
        };
        let p = pos(1, 2, 3);
        let info4 = info(p, BlockKind::Slab, false, Some(vec![1, 2, 3]));
        let info3 = info(pos(0, 0, 0), BlockKind::Air, false, None);
        let r = proc
            .process_block(
                &level,
                &pos(0, 0, 0),
                &pos(0, 0, 0),
                &info3,
                &info4,
                &StructurePlaceSettings,
            )
            .unwrap();
        assert_eq!(r.pos, p);
        assert_eq!(r.state.kind, BlockKind::Lava);
        assert!(r.state.is_full_block_shape);
        assert_eq!(r.nbt, Some(vec![1, 2, 3]));
    }

    #[test]
    fn keeps_full_block_submerged_in_lava() {
        let proc = LavaSubmergedBlockProcessor;
        let level = MockLevel {
            state: BlockState {
                kind: BlockKind::Lava,
                is_full_block_shape: true,
            },
        };
        let p = pos(0, 0, 0);
        let info4 = info(p, BlockKind::Stone, true, None);
        let info3 = info(p, BlockKind::Air, false, None);
        let r = proc
            .process_block(&level, &p, &p, &info3, &info4, &StructurePlaceSettings)
            .unwrap();
        assert_eq!(r, info4);
    }

    #[test]
    fn keeps_non_full_block_when_not_in_lava() {
        let proc = LavaSubmergedBlockProcessor;
        let level = MockLevel {
            state: BlockState {
                kind: BlockKind::Air,
                is_full_block_shape: false,
            },
        };
        let p = pos(-5, 64, 100);
        let info4 = info(p, BlockKind::Slab, false, None);
        let info3 = info(pos(0, 0, 0), BlockKind::Air, false, None);
        let r = proc
            .process_block(&level, &p, &p, &info3, &info4, &StructurePlaceSettings)
            .unwrap();
        assert_eq!(r, info4);
    }

    #[test]
    fn works_with_negative_coordinates_and_preserves_nbt() {
        let proc = LavaSubmergedBlockProcessor;
        let level = MockLevel {
            state: BlockState {
                kind: BlockKind::Lava,
                is_full_block_shape: true,
            },
        };
        let p = pos(-10, -64, -1);
        let info4 = info(p, BlockKind::Other, false, Some(vec![42]));
        let info3 = info(pos(0, 0, 0), BlockKind::Air, false, None);
        let r = proc
            .process_block(&level, &p, &p, &info3, &info4, &StructurePlaceSettings)
            .unwrap();
        assert_eq!(r.pos, p);
        assert_eq!(r.state, BlockState::lava_default());
        assert_eq!(r.nbt, Some(vec![42]));
    }

    #[test]
    fn get_type_and_instance() {
        assert_eq!(
            LavaSubmergedBlockProcessor::INSTANCE.get_type(),
            StructureProcessorType::LavaSubmergedBlock
        );
    }
}
