use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

// ── Minimal self-contained stubs for unported dependencies ──────────────────

/// Minimal placeholder for `net.minecraft.resources.Identifier`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: &'static str,
    pub path: &'static str,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#withDefaultNamespace
    pub const fn with_default_namespace(path: &'static str) -> Self {
        Self {
            namespace: "minecraft",
            path,
        }
    }
}

/// Minimal placeholder for `net.minecraft.resources.ResourceKey<T>`.
pub struct ResourceKey<T: 'static> {
    registry: Identifier,
    location: Identifier,
    _marker: PhantomData<T>,
}

impl<T: 'static> ResourceKey<T> {
    // JAVA: net.minecraft.resources.ResourceKey#create
    pub const fn create(registry: Identifier, location: Identifier) -> Self {
        Self {
            registry,
            location,
            _marker: PhantomData,
        }
    }

    // JAVA: net.minecraft.resources.ResourceKey#identifier
    pub fn identifier(&self) -> &Identifier {
        &self.location
    }

    pub fn registry(&self) -> &Identifier {
        &self.registry
    }
}

impl<T: 'static> Clone for ResourceKey<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for ResourceKey<T> {}

impl<T: 'static> PartialEq for ResourceKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.registry == other.registry && self.location == other.location
    }
}

impl<T: 'static> Eq for ResourceKey<T> {}

impl<T: 'static> Hash for ResourceKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.registry.hash(state);
        self.location.hash(state);
    }
}

impl<T: 'static> std::fmt::Debug for ResourceKey<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceKey")
            .field("registry", &self.registry)
            .field("location", &self.location)
            .finish()
    }
}

/// Minimal placeholder for `net.minecraft.core.registries.Registries`.
pub struct Registries;

impl Registries {
    /// Registry identifier for normal noise parameters (`minecraft:worldgen/noise`).
    pub const NOISE: Identifier = Identifier {
        namespace: "minecraft",
        path: "worldgen/noise",
    };
}

/// Minimal placeholder for
/// `net.minecraft.world.level.levelgen.synth.NormalNoise.NoiseParameters`.
#[derive(Debug, Clone, PartialEq)]
pub struct NoiseParameters {
    pub first_octave: i32,
    pub amplitudes: Vec<f64>,
}

/// Minimal placeholder for `net.minecraft.util.RandomSource`.
pub trait RandomSource {}

/// Minimal placeholder for `net.minecraft.world.level.levelgen.PositionalRandomFactory`.
pub trait PositionalRandomFactory {
    type Random: RandomSource;

    // JAVA: net.minecraft.world.level.levelgen.PositionalRandomFactory#fromHashOf
    fn from_hash_of(&self, identifier: &Identifier) -> Self::Random;
}

/// Minimal placeholder for `net.minecraft.core.Holder<T>`.
pub trait Holder<T: 'static> {
    // JAVA: net.minecraft.core.Holder#unwrapKey
    fn unwrap_key(&self) -> Option<ResourceKey<T>>;

    // JAVA: net.minecraft.core.Holder#value
    fn value(&self) -> &T;
}

/// Minimal placeholder for `net.minecraft.core.HolderGetter<T>`.
pub trait HolderGetter<T: 'static> {
    type H: Holder<T>;

    // JAVA: net.minecraft.core.HolderGetter#getOrThrow
    fn get_or_throw(&self, key: &ResourceKey<T>) -> Self::H;
}

/// Minimal placeholder for `net.minecraft.world.level.levelgen.synth.NormalNoise`.
#[derive(Debug, Clone)]
pub struct NormalNoise;

impl NormalNoise {
    // JAVA: net.minecraft.world.level.levelgen.synth.NormalNoise#create
    pub fn create<R: RandomSource>(_random: R, _parameters: &NoiseParameters) -> Self {
        Self
    }
}

// ── Port of net.minecraft.world.level.levelgen.Noises ───────────────────────

/// Port of `net.minecraft.world.level.levelgen.Noises`.
pub struct Noises;

impl Noises {
    pub const TEMPERATURE: ResourceKey<NoiseParameters> = Self::create_key("temperature");
    pub const VEGETATION: ResourceKey<NoiseParameters> = Self::create_key("vegetation");
    pub const CONTINENTALNESS: ResourceKey<NoiseParameters> = Self::create_key("continentalness");
    pub const EROSION: ResourceKey<NoiseParameters> = Self::create_key("erosion");
    pub const TEMPERATURE_LARGE: ResourceKey<NoiseParameters> =
        Self::create_key("temperature_large");
    pub const VEGETATION_LARGE: ResourceKey<NoiseParameters> =
        Self::create_key("vegetation_large");
    pub const CONTINENTALNESS_LARGE: ResourceKey<NoiseParameters> =
        Self::create_key("continentalness_large");
    pub const EROSION_LARGE: ResourceKey<NoiseParameters> = Self::create_key("erosion_large");
    pub const RIDGE: ResourceKey<NoiseParameters> = Self::create_key("ridge");
    pub const SHIFT: ResourceKey<NoiseParameters> = Self::create_key("offset");
    pub const AQUIFER_BARRIER: ResourceKey<NoiseParameters> = Self::create_key("aquifer_barrier");
    pub const AQUIFER_FLUID_LEVEL_FLOODEDNESS: ResourceKey<NoiseParameters> =
        Self::create_key("aquifer_fluid_level_floodedness");
    pub const AQUIFER_LAVA: ResourceKey<NoiseParameters> = Self::create_key("aquifer_lava");
    pub const AQUIFER_FLUID_LEVEL_SPREAD: ResourceKey<NoiseParameters> =
        Self::create_key("aquifer_fluid_level_spread");
    pub const PILLAR: ResourceKey<NoiseParameters> = Self::create_key("pillar");
    pub const PILLAR_RARENESS: ResourceKey<NoiseParameters> = Self::create_key("pillar_rareness");
    pub const PILLAR_THICKNESS: ResourceKey<NoiseParameters> =
        Self::create_key("pillar_thickness");
    pub const SPAGHETTI_2D: ResourceKey<NoiseParameters> = Self::create_key("spaghetti_2d");
    pub const SPAGHETTI_2D_ELEVATION: ResourceKey<NoiseParameters> =
        Self::create_key("spaghetti_2d_elevation");
    pub const SPAGHETTI_2D_MODULATOR: ResourceKey<NoiseParameters> =
        Self::create_key("spaghetti_2d_modulator");
    pub const SPAGHETTI_2D_THICKNESS: ResourceKey<NoiseParameters> =
        Self::create_key("spaghetti_2d_thickness");
    pub const SPAGHETTI_3D_1: ResourceKey<NoiseParameters> = Self::create_key("spaghetti_3d_1");
    pub const SPAGHETTI_3D_2: ResourceKey<NoiseParameters> = Self::create_key("spaghetti_3d_2");
    pub const SPAGHETTI_3D_RARITY: ResourceKey<NoiseParameters> =
        Self::create_key("spaghetti_3d_rarity");
    pub const SPAGHETTI_3D_THICKNESS: ResourceKey<NoiseParameters> =
        Self::create_key("spaghetti_3d_thickness");
    pub const SPAGHETTI_ROUGHNESS: ResourceKey<NoiseParameters> =
        Self::create_key("spaghetti_roughness");
    pub const SPAGHETTI_ROUGHNESS_MODULATOR: ResourceKey<NoiseParameters> =
        Self::create_key("spaghetti_roughness_modulator");
    pub const CAVE_ENTRANCE: ResourceKey<NoiseParameters> = Self::create_key("cave_entrance");
    pub const CAVE_LAYER: ResourceKey<NoiseParameters> = Self::create_key("cave_layer");
    pub const CAVE_CHEESE: ResourceKey<NoiseParameters> = Self::create_key("cave_cheese");
    pub const ORE_VEININESS: ResourceKey<NoiseParameters> = Self::create_key("ore_veininess");
    pub const ORE_VEIN_A: ResourceKey<NoiseParameters> = Self::create_key("ore_vein_a");
    pub const ORE_VEIN_B: ResourceKey<NoiseParameters> = Self::create_key("ore_vein_b");
    pub const ORE_GAP: ResourceKey<NoiseParameters> = Self::create_key("ore_gap");
    pub const NOODLE: ResourceKey<NoiseParameters> = Self::create_key("noodle");
    pub const NOODLE_THICKNESS: ResourceKey<NoiseParameters> = Self::create_key("noodle_thickness");
    pub const NOODLE_RIDGE_A: ResourceKey<NoiseParameters> = Self::create_key("noodle_ridge_a");
    pub const NOODLE_RIDGE_B: ResourceKey<NoiseParameters> = Self::create_key("noodle_ridge_b");
    pub const JAGGED: ResourceKey<NoiseParameters> = Self::create_key("jagged");
    pub const SURFACE: ResourceKey<NoiseParameters> = Self::create_key("surface");
    pub const SURFACE_SECONDARY: ResourceKey<NoiseParameters> =
        Self::create_key("surface_secondary");
    pub const CLAY_BANDS_OFFSET: ResourceKey<NoiseParameters> =
        Self::create_key("clay_bands_offset");
    pub const BADLANDS_PILLAR: ResourceKey<NoiseParameters> = Self::create_key("badlands_pillar");
    pub const BADLANDS_PILLAR_ROOF: ResourceKey<NoiseParameters> =
        Self::create_key("badlands_pillar_roof");
    pub const BADLANDS_SURFACE: ResourceKey<NoiseParameters> =
        Self::create_key("badlands_surface");
    pub const ICEBERG_PILLAR: ResourceKey<NoiseParameters> = Self::create_key("iceberg_pillar");
    pub const ICEBERG_PILLAR_ROOF: ResourceKey<NoiseParameters> =
        Self::create_key("iceberg_pillar_roof");
    pub const ICEBERG_SURFACE: ResourceKey<NoiseParameters> = Self::create_key("iceberg_surface");
    pub const SWAMP: ResourceKey<NoiseParameters> = Self::create_key("surface_swamp");
    pub const CALCITE: ResourceKey<NoiseParameters> = Self::create_key("calcite");
    pub const GRAVEL: ResourceKey<NoiseParameters> = Self::create_key("gravel");
    pub const POWDER_SNOW: ResourceKey<NoiseParameters> = Self::create_key("powder_snow");
    pub const PACKED_ICE: ResourceKey<NoiseParameters> = Self::create_key("packed_ice");
    pub const ICE: ResourceKey<NoiseParameters> = Self::create_key("ice");
    pub const SOUL_SAND_LAYER: ResourceKey<NoiseParameters> = Self::create_key("soul_sand_layer");
    pub const GRAVEL_LAYER: ResourceKey<NoiseParameters> = Self::create_key("gravel_layer");
    pub const PATCH: ResourceKey<NoiseParameters> = Self::create_key("patch");
    pub const NETHERRACK: ResourceKey<NoiseParameters> = Self::create_key("netherrack");
    pub const NETHER_WART: ResourceKey<NoiseParameters> = Self::create_key("nether_wart");
    pub const NETHER_STATE_SELECTOR: ResourceKey<NoiseParameters> =
        Self::create_key("nether_state_selector");

    // JAVA: net.minecraft.world.level.levelgen.Noises#createKey
    const fn create_key(name: &'static str) -> ResourceKey<NoiseParameters> {
        ResourceKey::create(Registries::NOISE, Identifier::with_default_namespace(name))
    }

    // JAVA: net.minecraft.world.level.levelgen.Noises#instantiate
    pub fn instantiate<G, F>(
        getter: &G,
        random: &F,
        key: &ResourceKey<NoiseParameters>,
    ) -> NormalNoise
    where
        G: HolderGetter<NoiseParameters>,
        F: PositionalRandomFactory,
    {
        let holder = getter.get_or_throw(key);
        let unwrapped_key = holder.unwrap_key().expect("missing noise key");
        NormalNoise::create(
            random.from_hash_of(unwrapped_key.identifier()),
            holder.value(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    struct MockRandom;
    impl RandomSource for MockRandom {}

    struct MockFactory {
        expected_id: Identifier,
    }

    impl PositionalRandomFactory for MockFactory {
        type Random = MockRandom;

        fn from_hash_of(&self, identifier: &Identifier) -> Self::Random {
            assert_eq!(*identifier, self.expected_id);
            MockRandom
        }
    }

    #[derive(Clone)]
    struct MockHolder {
        key: ResourceKey<NoiseParameters>,
        params: NoiseParameters,
    }

    impl Holder<NoiseParameters> for MockHolder {
        fn unwrap_key(&self) -> Option<ResourceKey<NoiseParameters>> {
            Some(self.key)
        }

        fn value(&self) -> &NoiseParameters {
            &self.params
        }
    }

    struct MockGetter {
        holder: MockHolder,
    }

    impl HolderGetter<NoiseParameters> for MockGetter {
        type H = MockHolder;

        fn get_or_throw(&self, _key: &ResourceKey<NoiseParameters>) -> Self::H {
            self.holder.clone()
        }
    }

    #[test]
    fn key_paths_and_namespace_match_java() {
        assert_eq!(Noises::TEMPERATURE.identifier().namespace, "minecraft");
        assert_eq!(Noises::TEMPERATURE.identifier().path, "temperature");
        assert_eq!(Noises::SHIFT.identifier().path, "offset");
        assert_eq!(Noises::SWAMP.identifier().path, "surface_swamp");
        assert_eq!(
            Noises::NETHER_STATE_SELECTOR.identifier().path,
            "nether_state_selector"
        );
        assert_eq!(Noises::EROSION.registry(), &Registries::NOISE);
    }

    #[test]
    fn resource_key_equality_hash_and_copy() {
        let a = Noises::TEMPERATURE;
        let b = a; // Copy
        let c = Noises::VEGETATION;
        assert_eq!(a, b);
        assert_ne!(a, c);

        let mut set = HashSet::new();
        set.insert(a);
        set.insert(b);
        set.insert(c);
        assert_eq!(set.len(), 2);

        let dbg = format!("{:?}", a);
        assert!(dbg.contains("ResourceKey"));
        assert!(dbg.contains("temperature"));
    }

    #[test]
    fn instantiate_uses_holder_key_and_factory() {
        let key = Noises::CAVE_CHEESE;
        let params = NoiseParameters {
            first_octave: -7,
            amplitudes: vec![1.0, 2.0],
        };
        let getter = MockGetter {
            holder: MockHolder {
                key,
                params: params.clone(),
            },
        };
        let factory = MockFactory {
            expected_id: *key.identifier(),
        };
        let _noise = Noises::instantiate(&getter, &factory, &key);
    }

    #[test]
    fn instantiate_panics_when_holder_has_no_key() {
        struct NoKeyHolder {
            params: NoiseParameters,
        }
        impl Holder<NoiseParameters> for NoKeyHolder {
            fn unwrap_key(&self) -> Option<ResourceKey<NoiseParameters>> {
                None
            }
            fn value(&self) -> &NoiseParameters {
                &self.params
            }
        }
        struct NoKeyGetter(NoKeyHolder);
        impl HolderGetter<NoiseParameters> for NoKeyGetter {
            type H = NoKeyHolder;
            fn get_or_throw(&self, _key: &ResourceKey<NoiseParameters>) -> Self::H {
                NoKeyHolder {
                    params: NoiseParameters {
                        first_octave: 0,
                        amplitudes: vec![],
                    },
                }
            }
        }
        struct DummyFactory;
        impl PositionalRandomFactory for DummyFactory {
            type Random = MockRandom;
            fn from_hash_of(&self, _identifier: &Identifier) -> Self::Random {
                MockRandom
            }
        }

        let result = std::panic::catch_unwind(|| {
            let getter = NoKeyGetter(NoKeyHolder {
                params: NoiseParameters {
                    first_octave: 0,
                    amplitudes: vec![],
                },
            });
            let factory = DummyFactory;
            let _ = Noises::instantiate(&getter, &factory, &Noises::TEMPERATURE);
        });
        assert!(result.is_err());
    }

    #[test]
    fn noise_parameters_negative_overflow_edge_and_float_bits() {
        let p = NoiseParameters {
            first_octave: i32::MIN,
            amplitudes: vec![-0.0, -1.5, f64::INFINITY, f64::NEG_INFINITY, f64::NAN],
        };
        assert_eq!(p.first_octave, i32::MIN);
        assert_eq!(p.first_octave.wrapping_sub(1), i32::MAX);
        assert_eq!(p.amplitudes[0].to_bits(), (-0.0f64).to_bits());
        assert_ne!(p.amplitudes[0].to_bits(), 0.0f64.to_bits());
        assert_eq!(p.amplitudes[1].to_bits(), (-1.5f64).to_bits());
        assert_eq!(p.amplitudes[2].to_bits(), f64::INFINITY.to_bits());
        assert_eq!(p.amplitudes[3].to_bits(), f64::NEG_INFINITY.to_bits());
        assert!(p.amplitudes[4].is_nan());
    }
}
