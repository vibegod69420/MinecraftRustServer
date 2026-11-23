// === FILE: random_state.rs ===
//! Port of `net.minecraft.world.level.levelgen.RandomState`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// Minimal self-contained stubs for unported Minecraft types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_owned(),
            path: path.to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    pub location: Identifier,
}

impl ResourceKey {
    pub fn new(location: Identifier) -> Self {
        Self { location }
    }

    pub fn identifier(&self) -> &Identifier {
        &self.location
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Holder<T> {
    key: Option<ResourceKey>,
    value: Arc<T>,
}

impl<T> Holder<T> {
    pub fn new(key: Option<ResourceKey>, value: Arc<T>) -> Self {
        Self { key, value }
    }

    pub fn is(&self, key: &ResourceKey) -> bool {
        self.key.as_ref() == Some(key)
    }

    pub fn unwrap_key(&self) -> Option<&ResourceKey> {
        self.key.as_ref()
    }

    pub fn value(&self) -> &Arc<T> {
        &self.value
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NoiseParameters {
    pub first_octave: i32,
    pub amplitudes: Vec<u64>, // f64::to_bits() for stable Hash/Eq and float-bit fidelity
}

impl NoiseParameters {
    pub fn new(first_octave: i32, amplitudes: Vec<f64>) -> Self {
        Self {
            first_octave,
            amplitudes: amplitudes.into_iter().map(f64::to_bits).collect(),
        }
    }
}

pub trait RandomSource: Send + Sync + std::fmt::Debug {
    fn fork_positional(&self) -> Arc<dyn PositionalRandomFactory>;
    fn seed_tag(&self) -> i64 {
        0
    }
}

pub trait PositionalRandomFactory: Send + Sync + std::fmt::Debug {
    fn from_hash_of(&self, id: &Identifier) -> Arc<dyn RandomSource>;
    fn fork_positional(&self) -> Arc<dyn PositionalRandomFactory>;
}

#[derive(Clone, Debug)]
pub struct LegacyRandomSource {
    pub seed: i64,
}

impl LegacyRandomSource {
    pub fn new(seed: i64) -> Self {
        Self { seed }
    }
}

impl RandomSource for LegacyRandomSource {
    fn fork_positional(&self) -> Arc<dyn PositionalRandomFactory> {
        Arc::new(SimplePositionalRandomFactory { seed: self.seed })
    }

    fn seed_tag(&self) -> i64 {
        self.seed
    }
}

#[derive(Clone, Debug)]
pub struct SimpleRandomSource {
    pub seed: i64,
}

impl RandomSource for SimpleRandomSource {
    fn fork_positional(&self) -> Arc<dyn PositionalRandomFactory> {
        Arc::new(SimplePositionalRandomFactory { seed: self.seed })
    }

    fn seed_tag(&self) -> i64 {
        self.seed
    }
}

#[derive(Clone, Debug)]
pub struct SimplePositionalRandomFactory {
    pub seed: i64,
}

impl PositionalRandomFactory for SimplePositionalRandomFactory {
    fn from_hash_of(&self, id: &Identifier) -> Arc<dyn RandomSource> {
        let mut h: i64 = self.seed;
        for b in id.namespace.bytes() {
            h = h.wrapping_mul(31).wrapping_add(b as i64);
        }
        h = h.wrapping_mul(31).wrapping_add(b':' as i64);
        for b in id.path.bytes() {
            h = h.wrapping_mul(31).wrapping_add(b as i64);
        }
        Arc::new(LegacyRandomSource::new(h))
    }

    fn fork_positional(&self) -> Arc<dyn PositionalRandomFactory> {
        Arc::new(SimplePositionalRandomFactory {
            seed: self.seed.wrapping_add(1),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NormalNoise {
    pub parameters: NoiseParameters,
    pub legacy_nether_biome: bool,
    pub random_seed: i64,
}

impl NormalNoise {
    pub fn create(random: &dyn RandomSource, parameters: NoiseParameters) -> Self {
        Self {
            parameters,
            legacy_nether_biome: false,
            random_seed: random.seed_tag(),
        }
    }

    pub fn create_legacy_nether_biome(
        random: &dyn RandomSource,
        parameters: NoiseParameters,
    ) -> Self {
        Self {
            parameters,
            legacy_nether_biome: true,
            random_seed: random.seed_tag(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NoiseHolder {
    pub noise_data: Holder<NoiseParameters>,
    pub noise: NormalNoise,
}

impl NoiseHolder {
    pub fn new(noise_data: Holder<NoiseParameters>, noise: NormalNoise) -> Self {
        Self { noise_data, noise }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlendedNoise {
    pub random_seed: i64,
}

impl BlendedNoise {
    pub fn with_new_random(&self, random: &dyn RandomSource) -> DensityFunction {
        DensityFunction::BlendedNoise(BlendedNoise {
            random_seed: random.seed_tag(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DensityFunction {
    NoiseHolder(NoiseHolder),
    BlendedNoise(BlendedNoise),
    EndIslandDensityFunction(i64),
    HolderHolder(Holder<DensityFunction>),
    Marker(Box<DensityFunction>),
    Constant(u64), // f64 bits
}

pub trait DensityFunctionVisitor {
    fn visit_noise(&mut self, noise: NoiseHolder) -> NoiseHolder {
        noise
    }

    fn apply(&mut self, f: DensityFunction) -> DensityFunction {
        f
    }
}

impl DensityFunction {
    pub fn map_all<V: DensityFunctionVisitor>(&self, visitor: &mut V) -> DensityFunction {
        match self {
            DensityFunction::NoiseHolder(nh) => {
                DensityFunction::NoiseHolder(visitor.visit_noise(nh.clone()))
            }
            DensityFunction::BlendedNoise(_)
            | DensityFunction::EndIslandDensityFunction(_)
            | DensityFunction::Constant(_) => visitor.apply(self.clone()),
            DensityFunction::HolderHolder(h) => {
                let mapped_inner = h.value().map_all(visitor);
                let new_h = Holder::new(h.key.clone(), Arc::new(mapped_inner));
                visitor.apply(DensityFunction::HolderHolder(new_h))
            }
            DensityFunction::Marker(inner) => {
                let mapped = inner.map_all(visitor);
                visitor.apply(DensityFunction::Marker(Box::new(mapped)))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NoiseRouter {
    pub barrier_noise: DensityFunction,
    pub fluid_level_floodedness_noise: DensityFunction,
    pub fluid_level_spread_noise: DensityFunction,
    pub lava_noise: DensityFunction,
    pub temperature: DensityFunction,
    pub vegetation: DensityFunction,
    pub continents: DensityFunction,
    pub erosion: DensityFunction,
    pub depth: DensityFunction,
    pub ridges: DensityFunction,
    pub initial_density_without_jaggedness: DensityFunction,
    pub final_density: DensityFunction,
    pub vein_toggle: DensityFunction,
    pub vein_ridged: DensityFunction,
    pub vein_gap: DensityFunction,
}

impl NoiseRouter {
    pub fn map_all<V: DensityFunctionVisitor>(&self, visitor: &mut V) -> Self {
        Self {
            barrier_noise: self.barrier_noise.map_all(visitor),
            fluid_level_floodedness_noise: self.fluid_level_floodedness_noise.map_all(visitor),
            fluid_level_spread_noise: self.fluid_level_spread_noise.map_all(visitor),
            lava_noise: self.lava_noise.map_all(visitor),
            temperature: self.temperature.map_all(visitor),
            vegetation: self.vegetation.map_all(visitor),
            continents: self.continents.map_all(visitor),
            erosion: self.erosion.map_all(visitor),
            depth: self.depth.map_all(visitor),
            ridges: self.ridges.map_all(visitor),
            initial_density_without_jaggedness: self
                .initial_density_without_jaggedness
                .map_all(visitor),
            final_density: self.final_density.map_all(visitor),
            vein_toggle: self.vein_toggle.map_all(visitor),
            vein_ridged: self.vein_ridged.map_all(visitor),
            vein_gap: self.vein_gap.map_all(visitor),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpawnTarget;

#[derive(Clone, Debug)]
pub struct ClimateSampler {
    pub temperature: DensityFunction,
    pub vegetation: DensityFunction,
    pub continents: DensityFunction,
    pub erosion: DensityFunction,
    pub depth: DensityFunction,
    pub ridges: DensityFunction,
    pub spawn_target: Vec<SpawnTarget>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct SurfaceSystem {
    pub default_block: BlockState,
    pub sea_level: i32,
}

pub trait WorldgenRandomAlgorithm: Send + Sync + std::fmt::Debug {
    fn new_instance(&self, seed: i64) -> Arc<dyn RandomSource>;
}

#[derive(Clone, Debug)]
pub struct XoroshiroAlgorithm;

impl WorldgenRandomAlgorithm for XoroshiroAlgorithm {
    fn new_instance(&self, seed: i64) -> Arc<dyn RandomSource> {
        Arc::new(SimpleRandomSource { seed })
    }
}

pub trait NoiseParametersGetter: Send + Sync {
    fn get_or_throw(&self, key: &ResourceKey) -> NoiseParameters;
}

pub trait RegistriesProvider: Send + Sync {
    fn noise_generator_settings(&self, key: &ResourceKey) -> NoiseGeneratorSettings;
    fn noise_parameters(&self) -> Arc<dyn NoiseParametersGetter>;
}

pub struct NoiseGeneratorSettings {
    pub random_source_algorithm: Arc<dyn WorldgenRandomAlgorithm>,
    pub noise_router: NoiseRouter,
    pub default_block: BlockState,
    pub sea_level: i32,
    pub use_legacy_random_source: bool,
    pub spawn_target: Vec<SpawnTarget>,
}

pub struct Noises;

impl Noises {
    pub fn temperature() -> ResourceKey {
        ResourceKey::new(Identifier::with_default_namespace("temperature"))
    }

    pub fn vegetation() -> ResourceKey {
        ResourceKey::new(Identifier::with_default_namespace("vegetation"))
    }

    pub fn shift() -> ResourceKey {
        ResourceKey::new(Identifier::with_default_namespace("offset"))
    }

    pub fn instantiate(
        noises: &dyn NoiseParametersGetter,
        random: &dyn PositionalRandomFactory,
        key: &ResourceKey,
    ) -> NormalNoise {
        let params = noises.get_or_throw(key);
        NormalNoise::create(random.from_hash_of(key.identifier()).as_ref(), params)
    }
}

// ---------------------------------------------------------------------------
// RandomState port
// ---------------------------------------------------------------------------

pub struct RandomState {
    random: Arc<dyn PositionalRandomFactory>,
    noises: Arc<dyn NoiseParametersGetter>,
    router: NoiseRouter,
    sampler: ClimateSampler,
    surface_system: SurfaceSystem,
    aquifer_random: Arc<dyn PositionalRandomFactory>,
    ore_random: Arc<dyn PositionalRandomFactory>,
    noise_instances: Mutex<HashMap<ResourceKey, NormalNoise>>,
    positional_randoms: Mutex<HashMap<Identifier, Arc<dyn PositionalRandomFactory>>>,
}

impl RandomState {
    // JAVA: net.minecraft.world.level.levelgen.RandomState#create
    pub fn create(provider: &dyn RegistriesProvider, settings_key: &ResourceKey, seed: i64) -> Self {
        let settings = provider.noise_generator_settings(settings_key);
        let noises = provider.noise_parameters();
        Self::create_from_settings(settings, noises, seed)
    }

    // JAVA: net.minecraft.world.level.levelgen.RandomState#create
    pub fn create_from_settings(
        settings: NoiseGeneratorSettings,
        noises: Arc<dyn NoiseParametersGetter>,
        seed: i64,
    ) -> Self {
        Self::new(settings, noises, seed)
    }

    // JAVA: net.minecraft.world.level.levelgen.RandomState#<init>
    fn new(
        settings: NoiseGeneratorSettings,
        noises: Arc<dyn NoiseParametersGetter>,
        seed: i64,
    ) -> Self {
        let random = settings
            .random_source_algorithm
            .new_instance(seed)
            .fork_positional();
        let aquifer_random = random
            .from_hash_of(&Identifier::with_default_namespace("aquifer"))
            .fork_positional();
        let ore_random = random
            .from_hash_of(&Identifier::with_default_namespace("ore"))
            .fork_positional();
        let noise_instances: Mutex<HashMap<ResourceKey, NormalNoise>> = Mutex::new(HashMap::new());
        let positional_randoms: Mutex<HashMap<Identifier, Arc<dyn PositionalRandomFactory>>> =
            Mutex::new(HashMap::new());
        let surface_system = SurfaceSystem {
            default_block: settings.default_block.clone(),
            sea_level: settings.sea_level,
        };
        let use_legacy = settings.use_legacy_random_source;

        let router = {
            let mut wiring = NoiseWiringHelper {
                seed,
                use_legacy,
                random: random.as_ref(),
                noises: noises.as_ref(),
                noise_instances: &noise_instances,
                wrapped: HashMap::new(),
            };
            settings.noise_router.map_all(&mut wiring)
        };

        let sampler = {
            let mut unwrapping = UnwrappingVisitor {
                wrapped: HashMap::new(),
            };
            ClimateSampler {
                temperature: router.temperature.map_all(&mut unwrapping),
                vegetation: router.vegetation.map_all(&mut unwrapping),
                continents: router.continents.map_all(&mut unwrapping),
                erosion: router.erosion.map_all(&mut unwrapping),
                depth: router.depth.map_all(&mut unwrapping),
                ridges: router.ridges.map_all(&mut unwrapping),
                spawn_target: settings.spawn_target.clone(),
            }
        };

        Self {
            random,
            noises,
            router,
            sampler,
            surface_system,
            aquifer_random,
            ore_random,
            noise_instances,
            positional_randoms,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.RandomState#getOrCreateNoise
    pub fn get_or_create_noise(&self, key: &ResourceKey) -> NormalNoise {
        let mut map = self
            .noise_instances
            .lock()
            .expect("noiseInstances mutex poisoned");
        if let Some(n) = map.get(key) {
            return n.clone();
        }
        let n = Noises::instantiate(self.noises.as_ref(), self.random.as_ref(), key);
        map.insert(key.clone(), n.clone());
        n
    }

    // JAVA: net.minecraft.world.level.levelgen.RandomState#getOrCreateRandomFactory
    pub fn get_or_create_random_factory(
        &self,
        id: &Identifier,
    ) -> Arc<dyn PositionalRandomFactory> {
        let mut map = self
            .positional_randoms
            .lock()
            .expect("positionalRandoms mutex poisoned");
        if let Some(f) = map.get(id) {
            return f.clone();
        }
        let f = self.random.from_hash_of(id).fork_positional();
        map.insert(id.clone(), f.clone());
        f
    }

    // JAVA: net.minecraft.world.level.levelgen.RandomState#router
    pub fn router(&self) -> &NoiseRouter {
        &self.router
    }

    // JAVA: net.minecraft.world.level.levelgen.RandomState#sampler
    pub fn sampler(&self) -> &ClimateSampler {
        &self.sampler
    }

    // JAVA: net.minecraft.world.level.levelgen.RandomState#surfaceSystem
    pub fn surface_system(&self) -> &SurfaceSystem {
        &self.surface_system
    }

    // JAVA: net.minecraft.world.level.levelgen.RandomState#aquiferRandom
    pub fn aquifer_random(&self) -> &Arc<dyn PositionalRandomFactory> {
        &self.aquifer_random
    }

    // JAVA: net.minecraft.world.level.levelgen.RandomState#oreRandom
    pub fn ore_random(&self) -> &Arc<dyn PositionalRandomFactory> {
        &self.ore_random
    }
}

struct NoiseWiringHelper<'a> {
    seed: i64,
    use_legacy: bool,
    random: &'a dyn PositionalRandomFactory,
    noises: &'a dyn NoiseParametersGetter,
    noise_instances: &'a Mutex<HashMap<ResourceKey, NormalNoise>>,
    wrapped: HashMap<DensityFunction, DensityFunction>,
}

impl<'a> NoiseWiringHelper<'a> {
    // JAVA: net.minecraft.world.level.levelgen.RandomState#newLegacyInstance
    fn new_legacy_instance(&self, offset: i64) -> Arc<dyn RandomSource> {
        Arc::new(LegacyRandomSource::new(self.seed.wrapping_add(offset)))
    }

    fn get_or_create_noise(&self, key: &ResourceKey) -> NormalNoise {
        let mut map = self
            .noise_instances
            .lock()
            .expect("noiseInstances mutex poisoned");
        if let Some(n) = map.get(key) {
            return n.clone();
        }
        let n = Noises::instantiate(self.noises, self.random, key);
        map.insert(key.clone(), n.clone());
        n
    }

    // JAVA: net.minecraft.world.level.levelgen.RandomState#wrapNew
    fn wrap_new(&mut self, f: DensityFunction) -> DensityFunction {
        match &f {
            DensityFunction::BlendedNoise(bn) => {
                let random: Arc<dyn RandomSource> = if self.use_legacy {
                    self.new_legacy_instance(0)
                } else {
                    self.random
                        .from_hash_of(&Identifier::with_default_namespace("terrain"))
                };
                bn.with_new_random(random.as_ref())
            }
            DensityFunction::EndIslandDensityFunction(_) => {
                DensityFunction::EndIslandDensityFunction(self.seed)
            }
            _ => f,
        }
    }
}

impl<'a> DensityFunctionVisitor for NoiseWiringHelper<'a> {
    // JAVA: net.minecraft.world.level.levelgen.RandomState#visitNoise
    fn visit_noise(&mut self, noise_holder: NoiseHolder) -> NoiseHolder {
        let holder = noise_holder.noise_data.clone();
        if self.use_legacy {
            if holder.is(&Noises::temperature()) {
                let noise = NormalNoise::create_legacy_nether_biome(
                    self.new_legacy_instance(0).as_ref(),
                    NoiseParameters::new(-7, vec![1.0, 1.0]),
                );
                return NoiseHolder::new(holder, noise);
            }

            if holder.is(&Noises::vegetation()) {
                let noise = NormalNoise::create_legacy_nether_biome(
                    self.new_legacy_instance(1).as_ref(),
                    NoiseParameters::new(-7, vec![1.0, 1.0]),
                );
                return NoiseHolder::new(holder, noise);
            }

            if holder.is(&Noises::shift()) {
                let noise = NormalNoise::create(
                    self.random
                        .from_hash_of(Noises::shift().identifier())
                        .as_ref(),
                    NoiseParameters::new(0, vec![0.0]),
                );
                return NoiseHolder::new(holder, noise);
            }
        }

        let key = holder
            .unwrap_key()
            .expect("noise holder missing key")
            .clone();
        let noise = self.get_or_create_noise(&key);
        NoiseHolder::new(holder, noise)
    }

    // JAVA: net.minecraft.world.level.levelgen.RandomState#apply
    fn apply(&mut self, f: DensityFunction) -> DensityFunction {
        if let Some(cached) = self.wrapped.get(&f) {
            return cached.clone();
        }
        let wrapped = self.wrap_new(f.clone());
        self.wrapped.insert(f, wrapped.clone());
        wrapped
    }
}

struct UnwrappingVisitor {
    wrapped: HashMap<DensityFunction, DensityFunction>,
}

impl UnwrappingVisitor {
    // JAVA: net.minecraft.world.level.levelgen.RandomState#wrapNew
    fn wrap_new(&self, f: DensityFunction) -> DensityFunction {
        match f {
            DensityFunction::HolderHolder(h) => h.value().as_ref().clone(),
            DensityFunction::Marker(inner) => (*inner).clone(),
            other => other,
        }
    }
}

impl DensityFunctionVisitor for UnwrappingVisitor {
    // JAVA: net.minecraft.world.level.levelgen.RandomState#apply
    fn apply(&mut self, f: DensityFunction) -> DensityFunction {
        if let Some(cached) = self.wrapped.get(&f) {
            return cached.clone();
        }
        let wrapped = self.wrap_new(f.clone());
        self.wrapped.insert(f, wrapped.clone());
        wrapped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestNoiseGetter {
        params: HashMap<ResourceKey, NoiseParameters>,
    }

    impl NoiseParametersGetter for TestNoiseGetter {
        fn get_or_throw(&self, key: &ResourceKey) -> NoiseParameters {
            self.params
                .get(key)
                .cloned()
                .unwrap_or_else(|| NoiseParameters::new(0, vec![1.0]))
        }
    }

    struct TestProvider;

    impl RegistriesProvider for TestProvider {
        fn noise_generator_settings(&self, _key: &ResourceKey) -> NoiseGeneratorSettings {
            test_settings(false, const_router())
        }

        fn noise_parameters(&self) -> Arc<dyn NoiseParametersGetter> {
            Arc::new(TestNoiseGetter {
                params: HashMap::new(),
            })
        }
    }

    fn const_router() -> NoiseRouter {
        let c = DensityFunction::Constant(0);
        NoiseRouter {
            barrier_noise: c.clone(),
            fluid_level_floodedness_noise: c.clone(),
            fluid_level_spread_noise: c.clone(),
            lava_noise: c.clone(),
            temperature: c.clone(),
            vegetation: c.clone(),
            continents: c.clone(),
            erosion: c.clone(),
            depth: c.clone(),
            ridges: c.clone(),
            initial_density_without_jaggedness: c.clone(),
            final_density: c.clone(),
            vein_toggle: c.clone(),
            vein_ridged: c.clone(),
            vein_gap: c,
        }
    }

    fn test_settings(use_legacy: bool, noise_router: NoiseRouter) -> NoiseGeneratorSettings {
        NoiseGeneratorSettings {
            random_source_algorithm: Arc::new(XoroshiroAlgorithm),
            noise_router,
            default_block: BlockState {
                name: "minecraft:stone".to_owned(),
            },
            sea_level: 63,
            use_legacy_random_source: use_legacy,
            spawn_target: vec![],
        }
    }

    #[test]
    fn create_and_accessors() {
        let key = ResourceKey::new(Identifier::with_default_namespace("overworld"));
        let rs = RandomState::create(&TestProvider, &key, 12345);
        assert_eq!(rs.surface_system().sea_level, 63);
        assert_eq!(rs.router().temperature, DensityFunction::Constant(0));
        assert_eq!(rs.sampler().temperature, DensityFunction::Constant(0));
        let _ = rs.aquifer_random();
        let _ = rs.ore_random();
    }

    #[test]
    fn get_or_create_noise_caches_and_float_bits() {
        let key = ResourceKey::new(Identifier::with_default_namespace("test"));
        let mut params_map = HashMap::new();
        params_map.insert(
            key.clone(),
            NoiseParameters::new(-10, vec![f64::NAN, f64::INFINITY, -0.0, -1.5]),
        );
        let getter = Arc::new(TestNoiseGetter { params: params_map });
        let rs = RandomState::create_from_settings(test_settings(false, const_router()), getter, 1);
        let n1 = rs.get_or_create_noise(&key);
        let n2 = rs.get_or_create_noise(&key);
        assert_eq!(n1, n2);
        assert_eq!(n1.parameters.first_octave, -10);
        assert_eq!(n1.parameters.amplitudes[0], f64::NAN.to_bits());
        assert_eq!(n1.parameters.amplitudes[1], f64::INFINITY.to_bits());
        assert_eq!(n1.parameters.amplitudes[2], (-0.0f64).to_bits());
        assert_ne!(n1.parameters.amplitudes[2], 0.0f64.to_bits());
        assert_eq!(n1.parameters.amplitudes[3], (-1.5f64).to_bits());
    }

    #[test]
    fn get_or_create_random_factory_caches() {
        let getter = Arc::new(TestNoiseGetter {
            params: HashMap::new(),
        });
        let rs = RandomState::create_from_settings(test_settings(false, const_router()), getter, 42);
        let id = Identifier::with_default_namespace("test_factory");
        let f1 = rs.get_or_create_random_factory(&id);
        let f2 = rs.get_or_create_random_factory(&id);
        assert!(Arc::ptr_eq(&f1, &f2));
        let other =
            rs.get_or_create_random_factory(&Identifier::with_default_namespace("other"));
        assert!(!Arc::ptr_eq(&f1, &other));
    }

    #[test]
    fn legacy_wiring_seed_wrapping_overflow_and_negative() {
        let seed = i64::MAX;
        let temp_key = Noises::temperature();
        let veg_key = Noises::vegetation();
        let shift_key = Noises::shift();

        let mk_holder = |k: &ResourceKey| {
            Holder::new(Some(k.clone()), Arc::new(NoiseParameters::new(0, vec![1.0])))
        };
        let dummy_noise = || {
            NormalNoise::create(
                &SimpleRandomSource { seed: 0 },
                NoiseParameters::new(0, vec![]),
            )
        };

        let mut router = const_router();
        router.temperature =
            DensityFunction::NoiseHolder(NoiseHolder::new(mk_holder(&temp_key), dummy_noise()));
        router.vegetation =
            DensityFunction::NoiseHolder(NoiseHolder::new(mk_holder(&veg_key), dummy_noise()));
        router.ridges =
            DensityFunction::NoiseHolder(NoiseHolder::new(mk_holder(&shift_key), dummy_noise()));
        router.final_density = DensityFunction::EndIslandDensityFunction(0);
        router.continents = DensityFunction::BlendedNoise(BlendedNoise { random_seed: 0 });

        let getter = Arc::new(TestNoiseGetter {
            params: HashMap::new(),
        });
        let rs = RandomState::create_from_settings(test_settings(true, router), getter, seed);

        match &rs.router().temperature {
            DensityFunction::NoiseHolder(nh) => {
                assert!(nh.noise.legacy_nether_biome);
                assert_eq!(nh.noise.random_seed, seed.wrapping_add(0));
                assert_eq!(nh.noise.parameters, NoiseParameters::new(-7, vec![1.0, 1.0]));
            }
            other => panic!("expected temperature NoiseHolder, got {:?}", other),
        }
        match &rs.router().vegetation {
            DensityFunction::NoiseHolder(nh) => {
                assert!(nh.noise.legacy_nether_biome);
                assert_eq!(nh.noise.random_seed, seed.wrapping_add(1));
                assert_eq!(nh.noise.parameters, NoiseParameters::new(-7, vec![1.0, 1.0]));
            }
            other => panic!("expected vegetation NoiseHolder, got {:?}", other),
        }
        match &rs.router().ridges {
            DensityFunction::NoiseHolder(nh) => {
                assert!(!nh.noise.legacy_nether_biome);
                assert_eq!(nh.noise.parameters, NoiseParameters::new(0, vec![0.0]));
            }
            other => panic!("expected shift NoiseHolder, got {:?}", other),
        }
        assert_eq!(
            rs.router().final_density,
            DensityFunction::EndIslandDensityFunction(seed)
        );
        assert_eq!(
            rs.router().continents,
            DensityFunction::BlendedNoise(BlendedNoise {
                random_seed: seed.wrapping_add(0)
            })
        );
    }

    #[test]
    fn sampler_unwraps_holder_and_marker_negative_seed() {
        let getter = Arc::new(TestNoiseGetter {
            params: HashMap::new(),
        });
        let inner = DensityFunction::Constant(42u64);
        let mut router = const_router();
        router.temperature = DensityFunction::Marker(Box::new(inner.clone()));
        router.vegetation =
            DensityFunction::HolderHolder(Holder::new(None, Arc::new(inner.clone())));
        let rs = RandomState::create_from_settings(test_settings(false, router), getter, -999);
        assert_eq!(rs.sampler().temperature, inner);
        assert_eq!(rs.sampler().vegetation, inner);
        assert_eq!(rs.sampler().continents, DensityFunction::Constant(0));
    }
}
