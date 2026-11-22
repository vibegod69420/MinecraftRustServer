use std::marker::PhantomData;

/// Minimal block identifier stub.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Block {
    pub id: u32,
}

/// Minimal tag key stub.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TagKey<T> {
    pub name: &'static str,
    _marker: PhantomData<fn() -> T>,
}

impl<T> TagKey<T> {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }
}

/// Minimal block state stub with tag membership.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockState {
    pub block: Block,
    pub tags: &'static [&'static str],
}

impl BlockState {
    pub fn is_tag(&self, tag: &TagKey<Block>) -> bool {
        let mut i = 0;
        while i < self.tags.len() {
            if self.tags[i] == tag.name {
                return true;
            }
            i += 1;
        }
        false
    }
}

/// Minimal random source stub.
#[derive(Debug, Default)]
pub struct RandomSource;

/// Placeholder for Mojang's serialization `MapCodec`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapCodec<T>(PhantomData<fn() -> T>);

impl<T> MapCodec<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

/// Minimal rule test type stub.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleTestType {
    BlockTest,
    TagTest,
}

/// Port of `net.minecraft.world.level.levelgen.structure.templatesystem.TagMatchTest`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagMatchTest {
    tag: TagKey<Block>,
}

pub const CODEC: MapCodec<TagMatchTest> = MapCodec::new();

impl TagMatchTest {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.TagMatchTest#<init>
    pub fn new(tag: TagKey<Block>) -> Self {
        Self { tag }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.TagMatchTest#test
    pub fn test(&self, state: &BlockState, _random: &mut RandomSource) -> bool {
        state.is_tag(&self.tag)
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.TagMatchTest#getType
    pub fn get_type(&self) -> RuleTestType {
        RuleTestType::TagTest
    }

    pub fn tag(&self) -> &TagKey<Block> {
        &self.tag
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_present_tag() {
        let rule = TagMatchTest::new(TagKey::<Block>::new("minecraft:logs"));
        let state = BlockState {
            block: Block { id: 1 },
            tags: &["minecraft:logs", "minecraft:oak_logs"],
        };
        let mut rng = RandomSource;
        assert!(rule.test(&state, &mut rng));
    }

    #[test]
    fn test_does_not_match_missing_tag() {
        let rule = TagMatchTest::new(TagKey::<Block>::new("minecraft:logs"));
        let state = BlockState {
            block: Block { id: 1 },
            tags: &["minecraft:planks"],
        };
        let mut rng = RandomSource;
        assert!(!rule.test(&state, &mut rng));
    }

    #[test]
    fn test_empty_tags_never_match() {
        let rule = TagMatchTest::new(TagKey::<Block>::new("minecraft:logs"));
        let state = BlockState {
            block: Block { id: 0 },
            tags: &[],
        };
        let mut rng = RandomSource;
        assert!(!rule.test(&state, &mut rng));
    }

    #[test]
    fn get_type_is_tag_test() {
        let rule = TagMatchTest::new(TagKey::<Block>::new("a"));
        assert_eq!(rule.get_type(), RuleTestType::TagTest);
    }

    #[test]
    fn test_tag_name_must_match_exactly() {
        let rule = TagMatchTest::new(TagKey::<Block>::new("minecraft:log"));
        let state = BlockState {
            block: Block { id: u32::MAX },
            tags: &["minecraft:logs"],
        };
        let mut rng = RandomSource;
        assert!(!rule.test(&state, &mut rng));
    }
}
