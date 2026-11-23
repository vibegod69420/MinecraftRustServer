use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Block(pub u32);

impl Block {
    pub const AIR: Block = Block(0);
    pub const STONE: Block = Block(1);
    pub const DIRT: Block = Block(2);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub block: Block,
}

impl BlockState {
    pub fn get_block(&self) -> Block {
        self.block
    }

    pub fn is_in(&self, blocks: &HashSet<Block>) -> bool {
        blocks.contains(&self.block)
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
    pub fn new(pos: BlockPos, state: BlockState, nbt: Option<u8>) -> Self {
        Self { pos, state, nbt }
    }

    pub fn pos(&self) -> BlockPos {
        self.pos
    }

    pub fn state(&self) -> &BlockState {
        &self.state
    }
}

pub trait RandomSource {
    fn next_float(&mut self) -> f32;
}

pub struct FixedRandom {
    value: f32,
}

impl RandomSource for FixedRandom {
    fn next_float(&mut self) -> f32 {
        self.value
    }
}

#[derive(Clone, Debug)]
pub struct StructurePlaceSettings {
    random_value: f32,
}

impl StructurePlaceSettings {
    pub fn new(random_value: f32) -> Self {
        Self { random_value }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructurePlaceSettings#getRandom
    pub fn get_random(&self, _pos: BlockPos) -> FixedRandom {
        FixedRandom {
            value: self.random_value,
        }
    }
}

pub trait LevelReader {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructureProcessorType {
    BlockRot,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlockRotProcessor {
    rottable_blocks: Option<HashSet<Block>>,
    integrity: f32,
}

impl BlockRotProcessor {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockRotProcessor#<init>(HolderSet,float)
    pub fn new_with_set(rottable_blocks: HashSet<Block>, integrity: f32) -> Self {
        Self {
            rottable_blocks: Some(rottable_blocks),
            integrity,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockRotProcessor#<init>(float)
    pub fn new(integrity: f32) -> Self {
        Self {
            rottable_blocks: None,
            integrity,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockRotProcessor#processBlock
    pub fn process_block(
        &self,
        _level: &dyn LevelReader,
        _block_pos: BlockPos,
        _pivot: BlockPos,
        original: &StructureBlockInfo,
        current: &StructureBlockInfo,
        settings: &StructurePlaceSettings,
    ) -> Option<StructureBlockInfo> {
        let mut random = settings.get_random(current.pos());
        let can_rot = match &self.rottable_blocks {
            None => true,
            Some(blocks) => original.state().is_in(blocks),
        };
        if can_rot && !(random.next_float() <= self.integrity) {
            None
        } else {
            Some(current.clone())
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlockRotProcessor#getType
    pub fn get_type(&self) -> StructureProcessorType {
        StructureProcessorType::BlockRot
    }

    pub fn integrity(&self) -> f32 {
        self.integrity
    }

    pub fn rottable_blocks(&self) -> Option<&HashSet<Block>> {
        self.rottable_blocks.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyLevel;
    impl LevelReader for DummyLevel {}

    fn make_info(block: Block) -> StructureBlockInfo {
        StructureBlockInfo::new(BlockPos::new(1, 2, 3), BlockState { block }, None)
    }

    #[test]
    fn integrity_one_keeps_block() {
        let proc = BlockRotProcessor::new(1.0f32);
        let info = make_info(Block::STONE);
        let out = proc.process_block(
            &DummyLevel,
            BlockPos::default(),
            BlockPos::default(),
            &info,
            &info,
            &StructurePlaceSettings::new(0.999f32),
        );
        assert_eq!(out, Some(info));
    }

    #[test]
    fn integrity_zero_rots_when_random_positive() {
        let proc = BlockRotProcessor::new(0.0f32);
        let info = make_info(Block::STONE);
        let out = proc.process_block(
            &DummyLevel,
            BlockPos::default(),
            BlockPos::default(),
            &info,
            &info,
            &StructurePlaceSettings::new(0.5f32),
        );
        assert!(out.is_none());
    }

    #[test]
    fn integrity_zero_keeps_when_random_is_zero() {
        let proc = BlockRotProcessor::new(0.0f32);
        let info = make_info(Block::DIRT);
        let out = proc.process_block(
            &DummyLevel,
            BlockPos::default(),
            BlockPos::default(),
            &info,
            &info,
            &StructurePlaceSettings::new(0.0f32),
        );
        assert!(out.is_some());
    }

    #[test]
    fn rottable_set_filters_blocks() {
        let set = HashSet::from([Block::STONE]);
        let proc = BlockRotProcessor::new_with_set(set.clone(), 0.0f32);
        let stone = make_info(Block::STONE);
        let dirt = make_info(Block::DIRT);
        let settings = StructurePlaceSettings::new(0.9f32);

        let stone_out = proc.process_block(
            &DummyLevel,
            BlockPos::default(),
            BlockPos::default(),
            &stone,
            &stone,
            &settings,
        );
        let dirt_out = proc.process_block(
            &DummyLevel,
            BlockPos::default(),
            BlockPos::default(),
            &dirt,
            &dirt,
            &settings,
        );
        assert!(stone_out.is_none());
        assert_eq!(dirt_out, Some(dirt));
        assert_eq!(proc.rottable_blocks(), Some(&set));
    }

    #[test]
    fn get_type_and_integrity_accessor() {
        let proc = BlockRotProcessor::new(0.75f32);
        assert_eq!(proc.get_type(), StructureProcessorType::BlockRot);
        assert!((proc.integrity() - 0.75f32).abs() < 1e-6);
        assert!(proc.rottable_blocks().is_none());
    }
}
