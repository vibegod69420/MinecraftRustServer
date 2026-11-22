use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vec3i {
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };

    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn offset(&self, offset: &Vec3i) -> Self {
        Self {
            x: self.x.wrapping_add(offset.x),
            y: self.y.wrapping_add(offset.y),
            z: self.z.wrapping_add(offset.z),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Block(pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderSet<T: Debug + Clone + PartialEq> {
    pub entries: Vec<T>,
}

impl<T: Debug + Clone + PartialEq> HolderSet<T> {
    pub fn new(entries: Vec<T>) -> Self {
        Self { entries }
    }

    pub fn contains(&self, value: &T) -> bool {
        self.entries.iter().any(|e| e == value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub block: Block,
}

impl BlockState {
    pub fn new(block: Block) -> Self {
        Self { block }
    }

    pub fn is_block(&self, blocks: &HolderSet<Block>) -> bool {
        blocks.contains(&self.block)
    }
}

pub trait WorldGenLevel {
    fn get_block_state(&self, pos: &BlockPos) -> BlockState;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockPredicateType {
    MatchingBlocks,
}

pub trait BlockPredicate {
    fn test(&self, level: &dyn WorldGenLevel, pos: &BlockPos) -> bool;
    fn predicate_type(&self) -> BlockPredicateType;
}

/// Port of `net.minecraft.world.level.levelgen.blockpredicates.MatchingBlocksPredicate`.
/// Extends `StateTestingPredicate` in Java; the offset+lookup delegation is inlined here
/// for self-containment.
#[derive(Debug, Clone)]
pub struct MatchingBlocksPredicate {
    offset: Vec3i,
    blocks: HolderSet<Block>,
}

impl MatchingBlocksPredicate {
    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.MatchingBlocksPredicate#<init>(Vec3i,HolderSet)
    pub fn new(offset: Vec3i, blocks: HolderSet<Block>) -> Self {
        Self { offset, blocks }
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.StateTestingPredicate#test(WorldGenLevel,BlockPos)
    pub fn test_at(&self, level: &dyn WorldGenLevel, pos: &BlockPos) -> bool {
        let state = level.get_block_state(&pos.offset(&self.offset));
        self.test_state(&state)
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.MatchingBlocksPredicate#test(BlockState)
    fn test_state(&self, state: &BlockState) -> bool {
        state.is_block(&self.blocks)
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.MatchingBlocksPredicate#type()
    pub fn get_type(&self) -> BlockPredicateType {
        BlockPredicateType::MatchingBlocks
    }
}

impl BlockPredicate for MatchingBlocksPredicate {
    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.StateTestingPredicate#test(WorldGenLevel,BlockPos)
    fn test(&self, level: &dyn WorldGenLevel, pos: &BlockPos) -> bool {
        self.test_at(level, pos)
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.MatchingBlocksPredicate#type()
    fn predicate_type(&self) -> BlockPredicateType {
        self.get_type()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockLevel {
        states: HashMap<(i32, i32, i32), BlockState>,
        default: BlockState,
    }

    impl MockLevel {
        fn new(default_block: u32) -> Self {
            Self {
                states: HashMap::new(),
                default: BlockState::new(Block(default_block)),
            }
        }

        fn set(&mut self, x: i32, y: i32, z: i32, block_id: u32) {
            self.states
                .insert((x, y, z), BlockState::new(Block(block_id)));
        }
    }

    impl WorldGenLevel for MockLevel {
        fn get_block_state(&self, pos: &BlockPos) -> BlockState {
            self.states
                .get(&(pos.x, pos.y, pos.z))
                .cloned()
                .unwrap_or_else(|| self.default.clone())
        }
    }

    fn holder(ids: &[u32]) -> HolderSet<Block> {
        HolderSet::new(ids.iter().copied().map(Block).collect())
    }

    #[test]
    fn matches_block_in_set() {
        let pred = MatchingBlocksPredicate::new(Vec3i::ZERO, holder(&[1, 2, 3]));
        let mut level = MockLevel::new(0);
        level.set(5, 10, -3, 2);
        assert!(pred.test(&level, &BlockPos::new(5, 10, -3)));
        assert!(!pred.test(&level, &BlockPos::new(0, 0, 0)));
    }

    #[test]
    fn applies_offset() {
        let pred = MatchingBlocksPredicate::new(Vec3i::new(1, -2, 3), holder(&[7]));
        let mut level = MockLevel::new(0);
        level.set(11, 8, 13, 7);
        assert!(pred.test(&level, &BlockPos::new(10, 10, 10)));
        assert!(!pred.test(&level, &BlockPos::new(11, 8, 13)));
    }

    #[test]
    fn negative_coordinates_and_offset() {
        let pred = MatchingBlocksPredicate::new(Vec3i::new(-1, -1, -1), holder(&[5]));
        let mut level = MockLevel::new(0);
        level.set(-11, -21, -31, 5);
        assert!(pred.test(&level, &BlockPos::new(-10, -20, -30)));
    }

    #[test]
    fn offset_wrapping_overflow() {
        let pred = MatchingBlocksPredicate::new(Vec3i::new(1, 0, 0), holder(&[9]));
        let mut level = MockLevel::new(0);
        level.set(i32::MIN, 0, 0, 9);
        assert!(pred.test(&level, &BlockPos::new(i32::MAX, 0, 0)));
    }

    #[test]
    fn predicate_type_is_matching_blocks() {
        let pred = MatchingBlocksPredicate::new(Vec3i::ZERO, holder(&[]));
        assert_eq!(pred.predicate_type(), BlockPredicateType::MatchingBlocks);
        assert_eq!(pred.get_type(), BlockPredicateType::MatchingBlocks);
    }
}
