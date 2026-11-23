/// Placeholder for `net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProvider`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockStateProvider {
    pub id: u32,
}

/// Placeholder for `net.minecraft.world.level.block.state.BlockState`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockState {
    pub id: u32,
}

/// Placeholder for `net.minecraft.tags.TagKey<net.minecraft.world.level.block.Block>`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockTagKey {
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeodeBlockSettings {
    pub filling_provider: BlockStateProvider,
    pub inner_layer_provider: BlockStateProvider,
    pub alternate_inner_layer_provider: BlockStateProvider,
    pub middle_layer_provider: BlockStateProvider,
    pub outer_layer_provider: BlockStateProvider,
    pub inner_placements: Vec<BlockState>,
    pub cannot_replace: BlockTagKey,
    pub invalid_blocks: BlockTagKey,
}

impl GeodeBlockSettings {
    // JAVA: net.minecraft.world.level.levelgen.GeodeBlockSettings#<init>
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        filling_provider: BlockStateProvider,
        inner_layer_provider: BlockStateProvider,
        alternate_inner_layer_provider: BlockStateProvider,
        middle_layer_provider: BlockStateProvider,
        outer_layer_provider: BlockStateProvider,
        inner_placements: Vec<BlockState>,
        cannot_replace: BlockTagKey,
        invalid_blocks: BlockTagKey,
    ) -> Self {
        Self {
            filling_provider,
            inner_layer_provider,
            alternate_inner_layer_provider,
            middle_layer_provider,
            outer_layer_provider,
            inner_placements,
            cannot_replace,
            invalid_blocks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provider(id: u32) -> BlockStateProvider {
        BlockStateProvider { id }
    }

    fn state(id: u32) -> BlockState {
        BlockState { id }
    }

    fn tag(id: u32) -> BlockTagKey {
        BlockTagKey { id }
    }

    fn sample() -> GeodeBlockSettings {
        GeodeBlockSettings::new(
            provider(1),
            provider(2),
            provider(3),
            provider(4),
            provider(5),
            vec![state(10), state(11)],
            tag(100),
            tag(101),
        )
    }

    #[test]
    fn new_stores_all_fields() {
        let s = sample();
        assert_eq!(s.filling_provider, provider(1));
        assert_eq!(s.inner_layer_provider, provider(2));
        assert_eq!(s.alternate_inner_layer_provider, provider(3));
        assert_eq!(s.middle_layer_provider, provider(4));
        assert_eq!(s.outer_layer_provider, provider(5));
        assert_eq!(s.inner_placements, vec![state(10), state(11)]);
        assert_eq!(s.cannot_replace, tag(100));
        assert_eq!(s.invalid_blocks, tag(101));
    }

    #[test]
    fn empty_inner_placements() {
        let s = GeodeBlockSettings::new(
            provider(0),
            provider(0),
            provider(0),
            provider(0),
            provider(0),
            vec![],
            tag(1),
            tag(2),
        );
        assert!(s.inner_placements.is_empty());
        assert_eq!(s.cannot_replace, tag(1));
        assert_eq!(s.invalid_blocks, tag(2));
    }

    #[test]
    fn clone_preserves_equality() {
        let s = sample();
        let cloned = s.clone();
        assert_eq!(s, cloned);
    }

    #[test]
    fn different_fields_are_not_equal() {
        let a = sample();
        let mut b = sample();
        b.invalid_blocks = tag(999);
        assert_ne!(a, b);

        let mut c = sample();
        c.inner_placements.push(state(12));
        assert_ne!(a, c);
    }
}
