use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vec3i {
    pub const ZERO: Vec3i = Vec3i { x: 0, y: 0, z: 0 };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const ZERO: BlockPos = BlockPos { x: 0, y: 0, z: 0 };

    // JAVA: net.minecraft.core.BlockPos#offset(int, int, int)
    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> BlockPos {
        BlockPos {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Block;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagKey<T> {
    pub name: String,
    _marker: PhantomData<T>,
}

impl<T> TagKey<T> {
    // JAVA: net.minecraft.tags.TagKey#create(...)
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub tags: Vec<String>,
}

impl BlockState {
    // JAVA: net.minecraft.world.level.block.state.BlockState#is(net.minecraft.tags.TagKey)
    pub fn is(&self, tag: &TagKey<Block>) -> bool {
        self.tags.iter().any(|t| t == &tag.name)
    }
}

pub trait WorldGenLevel {
    fn get_block_state(&self, pos: &BlockPos) -> BlockState;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockPredicateType {
    MATCHING_BLOCK_TAG,
    UNOBSTRUCTED,
    INSIDE_WORLD_BOUNDS,
    OTHER,
}

pub struct MapCodec<T>(PhantomData<T>);

impl<T> MapCodec<T> {
    pub const INSTANCE: Self = Self(PhantomData);
}

// JAVA: net.minecraft.world.level.levelgen.blockpredicates.StateTestingPredicate
pub struct StateTestingPredicateBase {
    pub offset: Vec3i,
}

impl StateTestingPredicateBase {
    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.StateTestingPredicate#<init>
    pub fn new(offset: Vec3i) -> Self {
        Self { offset }
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.StateTestingPredicate#test(net.minecraft.world.level.WorldGenLevel, net.minecraft.core.BlockPos)
    pub fn test_at<L: WorldGenLevel>(&self, level: &L, pos: &BlockPos, test_state: impl Fn(&BlockState) -> bool) -> bool {
        let target = pos.offset(self.offset.x, self.offset.y, self.offset.z);
        let state = level.get_block_state(&target);
        test_state(&state)
    }
}

// JAVA: net.minecraft.world.level.levelgen.blockpredicates.MatchingBlockTagPredicate
pub struct MatchingBlockTagPredicate {
    base: StateTestingPredicateBase,
    pub tag: TagKey<Block>,
}

impl MatchingBlockTagPredicate {
    pub const CODEC: MapCodec<Self> = MapCodec::<Self>::INSTANCE;

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.MatchingBlockTagPredicate#<init>
    pub fn new(offset: Vec3i, tag: TagKey<Block>) -> Self {
        Self {
            base: StateTestingPredicateBase::new(offset),
            tag,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.MatchingBlockTagPredicate#test(net.minecraft.world.level.block.state.BlockState)
    fn test_state(&self, state: &BlockState) -> bool {
        state.is(&self.tag)
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.MatchingBlockTagPredicate#test(net.minecraft.world.level.WorldGenLevel, net.minecraft.core.BlockPos)
    pub fn test<L: WorldGenLevel>(&self, level: &L, pos: &BlockPos) -> bool {
        let tag = &self.tag;
        self.base.test_at(level, pos, |s| s.is(tag))
    }

    // JAVA: net.minecraft.world.level.levelgen.blockpredicates.MatchingBlockTagPredicate#type
    pub fn predicate_type(&self) -> BlockPredicateType {
        BlockPredicateType::MATCHING_BLOCK_TAG
    }

    pub fn offset(&self) -> Vec3i {
        self.base.offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLevel {
        state: BlockState,
    }

    impl WorldGenLevel for MockLevel {
        fn get_block_state(&self, _pos: &BlockPos) -> BlockState {
            self.state.clone()
        }
    }

    #[test]
    fn test_matching_tag_returns_true() {
        let pred = MatchingBlockTagPredicate::new(
            Vec3i::ZERO,
            TagKey::new("minecraft:logs"),
        );
        let level = MockLevel {
            state: BlockState {
                tags: vec!["minecraft:logs".to_string(), "minecraft:mineable/axe".to_string()],
            },
        };
        assert!(pred.test(&level, &BlockPos::ZERO));
        assert_eq!(pred.predicate_type(), BlockPredicateType::MATCHING_BLOCK_TAG);
    }

    #[test]
    fn test_non_matching_tag_returns_false() {
        let pred = MatchingBlockTagPredicate::new(
            Vec3i::ZERO,
            TagKey::new("minecraft:logs"),
        );
        let level = MockLevel {
            state: BlockState {
                tags: vec!["minecraft:stone".to_string()],
            },
        };
        assert!(!pred.test(&level, &BlockPos::ZERO));
    }

    #[test]
    fn test_offset_is_applied() {
        struct PosTrackingLevel {
            expected: BlockPos,
            state: BlockState,
        }
        impl WorldGenLevel for PosTrackingLevel {
            fn get_block_state(&self, pos: &BlockPos) -> BlockState {
                assert_eq!(*pos, self.expected);
                self.state.clone()
            }
        }

        let pred = MatchingBlockTagPredicate::new(
            Vec3i { x: 1, y: -2, z: 3 },
            TagKey::new("test:tag"),
        );
        let level = PosTrackingLevel {
            expected: BlockPos { x: 11, y: 8, z: 23 },
            state: BlockState { tags: vec!["test:tag".to_string()] },
        };
        assert!(pred.test(&level, &BlockPos { x: 10, y: 10, z: 20 }));
        assert_eq!(pred.offset(), Vec3i { x: 1, y: -2, z: 3 });
    }

    #[test]
    fn test_offset_wrapping() {
        struct PosTrackingLevel {
            expected: BlockPos,
        }
        impl WorldGenLevel for PosTrackingLevel {
            fn get_block_state(&self, pos: &BlockPos) -> BlockState {
                assert_eq!(*pos, self.expected);
                BlockState { tags: vec![] }
            }
        }

        let pred = MatchingBlockTagPredicate::new(
            Vec3i { x: 1, y: 0, z: 0 },
            TagKey::new("t"),
        );
        let level = PosTrackingLevel {
            expected: BlockPos { x: i32::MIN, y: 0, z: 0 },
        };
        assert!(!pred.test(&level, &BlockPos { x: i32::MAX, y: 0, z: 0 }));
    }

    #[test]
    fn test_empty_tags() {
        let pred = MatchingBlockTagPredicate::new(Vec3i::ZERO, TagKey::new("any:tag"));
        let level = MockLevel {
            state: BlockState { tags: vec![] },
        };
        assert!(!pred.test(&level, &BlockPos { x: -100, y: 64, z: 200 }));
    }
}
