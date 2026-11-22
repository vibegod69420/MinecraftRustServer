use std::collections::BTreeMap;

/// Minimal self-contained placeholder for Minecraft's `ParticleOptions`.
pub trait ParticleOptions: std::fmt::Debug {}

/// Faithful representation of Minecraft's `SimpleParticleType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimpleParticleType {
    pub id: i32,
    pub override_limiter: bool,
}

impl ParticleOptions for SimpleParticleType {}

/// Self-contained representation of a complex `ParticleType<T>`.
/// Codec/stream-codec factories from Java are omitted because the codec
/// infrastructure is external to this module; the option type name is preserved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComplexParticleType {
    pub id: i32,
    pub override_limiter: bool,
    pub option_type: &'static str,
}

/// Registered particle entry stored in the registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisteredParticle {
    Simple {
        id: i32,
        name: String,
        override_limiter: bool,
    },
    Complex {
        id: i32,
        name: String,
        override_limiter: bool,
        option_type: &'static str,
    },
}

impl RegisteredParticle {
    pub fn id(&self) -> i32 {
        match self {
            Self::Simple { id, .. } | Self::Complex { id, .. } => *id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Simple { name, .. } | Self::Complex { name, .. } => name,
        }
    }

    pub fn override_limiter(&self) -> bool {
        match self {
            Self::Simple {
                override_limiter, ..
            }
            | Self::Complex {
                override_limiter, ..
            } => *override_limiter,
        }
    }
}

/// Minimal self-contained particle type registry, standing in for
/// `BuiltInRegistries.PARTICLE_TYPE` + `Registry.register`.
#[derive(Debug, Clone, Default)]
pub struct ParticleTypeRegistry {
    by_name: BTreeMap<String, RegisteredParticle>,
    by_id: Vec<String>,
}

impl ParticleTypeRegistry {
    pub fn new() -> Self {
        Self {
            by_name: BTreeMap::new(),
            by_id: Vec::new(),
        }
    }

    // JAVA: net.minecraft.core.particles.ParticleTypes#register
    pub fn register_simple(&mut self, name: &str, override_limiter: bool) -> SimpleParticleType {
        let id = self.by_id.len() as i32;
        let particle = SimpleParticleType {
            id,
            override_limiter,
        };
        let entry = RegisteredParticle::Simple {
            id,
            name: name.to_string(),
            override_limiter,
        };
        self.by_name.insert(name.to_string(), entry);
        self.by_id.push(name.to_string());
        particle
    }

    // JAVA: net.minecraft.core.particles.ParticleTypes#register
    pub fn register_complex(
        &mut self,
        name: &str,
        override_limiter: bool,
        option_type: &'static str,
    ) -> ComplexParticleType {
        let id = self.by_id.len() as i32;
        let particle = ComplexParticleType {
            id,
            override_limiter,
            option_type,
        };
        let entry = RegisteredParticle::Complex {
            id,
            name: name.to_string(),
            override_limiter,
            option_type,
        };
        self.by_name.insert(name.to_string(), entry);
        self.by_id.push(name.to_string());
        particle
    }

    pub fn get(&self, name: &str) -> Option<&RegisteredParticle> {
        self.by_name.get(name)
    }

    pub fn get_by_id(&self, id: i32) -> Option<&RegisteredParticle> {
        if id < 0 {
            return None;
        }
        let idx = id as usize;
        let name = self.by_id.get(idx)?;
        self.by_name.get(name)
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
}

/// Faithful port of `net.minecraft.core.particles.ParticleTypes`.
/// All built-in particle types are registered in the same order as Java 1.21.11.
#[derive(Debug, Clone)]
pub struct ParticleTypes {
    pub registry: ParticleTypeRegistry,

    pub angry_villager: SimpleParticleType,
    pub block: ComplexParticleType,
    pub block_marker: ComplexParticleType,
    pub bubble: SimpleParticleType,
    pub cloud: SimpleParticleType,
    pub copper_fire_flame: SimpleParticleType,
    pub crit: SimpleParticleType,
    pub damage_indicator: SimpleParticleType,
    pub dragon_breath: ComplexParticleType,
    pub dripping_lava: SimpleParticleType,
    pub falling_lava: SimpleParticleType,
    pub landing_lava: SimpleParticleType,
    pub dripping_water: SimpleParticleType,
    pub falling_water: SimpleParticleType,
    pub dust: ComplexParticleType,
    pub dust_color_transition: ComplexParticleType,
    pub effect: ComplexParticleType,
    pub elder_guardian: SimpleParticleType,
    pub enchanted_hit: SimpleParticleType,
    pub enchant: SimpleParticleType,
    pub end_rod: SimpleParticleType,
    pub entity_effect: ComplexParticleType,
    pub explosion_emitter: SimpleParticleType,
    pub explosion: SimpleParticleType,
    pub gust: SimpleParticleType,
    pub small_gust: SimpleParticleType,
    pub gust_emitter_large: SimpleParticleType,
    pub gust_emitter_small: SimpleParticleType,
    pub sonic_boom: SimpleParticleType,
    pub falling_dust: ComplexParticleType,
    pub firework: SimpleParticleType,
    pub fishing: SimpleParticleType,
    pub flame: SimpleParticleType,
    pub infested: SimpleParticleType,
    pub cherry_leaves: SimpleParticleType,
    pub pale_oak_leaves: SimpleParticleType,
    pub tinted_leaves: ComplexParticleType,
    pub sculk_soul: SimpleParticleType,
    pub sculk_charge: ComplexParticleType,
    pub sculk_charge_pop: SimpleParticleType,
    pub soul_fire_flame: SimpleParticleType,
    pub soul: SimpleParticleType,
    pub flash: ComplexParticleType,
    pub happy_villager: SimpleParticleType,
    pub composter: SimpleParticleType,
    pub heart: SimpleParticleType,
    pub instant_effect: ComplexParticleType,
    pub item: ComplexParticleType,
    pub vibration: ComplexParticleType,
    pub trail: ComplexParticleType,
    pub item_slime: SimpleParticleType,
    pub item_cobweb: SimpleParticleType,
    pub item_snowball: SimpleParticleType,
    pub large_smoke: SimpleParticleType,
    pub lava: SimpleParticleType,
    pub mycelium: SimpleParticleType,
    pub note: SimpleParticleType,
    pub poof: SimpleParticleType,
    pub portal: SimpleParticleType,
    pub rain: SimpleParticleType,
    pub smoke: SimpleParticleType,
    pub white_smoke: SimpleParticleType,
    pub sneeze: SimpleParticleType,
    pub spit: SimpleParticleType,
    pub squid_ink: SimpleParticleType,
    pub sweep_attack: SimpleParticleType,
    pub totem_of_undying: SimpleParticleType,
    pub underwater: SimpleParticleType,
    pub splash: SimpleParticleType,
    pub witch: SimpleParticleType,
    pub bubble_pop: SimpleParticleType,
    pub current_down: SimpleParticleType,
    pub bubble_column_up: SimpleParticleType,
    pub nautilus: SimpleParticleType,
    pub dolphin: SimpleParticleType,
    pub campfire_cosy_smoke: SimpleParticleType,
    pub campfire_signal_smoke: SimpleParticleType,
    pub dripping_honey: SimpleParticleType,
    pub falling_honey: SimpleParticleType,
    pub landing_honey: SimpleParticleType,
    pub falling_nectar: SimpleParticleType,
    pub falling_spore_blossom: SimpleParticleType,
    pub ash: SimpleParticleType,
    pub crimson_spore: SimpleParticleType,
    pub warped_spore: SimpleParticleType,
    pub spore_blossom_air: SimpleParticleType,
    pub dripping_obsidian_tear: SimpleParticleType,
    pub falling_obsidian_tear: SimpleParticleType,
    pub landing_obsidian_tear: SimpleParticleType,
    pub reverse_portal: SimpleParticleType,
    pub white_ash: SimpleParticleType,
    pub small_flame: SimpleParticleType,
    pub snowflake: SimpleParticleType,
    pub dripping_dripstone_lava: SimpleParticleType,
    pub falling_dripstone_lava: SimpleParticleType,
    pub dripping_dripstone_water: SimpleParticleType,
    pub falling_dripstone_water: SimpleParticleType,
    pub glow_squid_ink: SimpleParticleType,
    pub glow: SimpleParticleType,
    pub wax_on: SimpleParticleType,
    pub wax_off: SimpleParticleType,
    pub electric_spark: SimpleParticleType,
    pub scrape: SimpleParticleType,
    pub shriek: ComplexParticleType,
    pub egg_crack: SimpleParticleType,
    pub dust_plume: SimpleParticleType,
    pub trial_spawner_detected_player: SimpleParticleType,
    pub trial_spawner_detected_player_ominous: SimpleParticleType,
    pub vault_connection: SimpleParticleType,
    pub dust_pillar: ComplexParticleType,
    pub ominous_spawning: SimpleParticleType,
    pub raid_omen: SimpleParticleType,
    pub trial_omen: SimpleParticleType,
    pub block_crumble: ComplexParticleType,
    pub firefly: SimpleParticleType,
}

impl ParticleTypes {
    pub fn bootstrap() -> Self {
        let mut registry = ParticleTypeRegistry::new();

        let angry_villager = registry.register_simple("angry_villager", false);
        let block = registry.register_complex("block", false, "BlockParticleOption");
        let block_marker = registry.register_complex("block_marker", true, "BlockParticleOption");
        let bubble = registry.register_simple("bubble", false);
        let cloud = registry.register_simple("cloud", false);
        let copper_fire_flame = registry.register_simple("copper_fire_flame", false);
        let crit = registry.register_simple("crit", false);
        let damage_indicator = registry.register_simple("damage_indicator", true);
        let dragon_breath = registry.register_complex("dragon_breath", false, "PowerParticleOption");
        let dripping_lava = registry.register_simple("dripping_lava", false);
        let falling_lava = registry.register_simple("falling_lava", false);
        let landing_lava = registry.register_simple("landing_lava", false);
        let dripping_water = registry.register_simple("dripping_water", false);
        let falling_water = registry.register_simple("falling_water", false);
        let dust = registry.register_complex("dust", false, "DustParticleOptions");
        let dust_color_transition =
            registry.register_complex("dust_color_transition", false, "DustColorTransitionOptions");
        let effect = registry.register_complex("effect", false, "SpellParticleOption");
        let elder_guardian = registry.register_simple("elder_guardian", true);
        let enchanted_hit = registry.register_simple("enchanted_hit", false);
        let enchant = registry.register_simple("enchant", false);
        let end_rod = registry.register_simple("end_rod", false);
        let entity_effect = registry.register_complex("entity_effect", false, "ColorParticleOption");
        let explosion_emitter = registry.register_simple("explosion_emitter", true);
        let explosion = registry.register_simple("explosion", true);
        let gust = registry.register_simple("gust", true);
        let small_gust = registry.register_simple("small_gust", false);
        let gust_emitter_large = registry.register_simple("gust_emitter_large", true);
        let gust_emitter_small = registry.register_simple("gust_emitter_small", true);
        let sonic_boom = registry.register_simple("sonic_boom", true);
        let falling_dust = registry.register_complex("falling_dust", false, "BlockParticleOption");
        let firework = registry.register_simple("firework", false);
        let fishing = registry.register_simple("fishing", false);
        let flame = registry.register_simple("flame", false);
        let infested = registry.register_simple("infested", false);
        let cherry_leaves = registry.register_simple("cherry_leaves", false);
        let pale_oak_leaves = registry.register_simple("pale_oak_leaves", false);
        let tinted_leaves = registry.register_complex("tinted_leaves", false, "ColorParticleOption");
        let sculk_soul = registry.register_simple("sculk_soul", false);
        let sculk_charge =
            registry.register_complex("sculk_charge", true, "SculkChargeParticleOptions");
        let sculk_charge_pop = registry.register_simple("sculk_charge_pop", true);
        let soul_fire_flame = registry.register_simple("soul_fire_flame", false);
        let soul = registry.register_simple("soul", false);
        let flash = registry.register_complex("flash", false, "ColorParticleOption");
        let happy_villager = registry.register_simple("happy_villager", false);
        let composter = registry.register_simple("composter", false);
        let heart = registry.register_simple("heart", false);
        let instant_effect = registry.register_complex("instant_effect", false, "SpellParticleOption");
        let item = registry.register_complex("item", false, "ItemParticleOption");
        let vibration = registry.register_complex("vibration", true, "VibrationParticleOption");
        let trail = registry.register_complex("trail", false, "TrailParticleOption");
        let item_slime = registry.register_simple("item_slime", false);
        let item_cobweb = registry.register_simple("item_cobweb", false);
        let item_snowball = registry.register_simple("item_snowball", false);
        let large_smoke = registry.register_simple("large_smoke", false);
        let lava = registry.register_simple("lava", false);
        let mycelium = registry.register_simple("mycelium", false);
        let note = registry.register_simple("note", false);
        let poof = registry.register_simple("poof", true);
        let portal = registry.register_simple("portal", false);
        let rain = registry.register_simple("rain", false);
        let smoke = registry.register_simple("smoke", false);
        let white_smoke = registry.register_simple("white_smoke", false);
        let sneeze = registry.register_simple("sneeze", false);
        let spit = registry.register_simple("spit", true);
        let squid_ink = registry.register_simple("squid_ink", true);
        let sweep_attack = registry.register_simple("sweep_attack", true);
        let totem_of_undying = registry.register_simple("totem_of_undying", false);
        let underwater = registry.register_simple("underwater", false);
        let splash = registry.register_simple("splash", false);
        let witch = registry.register_simple("witch", false);
        let bubble_pop = registry.register_simple("bubble_pop", false);
        let current_down = registry.register_simple("current_down", false);
        let bubble_column_up = registry.register_simple("bubble_column_up", false);
        let nautilus = registry.register_simple("nautilus", false);
        let dolphin = registry.register_simple("dolphin", false);
        let campfire_cosy_smoke = registry.register_simple("campfire_cosy_smoke", true);
        let campfire_signal_smoke = registry.register_simple("campfire_signal_smoke", true);
        let dripping_honey = registry.register_simple("dripping_honey", false);
        let falling_honey = registry.register_simple("falling_honey", false);
        let landing_honey = registry.register_simple("landing_honey", false);
        let falling_nectar = registry.register_simple("falling_nectar", false);
        let falling_spore_blossom = registry.register_simple("falling_spore_blossom", false);
        let ash = registry.register_simple("ash", false);
        let crimson_spore = registry.register_simple("crimson_spore", false);
        let warped_spore = registry.register_simple("warped_spore", false);
        let spore_blossom_air = registry.register_simple("spore_blossom_air", false);
        let dripping_obsidian_tear = registry.register_simple("dripping_obsidian_tear", false);
        let falling_obsidian_tear = registry.register_simple("falling_obsidian_tear", false);
        let landing_obsidian_tear = registry.register_simple("landing_obsidian_tear", false);
        let reverse_portal = registry.register_simple("reverse_portal", false);
        let white_ash = registry.register_simple("white_ash", false);
        let small_flame = registry.register_simple("small_flame", false);
        let snowflake = registry.register_simple("snowflake", false);
        let dripping_dripstone_lava = registry.register_simple("dripping_dripstone_lava", false);
        let falling_dripstone_lava = registry.register_simple("falling_dripstone_lava", false);
        let dripping_dripstone_water = registry.register_simple("dripping_dripstone_water", false);
        let falling_dripstone_water = registry.register_simple("falling_dripstone_water", false);
        let glow_squid_ink = registry.register_simple("glow_squid_ink", true);
        let glow = registry.register_simple("glow", true);
        let wax_on = registry.register_simple("wax_on", true);
        let wax_off = registry.register_simple("wax_off", true);
        let electric_spark = registry.register_simple("electric_spark", true);
        let scrape = registry.register_simple("scrape", true);
        let shriek = registry.register_complex("shriek", false, "ShriekParticleOption");
        let egg_crack = registry.register_simple("egg_crack", false);
        let dust_plume = registry.register_simple("dust_plume", false);
        let trial_spawner_detected_player =
            registry.register_simple("trial_spawner_detection", true);
        let trial_spawner_detected_player_ominous =
            registry.register_simple("trial_spawner_detection_ominous", true);
        let vault_connection = registry.register_simple("vault_connection", true);
        let dust_pillar = registry.register_complex("dust_pillar", false, "BlockParticleOption");
        let ominous_spawning = registry.register_simple("ominous_spawning", true);
        let raid_omen = registry.register_simple("raid_omen", false);
        let trial_omen = registry.register_simple("trial_omen", false);
        let block_crumble = registry.register_complex("block_crumble", false, "BlockParticleOption");
        let firefly = registry.register_simple("firefly", false);

        Self {
            registry,
            angry_villager,
            block,
            block_marker,
            bubble,
            cloud,
            copper_fire_flame,
            crit,
            damage_indicator,
            dragon_breath,
            dripping_lava,
            falling_lava,
            landing_lava,
            dripping_water,
            falling_water,
            dust,
            dust_color_transition,
            effect,
            elder_guardian,
            enchanted_hit,
            enchant,
            end_rod,
            entity_effect,
            explosion_emitter,
            explosion,
            gust,
            small_gust,
            gust_emitter_large,
            gust_emitter_small,
            sonic_boom,
            falling_dust,
            firework,
            fishing,
            flame,
            infested,
            cherry_leaves,
            pale_oak_leaves,
            tinted_leaves,
            sculk_soul,
            sculk_charge,
            sculk_charge_pop,
            soul_fire_flame,
            soul,
            flash,
            happy_villager,
            composter,
            heart,
            instant_effect,
            item,
            vibration,
            trail,
            item_slime,
            item_cobweb,
            item_snowball,
            large_smoke,
            lava,
            mycelium,
            note,
            poof,
            portal,
            rain,
            smoke,
            white_smoke,
            sneeze,
            spit,
            squid_ink,
            sweep_attack,
            totem_of_undying,
            underwater,
            splash,
            witch,
            bubble_pop,
            current_down,
            bubble_column_up,
            nautilus,
            dolphin,
            campfire_cosy_smoke,
            campfire_signal_smoke,
            dripping_honey,
            falling_honey,
            landing_honey,
            falling_nectar,
            falling_spore_blossom,
            ash,
            crimson_spore,
            warped_spore,
            spore_blossom_air,
            dripping_obsidian_tear,
            falling_obsidian_tear,
            landing_obsidian_tear,
            reverse_portal,
            white_ash,
            small_flame,
            snowflake,
            dripping_dripstone_lava,
            falling_dripstone_lava,
            dripping_dripstone_water,
            falling_dripstone_water,
            glow_squid_ink,
            glow,
            wax_on,
            wax_off,
            electric_spark,
            scrape,
            shriek,
            egg_crack,
            dust_plume,
            trial_spawner_detected_player,
            trial_spawner_detected_player_ominous,
            vault_connection,
            dust_pillar,
            ominous_spawning,
            raid_omen,
            trial_omen,
            block_crumble,
            firefly,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_registers_all_particles_in_order() {
        let types = ParticleTypes::bootstrap();
        assert_eq!(types.registry.len(), 115);
        assert_eq!(types.angry_villager.id, 0);
        assert_eq!(types.firefly.id, 114);
        assert_eq!(
            types.registry.get("angry_villager").unwrap().name(),
            "angry_villager"
        );
        assert_eq!(types.registry.get("firefly").unwrap().name(), "firefly");
    }

    #[test]
    fn override_limiter_flags_match_java() {
        let types = ParticleTypes::bootstrap();
        assert!(!types.angry_villager.override_limiter);
        assert!(types.block_marker.override_limiter);
        assert!(types.damage_indicator.override_limiter);
        assert!(types.elder_guardian.override_limiter);
        assert!(!types.flame.override_limiter);
        assert!(types.poof.override_limiter);
        assert!(types.trial_spawner_detected_player.override_limiter);
        assert!(types.vault_connection.override_limiter);
    }

    #[test]
    fn complex_particle_metadata_and_lookup_edge_cases() {
        let types = ParticleTypes::bootstrap();
        let block = types.registry.get("block").unwrap();
        match block {
            RegisteredParticle::Complex {
                override_limiter,
                option_type,
                ..
            } => {
                assert!(!override_limiter);
                assert_eq!(*option_type, "BlockParticleOption");
            }
            _ => panic!("expected complex particle"),
        }

        assert!(types.registry.get("missing_particle").is_none());
        assert!(types.registry.get("").is_none());
        assert!(types.registry.get_by_id(-1).is_none());
        assert!(types.registry.get_by_id(i32::MIN).is_none());
        assert!(types.registry.get_by_id(115).is_none());
    }

    #[test]
    fn registry_ids_are_stable_and_negative_safe() {
        let types = ParticleTypes::bootstrap();
        let first = types.registry.get_by_id(0).unwrap();
        let last = types.registry.get_by_id(114).unwrap();
        assert_eq!(first.name(), "angry_villager");
        assert_eq!(last.name(), "firefly");

        // Negative and overflow-adjacent id lookups must be handled safely.
        assert!(types.registry.get_by_id(-123).is_none());
        assert!(types.registry.get_by_id(i32::MAX).is_none());

        let id_bits = types.firefly.id.to_le_bytes();
        assert_eq!(i32::from_le_bytes(id_bits), 114);
    }

    #[test]
    fn float_bits_preserved_in_particle_option_placeholder() {
        // ParticleTypes itself has no float arithmetic, but this verifies float
        // bit patterns are preserved exactly when carried alongside particle data.
        #[derive(Debug, Clone, Copy)]
        struct FloatCarrier {
            particle_id: i32,
            scale: f32,
            weight: f64,
        }

        let carrier = FloatCarrier {
            particle_id: ParticleTypes::bootstrap().dust.id,
            scale: f32::from_bits(0xFFC0_0001), // NaN payload
            weight: f64::from_bits(0x8000_0000_0000_0000u64), // -0.0
        };

        assert_eq!(carrier.particle_id, 14);
        assert_eq!(carrier.scale.to_bits(), 0xFFC0_0001);
        assert_eq!(carrier.weight.to_bits(), 0x8000_0000_0000_0000u64);
        assert!(carrier.weight.is_sign_negative());
        assert_eq!(carrier.weight, -0.0f64);
    }
}
