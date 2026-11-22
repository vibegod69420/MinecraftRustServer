use std::fmt::Debug;

/// Minimal block state representation for self-contained porting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub id: i32,
}

/// Minimal block position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Minimal random source interface.
pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

/// Block state provider type discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockStateProviderType {
    SimpleStateProvider,
    RotatedBlockProvider,
    WeightedStateProvider,
}

/// Abstract base for block state providers.
// JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProvider
pub trait BlockStateProvider {
    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProvider#type
    fn provider_type(&self) -> BlockStateProviderType;

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProvider#getState
    fn get_state(&self, random: &mut dyn RandomSource, pos: BlockPos) -> BlockState;
}

/// Minimal weighted list entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightedEntry<T: Clone> {
    pub data: T,
    pub weight: i32,
}

/// Minimal weighted list, mirroring `net.minecraft.util.random.WeightedList`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightedList<T: Clone> {
    entries: Vec<WeightedEntry<T>>,
    total_weight: i32,
}

impl<T: Clone> WeightedList<T> {
    pub fn new(entries: Vec<WeightedEntry<T>>) -> Self {
        let mut total_weight: i32 = 0;
        for e in &entries {
            let w = if e.weight < 0 { 0 } else { e.weight };
            total_weight = total_weight.wrapping_add(w);
        }
        Self {
            entries,
            total_weight,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty() || self.total_weight <= 0
    }

    /// Returns a random element based on weights, or panics if empty/invalid.
    /// Faithful to `getRandomOrThrow` behavior: throws when no element can be selected.
    pub fn get_random_or_throw(&self, random: &mut dyn RandomSource) -> T {
        if self.is_empty() {
            panic!("WeightedList is empty");
        }
        let total = self.total_weight;
        let mut r = random.next_int(total);
        if r < 0 {
            r = r.rem_euclid(total);
        }
        for entry in &self.entries {
            let w = if entry.weight < 0 { 0 } else { entry.weight };
            r = r.wrapping_sub(w);
            if r < 0 {
                return entry.data.clone();
            }
        }
        // Fallback to last entry if rounding/weight issues occur.
        self.entries
            .last()
            .expect("WeightedList should not be empty")
            .data
            .clone()
    }
}

/// Builder for `WeightedList`.
#[derive(Debug, Clone, Default)]
pub struct WeightedListBuilder<T: Clone> {
    entries: Vec<WeightedEntry<T>>,
}

impl<T: Clone> WeightedListBuilder<T> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add(mut self, data: T, weight: i32) -> Self {
        self.entries.push(WeightedEntry { data, weight });
        self
    }

    pub fn build(self) -> WeightedList<T> {
        WeightedList::new(self.entries)
    }
}

/// A provider that selects a block state from a weighted list.
// JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.WeightedStateProvider
#[derive(Debug, Clone)]
pub struct WeightedStateProvider {
    pub weighted_list: WeightedList<BlockState>,
}

impl WeightedStateProvider {
    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.WeightedStateProvider#create
    fn create(weighted_list: WeightedList<BlockState>) -> Result<WeightedStateProvider, String> {
        if weighted_list.is_empty() {
            Err("WeightedStateProvider with no states".to_string())
        } else {
            Ok(WeightedStateProvider::new(weighted_list))
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.WeightedStateProvider#<init>
    pub fn new(weighted_list: WeightedList<BlockState>) -> Self {
        Self { weighted_list }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.WeightedStateProvider#<init>
    pub fn from_builder(builder: WeightedListBuilder<BlockState>) -> Self {
        Self::new(builder.build())
    }
}

impl BlockStateProvider for WeightedStateProvider {
    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.WeightedStateProvider#type
    fn provider_type(&self) -> BlockStateProviderType {
        BlockStateProviderType::WeightedStateProvider
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.WeightedStateProvider#getState
    fn get_state(&self, random: &mut dyn RandomSource, _pos: BlockPos) -> BlockState {
        self.weighted_list.get_random_or_throw(random)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedRandom(i32);
    impl RandomSource for FixedRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                0
            } else {
                self.0.rem_euclid(bound)
            }
        }
    }

    struct SeqRandom {
        values: Vec<i32>,
        idx: usize,
    }
    impl SeqRandom {
        fn new(values: Vec<i32>) -> Self {
            Self { values, idx: 0 }
        }
    }
    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            let v = if self.idx < self.values.len() {
                let v = self.values[self.idx];
                self.idx += 1;
                v
            } else {
                0
            };
            if bound <= 0 {
                0
            } else {
                v.rem_euclid(bound)
            }
        }
    }

    #[test]
    fn provider_type_is_weighted() {
        let list = WeightedListBuilder::new()
            .add(BlockState { id: 1 }, 1)
            .build();
        let provider = WeightedStateProvider::new(list);
        assert_eq!(
            provider.provider_type(),
            BlockStateProviderType::WeightedStateProvider
        );
    }

    #[test]
    fn get_state_selects_by_weight() {
        let list = WeightedListBuilder::new()
            .add(BlockState { id: 10 }, 1)
            .add(BlockState { id: 20 }, 3)
            .build();
        let provider = WeightedStateProvider::new(list);
        let pos = BlockPos { x: 0, y: 0, z: 0 };

        let mut r0 = FixedRandom(0); // selects first
        assert_eq!(provider.get_state(&mut r0, pos).id, 10);

        let mut r1 = FixedRandom(1); // selects second
        assert_eq!(provider.get_state(&mut r1, pos).id, 20);

        let mut r3 = FixedRandom(3); // selects second
        assert_eq!(provider.get_state(&mut r3, pos).id, 20);
    }

    #[test]
    fn create_rejects_empty_list() {
        let empty = WeightedList::<BlockState>::new(vec![]);
        let err = WeightedStateProvider::create(empty).unwrap_err();
        assert_eq!(err, "WeightedStateProvider with no states");
    }

    #[test]
    fn from_builder_and_negative_position() {
        let provider = WeightedStateProvider::from_builder(
            WeightedListBuilder::new().add(BlockState { id: -7 }, 5),
        );
        let mut rng = FixedRandom(0);
        let pos = BlockPos { x: -1, y: i32::MIN, z: i32::MAX };
        assert_eq!(provider.get_state(&mut rng, pos).id, -7);
    }

    #[test]
    #[should_panic(expected = "WeightedList is empty")]
    fn get_random_or_throw_panics_on_empty() {
        let list = WeightedList::<BlockState>::new(vec![]);
        let mut rng = FixedRandom(0);
        let _ = list.get_random_or_throw(&mut rng);
    }

    #[test]
    fn handles_zero_and_negative_weights_safely() {
        let list = WeightedListBuilder::new()
            .add(BlockState { id: 1 }, 0)
            .add(BlockState { id: 2 }, -5)
            .add(BlockState { id: 3 }, 2)
            .build();
        let provider = WeightedStateProvider::new(list);
        let mut rng = SeqRandom::new(vec![0, 1]);
        let pos = BlockPos { x: 0, y: 0, z: 0 };
        assert_eq!(provider.get_state(&mut rng, pos).id, 3);
        assert_eq!(provider.get_state(&mut rng, pos).id, 3);
    }
}
