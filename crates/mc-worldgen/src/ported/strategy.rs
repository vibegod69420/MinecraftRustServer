use std::rc::Rc;

/// Minimal identity map abstraction used by [`Strategy`].
pub trait IdMap {
    fn size(&self) -> i32;
}

/// Minimal palette factory marker matching `Palette.Factory` usage in `Strategy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteFactory {
    SingleValue,
    Linear,
    HashMap,
}

/// Minimal palette configuration matching `Configuration.Simple` and `Configuration.Global`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Configuration {
    Simple {
        factory: PaletteFactory,
        bits: i32,
    },
    Global {
        global_palette_bits_in_memory: i32,
        bits_in_storage: i32,
    },
}

/// Minimal global palette wrapper used by [`Strategy`].
#[derive(Debug, Clone)]
pub struct GlobalPalette<M> {
    map: Rc<M>,
}

impl<M: IdMap> GlobalPalette<M> {
    fn new(map: Rc<M>) -> Self {
        Self { map }
    }

    pub fn map(&self) -> &M {
        &self.map
    }
}

// JAVA: net.minecraft.world.level.chunk.Strategy#ZERO_BITS
pub const ZERO_BITS: Configuration = Configuration::Simple {
    factory: PaletteFactory::SingleValue,
    bits: 0,
};
// JAVA: net.minecraft.world.level.chunk.Strategy#ONE_BIT_LINEAR
pub const ONE_BIT_LINEAR: Configuration = Configuration::Simple {
    factory: PaletteFactory::Linear,
    bits: 1,
};
// JAVA: net.minecraft.world.level.chunk.Strategy#TWO_BITS_LINEAR
pub const TWO_BITS_LINEAR: Configuration = Configuration::Simple {
    factory: PaletteFactory::Linear,
    bits: 2,
};
// JAVA: net.minecraft.world.level.chunk.Strategy#THREE_BITS_LINEAR
pub const THREE_BITS_LINEAR: Configuration = Configuration::Simple {
    factory: PaletteFactory::Linear,
    bits: 3,
};
// JAVA: net.minecraft.world.level.chunk.Strategy#FOUR_BITS_LINEAR
pub const FOUR_BITS_LINEAR: Configuration = Configuration::Simple {
    factory: PaletteFactory::Linear,
    bits: 4,
};
// JAVA: net.minecraft.world.level.chunk.Strategy#FIVE_BITS_HASHMAP
pub const FIVE_BITS_HASHMAP: Configuration = Configuration::Simple {
    factory: PaletteFactory::HashMap,
    bits: 5,
};
// JAVA: net.minecraft.world.level.chunk.Strategy#SIX_BITS_HASHMAP
pub const SIX_BITS_HASHMAP: Configuration = Configuration::Simple {
    factory: PaletteFactory::HashMap,
    bits: 6,
};
// JAVA: net.minecraft.world.level.chunk.Strategy#SEVEN_BITS_HASHMAP
pub const SEVEN_BITS_HASHMAP: Configuration = Configuration::Simple {
    factory: PaletteFactory::HashMap,
    bits: 7,
};
// JAVA: net.minecraft.world.level.chunk.Strategy#EIGHT_BITS_HASHMAP
pub const EIGHT_BITS_HASHMAP: Configuration = Configuration::Simple {
    factory: PaletteFactory::HashMap,
    bits: 8,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StrategyKind {
    BlockStates,
    Biomes,
}

#[derive(Debug, Clone)]
pub struct Strategy<M> {
    global_map: Rc<M>,
    global_palette: GlobalPalette<M>,
    pub global_palette_bits_in_memory: i32,
    bits_per_axis: i32,
    entry_count: i32,
    kind: StrategyKind,
}

impl<M: IdMap> Strategy<M> {
    // JAVA: net.minecraft.world.level.chunk.Strategy#<init>
    fn new(global_map: M, bits_per_axis: i32, kind: StrategyKind) -> Self {
        let global_map = Rc::new(global_map);
        let global_palette = GlobalPalette::new(Rc::clone(&global_map));
        let global_palette_bits_in_memory =
            minimum_bits_required_for_distinct_values(global_map.size());
        let entry_count = 1_i32 << ((bits_per_axis.wrapping_mul(3)) & 31);
        Self {
            global_map,
            global_palette,
            global_palette_bits_in_memory,
            bits_per_axis,
            entry_count,
            kind,
        }
    }

    // JAVA: net.minecraft.world.level.chunk.Strategy#createForBlockStates
    pub fn create_for_block_states(global_map: M) -> Self {
        Self::new(global_map, 4, StrategyKind::BlockStates)
    }

    // JAVA: net.minecraft.world.level.chunk.Strategy#createForBiomes
    pub fn create_for_biomes(global_map: M) -> Self {
        Self::new(global_map, 2, StrategyKind::Biomes)
    }

    // JAVA: net.minecraft.world.level.chunk.Strategy#entryCount
    pub fn entry_count(&self) -> i32 {
        self.entry_count
    }

    // JAVA: net.minecraft.world.level.chunk.Strategy#getIndex
    pub fn get_index(&self, x: i32, y: i32, z: i32) -> i32 {
        let bpa = self.bits_per_axis & 31;
        ((y << bpa) | z) << bpa | x
    }

    // JAVA: net.minecraft.world.level.chunk.Strategy#globalMap
    pub fn global_map(&self) -> &M {
        &self.global_map
    }

    // JAVA: net.minecraft.world.level.chunk.Strategy#globalPalette
    pub fn global_palette(&self) -> &GlobalPalette<M> {
        &self.global_palette
    }

    // JAVA: net.minecraft.world.level.chunk.Strategy#getConfigurationForBitCount
    pub fn get_configuration_for_bit_count(&self, bits: i32) -> Configuration {
        match self.kind {
            StrategyKind::BlockStates => match bits {
                0 => ZERO_BITS,
                1 | 2 | 3 | 4 => FOUR_BITS_LINEAR,
                5 => FIVE_BITS_HASHMAP,
                6 => SIX_BITS_HASHMAP,
                7 => SEVEN_BITS_HASHMAP,
                8 => EIGHT_BITS_HASHMAP,
                _ => Configuration::Global {
                    global_palette_bits_in_memory: self.global_palette_bits_in_memory,
                    bits_in_storage: bits,
                },
            },
            StrategyKind::Biomes => match bits {
                0 => ZERO_BITS,
                1 => ONE_BIT_LINEAR,
                2 => TWO_BITS_LINEAR,
                3 => THREE_BITS_LINEAR,
                _ => Configuration::Global {
                    global_palette_bits_in_memory: self.global_palette_bits_in_memory,
                    bits_in_storage: bits,
                },
            },
        }
    }

    // JAVA: net.minecraft.world.level.chunk.Strategy#getConfigurationForPaletteSize
    pub fn get_configuration_for_palette_size(&self, size: i32) -> Configuration {
        let bits = minimum_bits_required_for_distinct_values(size);
        self.get_configuration_for_bit_count(bits)
    }
}

// JAVA: net.minecraft.world.level.chunk.Strategy#minimumBitsRequiredForDistinctValues
fn minimum_bits_required_for_distinct_values(size: i32) -> i32 {
    ceillog2(size)
}

// JAVA: net.minecraft.util.Mth#ceillog2
fn ceillog2(value: i32) -> i32 {
    if value > 0 {
        32_i32.wrapping_sub(value.wrapping_sub(1).leading_zeros() as i32)
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct MockIdMap {
        size: i32,
    }

    impl IdMap for MockIdMap {
        fn size(&self) -> i32 {
            self.size
        }
    }

    fn mock_map(size: i32) -> MockIdMap {
        MockIdMap { size }
    }

    #[test]
    fn test_minimum_bits_required_for_distinct_values_edge_cases() {
        assert_eq!(minimum_bits_required_for_distinct_values(0), 0);
        assert_eq!(minimum_bits_required_for_distinct_values(1), 0);
        assert_eq!(minimum_bits_required_for_distinct_values(2), 1);
        assert_eq!(minimum_bits_required_for_distinct_values(3), 2);
        assert_eq!(minimum_bits_required_for_distinct_values(4), 2);
        assert_eq!(minimum_bits_required_for_distinct_values(5), 3);
        assert_eq!(minimum_bits_required_for_distinct_values(16), 4);
        assert_eq!(minimum_bits_required_for_distinct_values(17), 5);
        assert_eq!(minimum_bits_required_for_distinct_values(-1), 0);
        assert_eq!(minimum_bits_required_for_distinct_values(i32::MIN), 0);
    }

    #[test]
    fn test_block_states_strategy_configuration() {
        let strategy = Strategy::create_for_block_states(mock_map(4096));
        assert_eq!(strategy.entry_count(), 4096);
        assert_eq!(strategy.get_configuration_for_bit_count(0), ZERO_BITS);
        assert_eq!(strategy.get_configuration_for_bit_count(1), FOUR_BITS_LINEAR);
        assert_eq!(strategy.get_configuration_for_bit_count(2), FOUR_BITS_LINEAR);
        assert_eq!(strategy.get_configuration_for_bit_count(3), FOUR_BITS_LINEAR);
        assert_eq!(strategy.get_configuration_for_bit_count(4), FOUR_BITS_LINEAR);
        assert_eq!(strategy.get_configuration_for_bit_count(5), FIVE_BITS_HASHMAP);
        assert_eq!(strategy.get_configuration_for_bit_count(6), SIX_BITS_HASHMAP);
        assert_eq!(strategy.get_configuration_for_bit_count(7), SEVEN_BITS_HASHMAP);
        assert_eq!(strategy.get_configuration_for_bit_count(8), EIGHT_BITS_HASHMAP);
        assert_eq!(
            strategy.get_configuration_for_bit_count(9),
            Configuration::Global {
                global_palette_bits_in_memory: strategy.global_palette_bits_in_memory,
                bits_in_storage: 9,
            }
        );
    }

    #[test]
    fn test_biome_strategy_configuration() {
        let strategy = Strategy::create_for_biomes(mock_map(256));
        assert_eq!(strategy.entry_count(), 64);
        assert_eq!(strategy.global_palette_bits_in_memory, 8);
        assert_eq!(strategy.get_configuration_for_bit_count(0), ZERO_BITS);
        assert_eq!(strategy.get_configuration_for_bit_count(1), ONE_BIT_LINEAR);
        assert_eq!(strategy.get_configuration_for_bit_count(2), TWO_BITS_LINEAR);
        assert_eq!(strategy.get_configuration_for_bit_count(3), THREE_BITS_LINEAR);
        assert_eq!(
            strategy.get_configuration_for_bit_count(4),
            Configuration::Global {
                global_palette_bits_in_memory: 8,
                bits_in_storage: 4,
            }
        );
    }

    #[test]
    fn test_get_index_and_negative_inputs() {
        let block_strategy = Strategy::create_for_block_states(mock_map(16));
        assert_eq!(block_strategy.get_index(0, 0, 0), 0);
        assert_eq!(block_strategy.get_index(15, 15, 15), 4095);
        assert_eq!(
            block_strategy.get_index(1, 2, 3),
            ((2_i32 << 4) | 3_i32) << 4 | 1_i32
        );

        let biome_strategy = Strategy::create_for_biomes(mock_map(16));
        assert_eq!(biome_strategy.get_index(0, 0, 0), 0);
        assert_eq!(biome_strategy.get_index(3, 3, 3), 63);

        // Negative inputs: bitwise behavior should match Java i32 semantics.
        let x = -1_i32;
        let y = -2_i32;
        let z = -3_i32;
        let expected = ((y << 4) | z) << 4 | x;
        assert_eq!(block_strategy.get_index(x, y, z), expected);
    }

    #[test]
    fn test_get_configuration_for_palette_size_and_accessors() {
        let strategy = Strategy::create_for_block_states(mock_map(1024));
        assert_eq!(strategy.global_palette_bits_in_memory, 10);
        assert_eq!(strategy.global_map().size(), 1024);
        assert_eq!(strategy.global_palette().map().size(), 1024);

        assert_eq!(strategy.get_configuration_for_palette_size(0), ZERO_BITS);
        assert_eq!(strategy.get_configuration_for_palette_size(1), ZERO_BITS);
        assert_eq!(strategy.get_configuration_for_palette_size(16), FOUR_BITS_LINEAR);
        assert_eq!(strategy.get_configuration_for_palette_size(17), FIVE_BITS_HASHMAP);
        assert_eq!(
            strategy.get_configuration_for_palette_size(1024),
            Configuration::Global {
                global_palette_bits_in_memory: 10,
                bits_in_storage: 10,
            }
        );
        assert_eq!(strategy.get_configuration_for_palette_size(-5), ZERO_BITS);
    }

    #[test]
    fn test_entry_count_overflow_semantics() {
        // Constructor uses wrapping_mul and masked shift, matching Java int semantics.
        let strategy = Strategy::<MockIdMap>::new(mock_map(1), 11, StrategyKind::BlockStates);
        // 11 * 3 = 33, 33 & 31 = 1, 1 << 1 = 2
        assert_eq!(strategy.entry_count(), 2);

        let strategy_neg = Strategy::<MockIdMap>::new(mock_map(1), -1, StrategyKind::BlockStates);
        // -1 * 3 = -3 in wrapping i32, -3 & 31 = 29, 1 << 29
        assert_eq!(strategy_neg.entry_count(), 1_i32 << 29);
    }
}
