/// Minimal self-contained placeholder for `com.mojang.serialization.MapCodec`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapCodec {
    pub id: String,
}

/// Minimal self-contained placeholder for `net.minecraft.world.level.chunk.ChunkGenerator`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChunkGenerator;

/// Minimal self-contained placeholder for `net.minecraft.core.Registry`.
#[derive(Debug, Clone, Default)]
pub struct Registry<T> {
    pub entries: Vec<(String, T)>,
}

impl<T: Clone> Registry<T> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    // JAVA: net.minecraft.core.Registry#register
    pub fn register(registry: &mut Registry<T>, name: &str, value: T) -> T {
        registry.entries.push((name.to_owned(), value.clone()));
        value
    }
}

/// Placeholder for `net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator`.
pub struct NoiseBasedChunkGenerator;

impl NoiseBasedChunkGenerator {
    pub fn codec() -> MapCodec {
        MapCodec {
            id: "noise".to_owned(),
        }
    }
}

/// Placeholder for `net.minecraft.world.level.levelgen.FlatLevelSource`.
pub struct FlatLevelSource;

impl FlatLevelSource {
    pub fn codec() -> MapCodec {
        MapCodec {
            id: "flat".to_owned(),
        }
    }
}

/// Placeholder for `net.minecraft.world.level.levelgen.DebugLevelSource`.
pub struct DebugLevelSource;

impl DebugLevelSource {
    pub fn codec() -> MapCodec {
        MapCodec {
            id: "debug".to_owned(),
        }
    }
}

/// Port of `net.minecraft.world.level.chunk.ChunkGenerators`.
pub struct ChunkGenerators;

impl ChunkGenerators {
    // JAVA: net.minecraft.world.level.chunk.ChunkGenerators#bootstrap
    pub fn bootstrap(registry: &mut Registry<MapCodec>) -> MapCodec {
        Registry::<MapCodec>::register(registry, "noise", NoiseBasedChunkGenerator::codec());
        Registry::<MapCodec>::register(registry, "flat", FlatLevelSource::codec());
        Registry::<MapCodec>::register(registry, "debug", DebugLevelSource::codec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_registers_three_generators_in_order() {
        let mut registry = Registry::<MapCodec>::new();
        let last = ChunkGenerators::bootstrap(&mut registry);
        assert_eq!(registry.entries.len(), 3);
        assert_eq!(registry.entries[0].0, "noise");
        assert_eq!(registry.entries[1].0, "flat");
        assert_eq!(registry.entries[2].0, "debug");
        assert_eq!(
            last,
            MapCodec {
                id: "debug".to_owned()
            }
        );
    }

    #[test]
    fn bootstrap_returns_debug_codec() {
        let mut registry = Registry::<MapCodec>::new();
        let returned = ChunkGenerators::bootstrap(&mut registry);
        assert_eq!(returned.id, "debug");
        assert_eq!(returned, DebugLevelSource::codec());
    }

    #[test]
    fn registry_register_returns_value_and_appends() {
        let mut registry = Registry::<MapCodec>::new();
        let v = Registry::register(
            &mut registry,
            "noise",
            NoiseBasedChunkGenerator::codec(),
        );
        assert_eq!(v.id, "noise");
        assert_eq!(registry.entries.len(), 1);
        assert_eq!(registry.entries[0].0, "noise");
    }

    #[test]
    fn bootstrap_on_nonempty_registry_appends() {
        let mut registry = Registry::<MapCodec>::new();
        Registry::register(
            &mut registry,
            "custom",
            MapCodec {
                id: "custom".to_owned(),
            },
        );
        ChunkGenerators::bootstrap(&mut registry);
        assert_eq!(registry.entries.len(), 4);
        assert_eq!(registry.entries[0].0, "custom");
        assert_eq!(registry.entries[3].0, "debug");
    }
}
