#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorldOptions {
    pub seed: i64,
    pub generate_features: bool,
    pub generate_bonus_chest: bool,
}

impl WorldOptions {
    pub fn new(seed: i64, generate_features: bool, generate_bonus_chest: bool) -> Self {
        Self {
            seed,
            generate_features,
            generate_bonus_chest,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorldDimensions {
    pub stem_count: u32,
}

impl WorldDimensions {
    pub fn new(stem_count: u32) -> Self {
        Self { stem_count }
    }

    fn from_registry_access(registry_access: &RegistryAccess) -> Self {
        // Mirrors: new WorldDimensions($$2.lookupOrThrow(Registries.LEVEL_STEM))
        Self {
            stem_count: registry_access.level_stem_count(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RegistryAccess {
    level_stems: u32,
}

impl RegistryAccess {
    pub fn new(level_stems: u32) -> Self {
        Self {
            level_stems,
        }
    }

    pub fn level_stem_count(&self) -> u32 {
        self.level_stems
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldGenSettings {
    options: WorldOptions,
    dimensions: WorldDimensions,
}

impl WorldGenSettings {
    // JAVA: net.minecraft.world.level.levelgen.WorldGenSettings#<init>
    pub fn new(options: WorldOptions, dimensions: WorldDimensions) -> Self {
        Self { options, dimensions }
    }

    // JAVA: net.minecraft.world.level.levelgen.WorldGenSettings#options
    pub fn options(&self) -> &WorldOptions {
        &self.options
    }

    // JAVA: net.minecraft.world.level.levelgen.WorldGenSettings#dimensions
    pub fn dimensions(&self) -> &WorldDimensions {
        &self.dimensions
    }

    // JAVA: net.minecraft.world.level.levelgen.WorldGenSettings#encode
    pub fn encode<T, Ops>(
        _ops: &Ops,
        options: WorldOptions,
        dimensions: WorldDimensions,
    ) -> Result<T, String> {
        let _settings = Self::new(options, dimensions);
        Err("CODEC.encodeStart is not available in this self-contained port".to_string())
    }

    // JAVA: net.minecraft.world.level.levelgen.WorldGenSettings#encode
    pub fn encode_with_registry_access<T, Ops>(
        ops: &Ops,
        options: WorldOptions,
        registry_access: &RegistryAccess,
    ) -> Result<T, String> {
        Self::encode(
            ops,
            options,
            WorldDimensions::from_registry_access(registry_access),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_accessors_preserve_fields() {
        let options = WorldOptions::new(12345, true, false);
        let dimensions = WorldDimensions::new(3);
        let settings = WorldGenSettings::new(options.clone(), dimensions.clone());
        assert_eq!(settings.options(), &options);
        assert_eq!(settings.dimensions(), &dimensions);
    }

    #[test]
    fn encode_returns_codec_unavailable_error() {
        let options = WorldOptions::new(0, false, false);
        let dimensions = WorldDimensions::new(1);
        let res: Result<(), String> = WorldGenSettings::encode(&(), options, dimensions);
        assert!(res.is_err());
    }

    #[test]
    fn encode_with_registry_access_delegates_to_encode() {
        let options = WorldOptions::new(-1, true, true);
        let registry = RegistryAccess::new(7);
        let res: Result<(), String> =
            WorldGenSettings::encode_with_registry_access(&(), options, &registry);
        assert_eq!(
            res,
            Err("CODEC.encodeStart is not available in this self-contained port".to_string())
        );
    }

    #[test]
    fn world_dimensions_from_registry_access_uses_level_stem_count() {
        let registry = RegistryAccess::new(42);
        let dims = WorldDimensions::from_registry_access(&registry);
        assert_eq!(dims, WorldDimensions::new(42));
    }

    #[test]
    fn options_preserve_negative_seed() {
        let options = WorldOptions::new(i64::MIN, false, true);
        let settings = WorldGenSettings::new(options.clone(), WorldDimensions::new(0));
        assert_eq!(settings.options().seed, i64::MIN);
        assert!(!settings.options().generate_features);
        assert!(settings.options().generate_bonus_chest);
    }
}
