use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockState {
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Biome {
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T: Clone> {
    pub value: T,
}

impl<T: Clone + Default> Default for Holder<T> {
    fn default() -> Self {
        Self {
            value: T::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Strategy<T> {
    _phantom: PhantomData<T>,
}

impl<T> Strategy<T> {
    // JAVA: net.minecraft.world.level.chunk.Strategy#createForBlockStates(IdMap)
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for Strategy<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Codec<T> {
    _phantom: PhantomData<T>,
}

impl<T> Codec<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for Codec<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PalettedContainer<T: Clone> {
    pub default_value: T,
    pub strategy: Strategy<T>,
}

impl<T: Clone> PalettedContainer<T> {
    pub fn new(default_value: T, strategy: Strategy<T>) -> Self {
        Self {
            default_value,
            strategy,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PalettedContainerRO<T: Clone> {
    pub default_value: T,
    pub strategy: Strategy<T>,
}

#[derive(Debug, Clone, Default)]
pub struct RegistryAccess;

#[derive(Debug, Clone)]
pub struct PalettedContainerFactory {
    pub block_states_strategy: Strategy<BlockState>,
    pub default_block_state: BlockState,
    pub block_states_container_codec: Codec<PalettedContainer<BlockState>>,
    pub biome_strategy: Strategy<Holder<Biome>>,
    pub default_biome: Holder<Biome>,
    pub biome_container_codec: Codec<PalettedContainerRO<Holder<Biome>>>,
}

impl PalettedContainerFactory {
    // JAVA: net.minecraft.world.level.chunk.PalettedContainerFactory#create(RegistryAccess)
    pub fn create(_registry_access: &RegistryAccess) -> Self {
        // Self-contained stub: real implementation wires registry/strategy/codec infrastructure.
        Self {
            block_states_strategy: Strategy::new(),
            default_block_state: BlockState::default(),
            block_states_container_codec: Codec::new(),
            biome_strategy: Strategy::new(),
            default_biome: Holder::default(),
            biome_container_codec: Codec::new(),
        }
    }

    // JAVA: net.minecraft.world.level.chunk.PalettedContainerFactory#createForBlockStates()
    pub fn create_for_block_states(&self) -> PalettedContainer<BlockState> {
        PalettedContainer::new(
            self.default_block_state.clone(),
            self.block_states_strategy.clone(),
        )
    }

    // JAVA: net.minecraft.world.level.chunk.PalettedContainerFactory#createForBiomes()
    pub fn create_for_biomes(&self) -> PalettedContainer<Holder<Biome>> {
        PalettedContainer::new(self.default_biome.clone(), self.biome_strategy.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_default_factory() {
        let f = PalettedContainerFactory::create(&RegistryAccess::default());
        assert_eq!(f.default_block_state, BlockState { id: 0 });
        assert_eq!(f.default_biome, Holder { value: Biome { id: 0 } });
    }

    #[test]
    fn test_create_for_block_states_uses_default() {
        let f = PalettedContainerFactory {
            block_states_strategy: Strategy::new(),
            default_block_state: BlockState { id: 42 },
            block_states_container_codec: Codec::new(),
            biome_strategy: Strategy::new(),
            default_biome: Holder { value: Biome { id: 1 } },
            biome_container_codec: Codec::new(),
        };
        let container = f.create_for_block_states();
        assert_eq!(container.default_value, BlockState { id: 42 });
    }

    #[test]
    fn test_create_for_biomes_uses_default() {
        let f = PalettedContainerFactory {
            block_states_strategy: Strategy::new(),
            default_block_state: BlockState { id: 0 },
            block_states_container_codec: Codec::new(),
            biome_strategy: Strategy::new(),
            default_biome: Holder { value: Biome { id: 7 } },
            biome_container_codec: Codec::new(),
        };
        let container = f.create_for_biomes();
        assert_eq!(container.default_value, Holder { value: Biome { id: 7 } });
    }

    #[test]
    fn test_negative_and_large_ids() {
        let bs = BlockState { id: u32::MAX };
        let biome = Biome { id: u32::MAX - 1 };
        let f = PalettedContainerFactory {
            block_states_strategy: Strategy::new(),
            default_block_state: bs.clone(),
            block_states_container_codec: Codec::new(),
            biome_strategy: Strategy::new(),
            default_biome: Holder { value: biome.clone() },
            biome_container_codec: Codec::new(),
        };
        assert_eq!(f.create_for_block_states().default_value.id, u32::MAX);
        assert_eq!(f.create_for_biomes().default_value.value.id, u32::MAX - 1);
    }

    #[test]
    fn test_clone_factory() {
        let f = PalettedContainerFactory::create(&RegistryAccess::default());
        let f2 = f.clone();
        assert_eq!(f2.default_block_state, f.default_block_state);
        assert_eq!(f2.default_biome, f.default_biome);
    }
}
