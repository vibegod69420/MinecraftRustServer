//! Port of `net.minecraft.world.level.levelgen.NoiseGeneratorSettings`.
//!
//! External Minecraft types are represented by minimal local placeholders so the
//! module remains self-contained while preserving the behavior of
//! `NoiseGeneratorSettings` itself.

/// Minimal placeholder for `net.minecraft.SharedConstants`.
pub struct SharedConstants;

impl SharedConstants {
    pub const DEBUG_DISABLE_AQUIFERS: bool = false;
    pub const DEBUG_DISABLE_ORE_VEINS: bool = false;
}

/// Minimal placeholder for `net.minecraft.resources.ResourceKey<NoiseGeneratorSettings>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    pub namespace: &'static str,
    pub path: &'static str,
}

impl ResourceKey {
    pub const fn create_noise_settings(path: &'static str) -> Self {
        Self {
            namespace: "minecraft",
            path,
        }
    }
}

/// Minimal placeholder for `net.minecraft.world.level.levelgen.NoiseSettings`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct NoiseSettings {
    pub name: &'static str,
}

impl NoiseSettings {
    pub const END_NOISE_SETTINGS: Self = Self { name: "end" };
    pub const NETHER_NOISE_SETTINGS: Self = Self { name: "nether" };
    pub const OVERWORLD_NOISE_SETTINGS: Self = Self { name: "overworld" };
    pub const CAVES_NOISE_SETTINGS: Self = Self { name: "caves" };
    pub const FLOATING_ISLANDS_NOISE_SETTINGS: Self = Self {
        name: "floating_islands",
    };
}

/// Minimal placeholder for `net.minecraft.world.level.block.state.BlockState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockState {
    pub name: &'static str,
}

impl BlockState {
    pub const fn of(name: &'static str) -> Self {
        Self { name }
    }
}

/// Minimal placeholder block constants corresponding to `Blocks.*.defaultBlockState()`.
pub mod blocks {
    use super::BlockState;

    pub const END_STONE: BlockState = BlockState::of("end_stone");
    pub const AIR: BlockState = BlockState::of("air");
    pub const NETHERRACK: BlockState = BlockState::of("netherrack");
    pub const LAVA: BlockState = BlockState::of("lava");
    pub const STONE: BlockState = BlockState::of("stone");
    pub const WATER: BlockState = BlockState::of("water");
}

/// Minimal placeholder for `net.minecraft.world.level.levelgen.NoiseRouter`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct NoiseRouter {
    pub name: &'static str,
}

impl NoiseRouter {
    pub fn none() -> Self {
        Self { name: "none" }
    }
}

/// Minimal placeholder for `net.minecraft.world.level.levelgen.SurfaceRules.RuleSource`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SurfaceRuleSource {
    End,
    Nether,
    Overworld,
    OverworldLike {
        capped: bool,
        sandy: bool,
        frozen: bool,
    },
    Air,
}

/// Minimal placeholder for `net.minecraft.world.level.biome.Climate.ParameterPoint`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ParameterPoint;

/// Minimal placeholder for `net.minecraft.world.level.levelgen.WorldgenRandom.Algorithm`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorldgenRandomAlgorithm {
    Legacy,
    Xoroshiro,
}

/// Abstraction over external data needed by `NoiseGeneratorSettings` factory methods.
///
/// This replaces direct dependencies on `BootstrapContext`, `NoiseRouterData`,
/// `SurfaceRuleData`, and `OverworldBiomeBuilder` while preserving factory behavior.
pub trait NoiseGeneratorSettingsContext {
    fn end_noise_router(&self) -> NoiseRouter;
    fn nether_noise_router(&self) -> NoiseRouter;
    fn overworld_noise_router(&self, large_biomes: bool, amplified: bool) -> NoiseRouter;
    fn caves_noise_router(&self) -> NoiseRouter;
    fn floating_islands_noise_router(&self) -> NoiseRouter;
    fn overworld_spawn_target(&self) -> Vec<ParameterPoint>;
}

/// Abstraction over `BootstrapContext<NoiseGeneratorSettings>::register`.
pub trait NoiseSettingsRegistrar {
    fn register(&mut self, key: ResourceKey, settings: NoiseGeneratorSettings);
}

pub const OVERWORLD: ResourceKey = ResourceKey::create_noise_settings("overworld");
pub const LARGE_BIOMES: ResourceKey = ResourceKey::create_noise_settings("large_biomes");
pub const AMPLIFIED: ResourceKey = ResourceKey::create_noise_settings("amplified");
pub const NETHER: ResourceKey = ResourceKey::create_noise_settings("nether");
pub const END: ResourceKey = ResourceKey::create_noise_settings("end");
pub const CAVES: ResourceKey = ResourceKey::create_noise_settings("caves");
pub const FLOATING_ISLANDS: ResourceKey = ResourceKey::create_noise_settings("floating_islands");

/// Port of `net.minecraft.world.level.levelgen.NoiseGeneratorSettings`.
#[derive(Debug, Clone, PartialEq)]
pub struct NoiseGeneratorSettings {
    pub noise_settings: NoiseSettings,
    pub default_block: BlockState,
    pub default_fluid: BlockState,
    pub noise_router: NoiseRouter,
    pub surface_rule: SurfaceRuleSource,
    pub spawn_target: Vec<ParameterPoint>,
    pub sea_level: i32,
    /// Corresponds to the deprecated `disableMobGeneration` record component.
    pub disable_mob_generation: bool,
    /// Raw record component; use `is_aquifers_enabled()` for gated behavior.
    pub aquifers_enabled: bool,
    /// Raw record component; use `ore_veins_enabled()` for gated behavior.
    pub ore_veins_enabled: bool,
    pub use_legacy_random_source: bool,
}

impl NoiseGeneratorSettings {
    // JAVA: net.minecraft.world.level.levelgen.NoiseGeneratorSettings#isAquifersEnabled
    pub fn is_aquifers_enabled(&self) -> bool {
        self.aquifers_enabled && !SharedConstants::DEBUG_DISABLE_AQUIFERS
    }

    // JAVA: net.minecraft.world.level.levelgen.NoiseGeneratorSettings#oreVeinsEnabled
    pub fn ore_veins_enabled(&self) -> bool {
        self.ore_veins_enabled && !SharedConstants::DEBUG_DISABLE_ORE_VEINS
    }

    // JAVA: net.minecraft.world.level.levelgen.NoiseGeneratorSettings#getRandomSource
    pub fn get_random_source(&self) -> WorldgenRandomAlgorithm {
        if self.use_legacy_random_source {
            WorldgenRandomAlgorithm::Legacy
        } else {
            WorldgenRandomAlgorithm::Xoroshiro
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.NoiseGeneratorSettings#bootstrap
    pub fn bootstrap<C: NoiseGeneratorSettingsContext + NoiseSettingsRegistrar>(ctx: &mut C) {
        let overworld = Self::overworld(ctx, false, false);
        let large_biomes = Self::overworld(ctx, false, true);
        let amplified = Self::overworld(ctx, true, false);
        let nether = Self::nether(ctx);
        let end = Self::end(ctx);
        let caves = Self::caves(ctx);
        let floating_islands = Self::floating_islands(ctx);

        ctx.register(OVERWORLD, overworld);
        ctx.register(LARGE_BIOMES, large_biomes);
        ctx.register(AMPLIFIED, amplified);
        ctx.register(NETHER, nether);
        ctx.register(END, end);
        ctx.register(CAVES, caves);
        ctx.register(FLOATING_ISLANDS, floating_islands);
    }

    // JAVA: net.minecraft.world.level.levelgen.NoiseGeneratorSettings#end
    fn end<C: NoiseGeneratorSettingsContext>(ctx: &C) -> Self {
        Self {
            noise_settings: NoiseSettings::END_NOISE_SETTINGS,
            default_block: blocks::END_STONE,
            default_fluid: blocks::AIR,
            noise_router: ctx.end_noise_router(),
            surface_rule: SurfaceRuleSource::End,
            spawn_target: Vec::new(),
            sea_level: 0,
            disable_mob_generation: true,
            aquifers_enabled: false,
            ore_veins_enabled: false,
            use_legacy_random_source: true,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.NoiseGeneratorSettings#nether
    fn nether<C: NoiseGeneratorSettingsContext>(ctx: &C) -> Self {
        Self {
            noise_settings: NoiseSettings::NETHER_NOISE_SETTINGS,
            default_block: blocks::NETHERRACK,
            default_fluid: blocks::LAVA,
            noise_router: ctx.nether_noise_router(),
            surface_rule: SurfaceRuleSource::Nether,
            spawn_target: Vec::new(),
            sea_level: 32,
            disable_mob_generation: false,
            aquifers_enabled: false,
            ore_veins_enabled: false,
            use_legacy_random_source: true,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.NoiseGeneratorSettings#overworld
    fn overworld<C: NoiseGeneratorSettingsContext>(
        ctx: &C,
        amplified: bool,
        large_biomes: bool,
    ) -> Self {
        Self {
            noise_settings: NoiseSettings::OVERWORLD_NOISE_SETTINGS,
            default_block: blocks::STONE,
            default_fluid: blocks::WATER,
            noise_router: ctx.overworld_noise_router(large_biomes, amplified),
            surface_rule: SurfaceRuleSource::Overworld,
            spawn_target: ctx.overworld_spawn_target(),
            sea_level: 63,
            disable_mob_generation: false,
            aquifers_enabled: true,
            ore_veins_enabled: true,
            use_legacy_random_source: false,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.NoiseGeneratorSettings#caves
    fn caves<C: NoiseGeneratorSettingsContext>(ctx: &C) -> Self {
        Self {
            noise_settings: NoiseSettings::CAVES_NOISE_SETTINGS,
            default_block: blocks::STONE,
            default_fluid: blocks::WATER,
            noise_router: ctx.caves_noise_router(),
            surface_rule: SurfaceRuleSource::OverworldLike {
                capped: false,
                sandy: true,
                frozen: true,
            },
            spawn_target: Vec::new(),
            sea_level: 32,
            disable_mob_generation: false,
            aquifers_enabled: false,
            ore_veins_enabled: false,
            use_legacy_random_source: true,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.NoiseGeneratorSettings#floatingIslands
    fn floating_islands<C: NoiseGeneratorSettingsContext>(ctx: &C) -> Self {
        Self {
            noise_settings: NoiseSettings::FLOATING_ISLANDS_NOISE_SETTINGS,
            default_block: blocks::STONE,
            default_fluid: blocks::WATER,
            noise_router: ctx.floating_islands_noise_router(),
            surface_rule: SurfaceRuleSource::OverworldLike {
                capped: false,
                sandy: false,
                frozen: false,
            },
            spawn_target: Vec::new(),
            sea_level: -64,
            disable_mob_generation: false,
            aquifers_enabled: false,
            ore_veins_enabled: false,
            use_legacy_random_source: true,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.NoiseGeneratorSettings#dummy
    pub fn dummy() -> Self {
        Self {
            noise_settings: NoiseSettings::OVERWORLD_NOISE_SETTINGS,
            default_block: blocks::STONE,
            default_fluid: blocks::AIR,
            noise_router: NoiseRouter::none(),
            surface_rule: SurfaceRuleSource::Air,
            spawn_target: Vec::new(),
            sea_level: 63,
            disable_mob_generation: true,
            aquifers_enabled: false,
            ore_veins_enabled: false,
            use_legacy_random_source: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCtx {
        registered: Vec<(ResourceKey, NoiseGeneratorSettings)>,
    }

    impl MockCtx {
        fn new() -> Self {
            Self {
                registered: Vec::new(),
            }
        }
    }

    impl NoiseGeneratorSettingsContext for MockCtx {
        fn end_noise_router(&self) -> NoiseRouter {
            NoiseRouter { name: "end" }
        }

        fn nether_noise_router(&self) -> NoiseRouter {
            NoiseRouter { name: "nether" }
        }

        fn overworld_noise_router(&self, large_biomes: bool, amplified: bool) -> NoiseRouter {
            let name = match (large_biomes, amplified) {
                (false, false) => "overworld",
                (true, false) => "large_biomes",
                (false, true) => "amplified",
                (true, true) => "overworld_large_amplified",
            };
            NoiseRouter { name }
        }

        fn caves_noise_router(&self) -> NoiseRouter {
            NoiseRouter { name: "caves" }
        }

        fn floating_islands_noise_router(&self) -> NoiseRouter {
            NoiseRouter {
                name: "floating_islands",
            }
        }

        fn overworld_spawn_target(&self) -> Vec<ParameterPoint> {
            vec![ParameterPoint]
        }
    }

    impl NoiseSettingsRegistrar for MockCtx {
        fn register(&mut self, key: ResourceKey, settings: NoiseGeneratorSettings) {
            self.registered.push((key, settings));
        }
    }

    #[test]
    fn test_is_aquifers_enabled_gating() {
        let mut s = NoiseGeneratorSettings::dummy();
        s.aquifers_enabled = false;
        assert!(!s.is_aquifers_enabled());
        s.aquifers_enabled = true;
        // SharedConstants::DEBUG_DISABLE_AQUIFERS is false by default.
        assert!(s.is_aquifers_enabled());
    }

    #[test]
    fn test_ore_veins_enabled_gating() {
        let mut s = NoiseGeneratorSettings::dummy();
        s.ore_veins_enabled = false;
        assert!(!s.ore_veins_enabled());
        s.ore_veins_enabled = true;
        assert!(s.ore_veins_enabled());
    }

    #[test]
    fn test_get_random_source() {
        let mut s = NoiseGeneratorSettings::dummy();
        s.use_legacy_random_source = true;
        assert_eq!(s.get_random_source(), WorldgenRandomAlgorithm::Legacy);
        s.use_legacy_random_source = false;
        assert_eq!(s.get_random_source(), WorldgenRandomAlgorithm::Xoroshiro);
    }

    #[test]
    fn test_factory_values_and_negative_sea_level() {
        let ctx = MockCtx::new();

        let end = NoiseGeneratorSettings::end(&ctx);
        assert_eq!(end.sea_level, 0);
        assert!(end.disable_mob_generation);
        assert!(!end.aquifers_enabled);
        assert!(end.use_legacy_random_source);
        assert_eq!(end.default_block, blocks::END_STONE);
        assert_eq!(end.default_fluid, blocks::AIR);
        assert_eq!(end.surface_rule, SurfaceRuleSource::End);

        let nether = NoiseGeneratorSettings::nether(&ctx);
        assert_eq!(nether.sea_level, 32);
        assert_eq!(nether.default_block, blocks::NETHERRACK);
        assert_eq!(nether.default_fluid, blocks::LAVA);
        assert_eq!(nether.surface_rule, SurfaceRuleSource::Nether);

        let overworld = NoiseGeneratorSettings::overworld(&ctx, false, false);
        assert_eq!(overworld.sea_level, 63);
        assert!(overworld.aquifers_enabled);
        assert!(overworld.ore_veins_enabled);
        assert!(!overworld.use_legacy_random_source);
        assert_eq!(overworld.spawn_target.len(), 1);
        assert_eq!(overworld.surface_rule, SurfaceRuleSource::Overworld);

        let floating = NoiseGeneratorSettings::floating_islands(&ctx);
        assert_eq!(floating.sea_level, -64);
        assert_eq!(
            floating.noise_settings,
            NoiseSettings::FLOATING_ISLANDS_NOISE_SETTINGS
        );
        assert_eq!(
            floating.surface_rule,
            SurfaceRuleSource::OverworldLike {
                capped: false,
                sandy: false,
                frozen: false,
            }
        );

        let caves = NoiseGeneratorSettings::caves(&ctx);
        assert_eq!(caves.sea_level, 32);
        assert_eq!(
            caves.surface_rule,
            SurfaceRuleSource::OverworldLike {
                capped: false,
                sandy: true,
                frozen: true,
            }
        );

        let dummy = NoiseGeneratorSettings::dummy();
        assert_eq!(dummy.sea_level, 63);
        assert!(dummy.disable_mob_generation);
        assert_eq!(dummy.default_fluid, blocks::AIR);
        assert_eq!(dummy.surface_rule, SurfaceRuleSource::Air);
        assert_eq!(dummy.noise_router, NoiseRouter::none());
    }

    #[test]
    fn test_bootstrap_registers_all_keys_in_order() {
        let mut ctx = MockCtx::new();
        NoiseGeneratorSettings::bootstrap(&mut ctx);

        let keys: Vec<ResourceKey> = ctx.registered.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            keys,
            vec![
                OVERWORLD,
                LARGE_BIOMES,
                AMPLIFIED,
                NETHER,
                END,
                CAVES,
                FLOATING_ISLANDS
            ]
        );
        assert_eq!(ctx.registered.len(), 7);

        let overworld = &ctx.registered[0].1;
        assert_eq!(overworld.noise_router.name, "overworld");
        let large_biomes = &ctx.registered[1].1;
        assert_eq!(large_biomes.noise_router.name, "large_biomes");
        let amplified = &ctx.registered[2].1;
        assert_eq!(amplified.noise_router.name, "amplified");
        let nether = &ctx.registered[3].1;
        assert_eq!(nether.sea_level, 32);
        let end = &ctx.registered[4].1;
        assert_eq!(end.sea_level, 0);
        let caves = &ctx.registered[5].1;
        assert_eq!(caves.noise_settings, NoiseSettings::CAVES_NOISE_SETTINGS);
        let floating_islands = &ctx.registered[6].1;
        assert_eq!(floating_islands.sea_level, -64);
    }
}
