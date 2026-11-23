#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Block(pub u32);

impl Block {
    pub const AIR: Block = Block(0);
    pub const STRUCTURE_BLOCK: Block = Block(1);
    pub const STONE: Block = Block(2);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub block: Block,
}

impl BlockState {
    // JAVA: net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#getBlock
    pub fn get_block(&self) -> Block {
        self.block
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructureBlockInfo {
    pos: BlockPos,
    state: BlockState,
    nbt: Option<u8>,
}

impl StructureBlockInfo {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate$StructureBlockInfo#<init>
    pub fn new(pos: BlockPos, state: BlockState, nbt: Option<u8>) -> Self {
        Self { pos, state, nbt }
    }

    pub fn pos(&self) -> BlockPos {
        self.pos
    }

    pub fn state(&self) -> &BlockState {
        &self.state
    }

    pub fn nbt(&self) -> Option<&u8> {
        self.nbt.as_ref()
    }
}

#[derive(Clone, Debug, Default)]
pub struct StructurePlaceSettings;

pub trait LevelReader {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructureProcessorType {
    BlockIgnore,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockIgnoreProcessor {
    to_ignore: Vec<Block>,
}

impl BlockIgnoreProcessor {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockIgnoreProcessor#<init>
    pub fn new(to_ignore: Vec<Block>) -> Self {
        Self { to_ignore }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockIgnoreProcessor#processBlock
    pub fn process_block(
        &self,
        _level: &dyn LevelReader,
        _block_pos: BlockPos,
        _pivot: BlockPos,
        _original: &StructureBlockInfo,
        current: &StructureBlockInfo,
        _settings: &StructurePlaceSettings,
    ) -> Option<StructureBlockInfo> {
        if self.to_ignore.contains(&current.state().get_block()) {
            None
        } else {
            Some(current.clone())
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockIgnoreProcessor#getType
    pub fn get_type(&self) -> StructureProcessorType {
        StructureProcessorType::BlockIgnore
    }

    pub fn to_ignore(&self) -> &[Block] {
        &self.to_ignore
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockIgnoreProcessor#STRUCTURE_BLOCK
pub fn structure_block_ignore() -> BlockIgnoreProcessor {
    BlockIgnoreProcessor::new(vec![Block::STRUCTURE_BLOCK])
}

// JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockIgnoreProcessor#AIR
pub fn air_ignore() -> BlockIgnoreProcessor {
    BlockIgnoreProcessor::new(vec![Block::AIR])
}

// JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockIgnoreProcessor#STRUCTURE_AND_AIR
pub fn structure_and_air_ignore() -> BlockIgnoreProcessor {
    BlockIgnoreProcessor::new(vec![Block::AIR, Block::STRUCTURE_BLOCK])
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyLevel;
    impl LevelReader for DummyLevel {}

    fn make_info(block: Block) -> StructureBlockInfo {
        StructureBlockInfo::new(
            BlockPos::new(1, 2, 3),
            BlockState { block },
            Some(7),
        )
    }

    #[test]
    fn ignores_air() {
        let proc = air_ignore();
        let info = make_info(Block::AIR);
        let out = proc.process_block(
            &DummyLevel,
            BlockPos::default(),
            BlockPos::default(),
            &info,
            &info,
            &StructurePlaceSettings,
        );
        assert!(out.is_none());
        assert_eq!(proc.get_type(), StructureProcessorType::BlockIgnore);
    }

    #[test]
    fn keeps_non_ignored_block() {
        let proc = structure_block_ignore();
        let info = make_info(Block::STONE);
        let out = proc
            .process_block(
                &DummyLevel,
                BlockPos::default(),
                BlockPos::default(),
                &info,
                &info,
                &StructurePlaceSettings,
            )
            .unwrap();
        assert_eq!(out, info);
    }

    #[test]
    fn structure_and_air_ignores_both() {
        let proc = structure_and_air_ignore();
        let air = make_info(Block::AIR);
        let sb = make_info(Block::STRUCTURE_BLOCK);
        let stone = make_info(Block::STONE);
        let ctx = (
            &DummyLevel as &dyn LevelReader,
            BlockPos::default(),
            BlockPos::default(),
            &StructurePlaceSettings,
        );
        assert!(proc
            .process_block(ctx.0, ctx.1, ctx.2, &air, &air, ctx.3)
            .is_none());
        assert!(proc
            .process_block(ctx.0, ctx.1, ctx.2, &sb, &sb, ctx.3)
            .is_none());
        assert!(proc
            .process_block(ctx.0, ctx.1, ctx.2, &stone, &stone, ctx.3)
            .is_some());
    }

    #[test]
    fn empty_ignore_keeps_everything() {
        let proc = BlockIgnoreProcessor::new(vec![]);
        let info = make_info(Block::AIR);
        let out = proc.process_block(
            &DummyLevel,
            BlockPos::default(),
            BlockPos::default(),
            &info,
            &info,
            &StructurePlaceSettings,
        );
        assert_eq!(out.unwrap().state().get_block(), Block::AIR);
    }

    #[test]
    fn to_ignore_exposes_list() {
        let proc = BlockIgnoreProcessor::new(vec![Block::STONE, Block::AIR]);
        assert_eq!(proc.to_ignore(), &[Block::STONE, Block::AIR]);
    }
}
