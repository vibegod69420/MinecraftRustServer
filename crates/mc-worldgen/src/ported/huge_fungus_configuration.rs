/// Minimal self-contained representation of a Minecraft block state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct BlockState {
    pub id: u32,
}

/// Minimal self-contained representation of a block predicate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct BlockPredicate {
    pub id: u32,
}

// NOTE: Codec/HugeFungusConfiguration CODEC is omitted in this self-contained port.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HugeFungusConfiguration {
    pub valid_base_state: BlockState,
    pub stem_state: BlockState,
    pub hat_state: BlockState,
    pub decor_state: BlockState,
    pub replaceable_blocks: BlockPredicate,
    pub planted: bool,
}

impl HugeFungusConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.HugeFungusConfiguration#<init>
    pub fn new(
        valid_base_state: BlockState,
        stem_state: BlockState,
        hat_state: BlockState,
        decor_state: BlockState,
        replaceable_blocks: BlockPredicate,
        planted: bool,
    ) -> Self {
        Self {
            valid_base_state,
            stem_state,
            hat_state,
            decor_state,
            replaceable_blocks,
            planted,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_all_fields() {
        let cfg = HugeFungusConfiguration::new(
            BlockState { id: 1 },
            BlockState { id: 2 },
            BlockState { id: 3 },
            BlockState { id: 4 },
            BlockPredicate { id: 5 },
            true,
        );
        assert_eq!(cfg.valid_base_state, BlockState { id: 1 });
        assert_eq!(cfg.stem_state, BlockState { id: 2 });
        assert_eq!(cfg.hat_state, BlockState { id: 3 });
        assert_eq!(cfg.decor_state, BlockState { id: 4 });
        assert_eq!(cfg.replaceable_blocks, BlockPredicate { id: 5 });
        assert!(cfg.planted);
    }

    #[test]
    fn planted_false() {
        let cfg = HugeFungusConfiguration::new(
            BlockState { id: 10 },
            BlockState { id: 20 },
            BlockState { id: 30 },
            BlockState { id: 40 },
            BlockPredicate { id: 50 },
            false,
        );
        assert!(!cfg.planted);
        assert_eq!(cfg.stem_state.id, 20);
    }

    #[test]
    fn edge_case_max_ids() {
        let cfg = HugeFungusConfiguration::new(
            BlockState { id: u32::MAX },
            BlockState { id: u32::MAX - 1 },
            BlockState { id: 0 },
            BlockState { id: 1 },
            BlockPredicate { id: u32::MAX },
            true,
        );
        assert_eq!(cfg.valid_base_state.id, u32::MAX);
        assert_eq!(cfg.hat_state.id, 0);
        assert_eq!(cfg.replaceable_blocks.id, u32::MAX);
    }

    #[test]
    fn copy_and_equality() {
        let a = HugeFungusConfiguration::new(
            BlockState { id: 1 },
            BlockState { id: 2 },
            BlockState { id: 3 },
            BlockState { id: 4 },
            BlockPredicate { id: 5 },
            false,
        );
        let b = a;
        assert_eq!(a, b);
        let c = HugeFungusConfiguration::new(
            BlockState { id: 1 },
            BlockState { id: 2 },
            BlockState { id: 3 },
            BlockState { id: 4 },
            BlockPredicate { id: 5 },
            true,
        );
        assert_ne!(a, c);
    }

    #[test]
    fn distinct_configs_differ_by_any_field() {
        let base = HugeFungusConfiguration::new(
            BlockState { id: 1 },
            BlockState { id: 2 },
            BlockState { id: 3 },
            BlockState { id: 4 },
            BlockPredicate { id: 5 },
            false,
        );
        let diff_base = HugeFungusConfiguration::new(
            BlockState { id: 9 },
            BlockState { id: 2 },
            BlockState { id: 3 },
            BlockState { id: 4 },
            BlockPredicate { id: 5 },
            false,
        );
        assert_ne!(base, diff_base);
        assert_eq!(base.decor_state, diff_base.decor_state);
    }
}
