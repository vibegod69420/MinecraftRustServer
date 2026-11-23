use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Half {
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlabType {
    Top,
    Bottom,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Block {
    Cobblestone,
    MossyCobblestone,
    Stone,
    StoneBricks,
    MossyStoneBricks,
    CobblestoneStairs,
    MossyCobblestoneStairs,
    StoneStairs,
    StoneBrickStairs,
    MossyStoneBrickStairs,
    CobblestoneSlab,
    MossyCobblestoneSlab,
    SmoothStoneSlab,
    StoneSlab,
    StoneBrickSlab,
    MossyStoneBrickSlab,
    StoneBrickWall,
    MossyStoneBrickWall,
    CobblestoneWall,
    MossyCobblestoneWall,
    ChiseledStoneBricks,
    CrackedStoneBricks,
    IronBars,
    Blackstone,
    PolishedBlackstone,
    PolishedBlackstoneBricks,
    BlackstoneStairs,
    PolishedBlackstoneStairs,
    PolishedBlackstoneBrickStairs,
    BlackstoneSlab,
    PolishedBlackstoneSlab,
    PolishedBlackstoneBrickSlab,
    PolishedBlackstoneBrickWall,
    BlackstoneWall,
    ChiseledPolishedBlackstone,
    CrackedPolishedBlackstoneBricks,
    IronChain,
    /// Fallback for blocks not relevant to this processor.
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub block: Block,
    pub facing: Option<Direction>,
    pub half: Option<Half>,
    pub slab_type: Option<SlabType>,
}

impl BlockState {
    pub fn new(block: Block) -> Self {
        Self {
            block,
            facing: None,
            half: None,
            slab_type: None,
        }
    }
}

impl Block {
    pub fn default_block_state(self) -> BlockState {
        BlockState::new(self)
    }
}

/// Minimal NBT placeholder for a self-contained port.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NbtCompound;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureBlockInfo {
    pub pos: BlockPos,
    pub state: BlockState,
    pub nbt: Option<NbtCompound>,
}

pub trait LevelReader {}

#[derive(Debug, Clone, Copy, Default)]
pub struct StructurePlaceSettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureProcessorType {
    BlackstoneReplace,
}

#[derive(Debug)]
pub struct BlackstoneReplaceProcessor {
    replacements: HashMap<Block, Block>,
}

pub static INSTANCE: LazyLock<BlackstoneReplaceProcessor> =
    LazyLock::new(BlackstoneReplaceProcessor::new);

impl BlackstoneReplaceProcessor {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlackstoneReplaceProcessor#<init>
    fn new() -> Self {
        let mut replacements = HashMap::new();
        replacements.insert(Block::Cobblestone, Block::Blackstone);
        replacements.insert(Block::MossyCobblestone, Block::Blackstone);
        replacements.insert(Block::Stone, Block::PolishedBlackstone);
        replacements.insert(Block::StoneBricks, Block::PolishedBlackstoneBricks);
        replacements.insert(Block::MossyStoneBricks, Block::PolishedBlackstoneBricks);
        replacements.insert(Block::CobblestoneStairs, Block::BlackstoneStairs);
        replacements.insert(Block::MossyCobblestoneStairs, Block::BlackstoneStairs);
        replacements.insert(Block::StoneStairs, Block::PolishedBlackstoneStairs);
        replacements.insert(Block::StoneBrickStairs, Block::PolishedBlackstoneBrickStairs);
        replacements.insert(
            Block::MossyStoneBrickStairs,
            Block::PolishedBlackstoneBrickStairs,
        );
        replacements.insert(Block::CobblestoneSlab, Block::BlackstoneSlab);
        replacements.insert(Block::MossyCobblestoneSlab, Block::BlackstoneSlab);
        replacements.insert(Block::SmoothStoneSlab, Block::PolishedBlackstoneSlab);
        replacements.insert(Block::StoneSlab, Block::PolishedBlackstoneSlab);
        replacements.insert(Block::StoneBrickSlab, Block::PolishedBlackstoneBrickSlab);
        replacements.insert(
            Block::MossyStoneBrickSlab,
            Block::PolishedBlackstoneBrickSlab,
        );
        replacements.insert(Block::StoneBrickWall, Block::PolishedBlackstoneBrickWall);
        replacements.insert(
            Block::MossyStoneBrickWall,
            Block::PolishedBlackstoneBrickWall,
        );
        replacements.insert(Block::CobblestoneWall, Block::BlackstoneWall);
        replacements.insert(Block::MossyCobblestoneWall, Block::BlackstoneWall);
        replacements.insert(
            Block::ChiseledStoneBricks,
            Block::ChiseledPolishedBlackstone,
        );
        replacements.insert(
            Block::CrackedStoneBricks,
            Block::CrackedPolishedBlackstoneBricks,
        );
        replacements.insert(Block::IronBars, Block::IronChain);
        Self { replacements }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlackstoneReplaceProcessor#processBlock
    pub fn process_block(
        &self,
        _level: &dyn LevelReader,
        _pos: BlockPos,
        _pivot: BlockPos,
        _original: &StructureBlockInfo,
        current: &StructureBlockInfo,
        _settings: &StructurePlaceSettings,
    ) -> StructureBlockInfo {
        let Some(&replacement) = self.replacements.get(&current.state.block) else {
            return current.clone();
        };

        let mut new_state = replacement.default_block_state();
        if let Some(facing) = current.state.facing {
            new_state.facing = Some(facing);
        }
        if let Some(half) = current.state.half {
            new_state.half = Some(half);
        }
        if let Some(slab_type) = current.state.slab_type {
            new_state.slab_type = Some(slab_type);
        }

        StructureBlockInfo {
            pos: current.pos,
            state: new_state,
            nbt: current.nbt.clone(),
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.BlackstoneReplaceProcessor#getType
    pub fn get_type(&self) -> StructureProcessorType {
        StructureProcessorType::BlackstoneReplace
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EmptyLevel;
    impl LevelReader for EmptyLevel {}

    fn run(current: StructureBlockInfo) -> StructureBlockInfo {
        let dummy_pos = BlockPos { x: 0, y: 0, z: 0 };
        let original = StructureBlockInfo {
            pos: dummy_pos,
            state: BlockState::new(Block::Stone),
            nbt: None,
        };
        INSTANCE.process_block(
            &EmptyLevel,
            dummy_pos,
            dummy_pos,
            &original,
            &current,
            &StructurePlaceSettings,
        )
    }

    #[test]
    fn unmapped_block_passthrough_preserves_negative_and_extreme_coords() {
        let current = StructureBlockInfo {
            pos: BlockPos {
                x: i32::MIN,
                y: -1,
                z: i32::MAX,
            },
            state: BlockState::new(Block::Other),
            nbt: Some(NbtCompound),
        };
        let out = run(current.clone());
        assert_eq!(out, current);
    }

    #[test]
    fn simple_replacement_preserves_nbt_and_pos() {
        let pos = BlockPos { x: 1, y: 64, z: -2 };
        let current = StructureBlockInfo {
            pos,
            state: BlockState::new(Block::Cobblestone),
            nbt: Some(NbtCompound),
        };
        let out = run(current);
        assert_eq!(out.pos, pos);
        assert_eq!(out.state.block, Block::Blackstone);
        assert_eq!(out.state.facing, None);
        assert_eq!(out.state.half, None);
        assert_eq!(out.state.slab_type, None);
        assert_eq!(out.nbt, Some(NbtCompound));
    }

    #[test]
    fn stair_properties_are_copied_from_source() {
        let mut state = BlockState::new(Block::StoneBrickStairs);
        state.facing = Some(Direction::East);
        state.half = Some(Half::Top);
        let current = StructureBlockInfo {
            pos: BlockPos { x: -10, y: 5, z: 3 },
            state,
            nbt: None,
        };
        let out = run(current);
        assert_eq!(out.state.block, Block::PolishedBlackstoneBrickStairs);
        assert_eq!(out.state.facing, Some(Direction::East));
        assert_eq!(out.state.half, Some(Half::Top));
        assert_eq!(out.state.slab_type, None);
    }

    #[test]
    fn slab_type_is_copied_from_source() {
        let mut state = BlockState::new(Block::MossyStoneBrickSlab);
        state.slab_type = Some(SlabType::Double);
        let current = StructureBlockInfo {
            pos: BlockPos { x: 0, y: -64, z: 7 },
            state,
            nbt: None,
        };
        let out = run(current);
        assert_eq!(out.state.block, Block::PolishedBlackstoneBrickSlab);
        assert_eq!(out.state.slab_type, Some(SlabType::Double));
        assert_eq!(out.state.facing, None);
        assert_eq!(out.state.half, None);
    }

    #[test]
    fn iron_bars_replaced_and_extreme_position_preserved() {
        let pos = BlockPos {
            x: i32::MAX,
            y: i32::MIN,
            z: -123_456,
        };
        let current = StructureBlockInfo {
            pos,
            state: BlockState::new(Block::IronBars),
            nbt: None,
        };
        let out = run(current);
        assert_eq!(out.state.block, Block::IronChain);
        assert_eq!(out.state, Block::IronChain.default_block_state());
        assert_eq!(out.pos, pos);
    }

    #[test]
    fn get_type_reports_blackstone_replace() {
        assert_eq!(INSTANCE.get_type(), StructureProcessorType::BlackstoneReplace);
    }
}
