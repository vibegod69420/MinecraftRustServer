//! Port of `net.minecraft.world.level.levelgen.carver.CaveCarverConfiguration`.

use std::marker::PhantomData;

/// Minimal placeholder for `net.minecraft.core.HolderSet<Block>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderSet<T> {
    _marker: PhantomData<T>,
}

impl<T> HolderSet<T> {
    pub fn empty() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// Minimal placeholder for `net.minecraft.world.level.levelgen.VerticalAnchor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerticalAnchor {
    pub value: i32,
}

impl VerticalAnchor {
    pub const fn absolute(value: i32) -> Self {
        Self { value }
    }
}

/// Minimal placeholder for `net.minecraft.world.level.levelgen.heightproviders.HeightProvider`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeightProvider {
    pub min: i32,
    pub max: i32,
}

impl HeightProvider {
    pub const fn uniform(min: i32, max: i32) -> Self {
        Self { min, max }
    }
}

/// Minimal placeholder for `net.minecraft.util.valueproviders.FloatProvider`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatProvider {
    pub min: f32,
    pub max: f32,
}

impl FloatProvider {
    pub const fn constant(value: f32) -> Self {
        Self {
            min: value,
            max: value,
        }
    }

    pub const fn uniform(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    pub fn to_bits(self) -> u32 {
        // For constant providers this matches the Java float value bits.
        // For non-constant providers, we use min bits as a stable identity.
        self.min.to_bits()
    }
}

/// Minimal placeholder for `net.minecraft.world.level.block.Block`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block;

/// Port of `net.minecraft.world.level.levelgen.carver.CarverDebugSettings`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarverDebugSettings {
    pub debug_mode: bool,
    pub air_state: i32,
    pub water_state: i32,
    pub lava_state: i32,
    pub barrier_state: i32,
}

impl CarverDebugSettings {
    // JAVA: net.minecraft.world.level.levelgen.carver.CarverDebugSettings#<clinit>/DEFAULT
    pub const DEFAULT: Self = Self {
        debug_mode: false,
        air_state: 0,
        water_state: 0,
        lava_state: 0,
        barrier_state: 0,
    };
}

/// Port of `net.minecraft.world.level.levelgen.carver.CarverConfiguration`.
#[derive(Debug, Clone, PartialEq)]
pub struct CarverConfiguration {
    pub probability: f32,
    pub y: HeightProvider,
    pub y_scale: FloatProvider,
    pub lava_level: VerticalAnchor,
    pub debug_settings: CarverDebugSettings,
    pub replaceable: HolderSet<Block>,
}

impl CarverConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.carver.CarverConfiguration#<init>(float,HeightProvider,FloatProvider,VerticalAnchor,CarverDebugSettings,HolderSet)
    pub fn new(
        probability: f32,
        y: HeightProvider,
        y_scale: FloatProvider,
        lava_level: VerticalAnchor,
        debug_settings: CarverDebugSettings,
        replaceable: HolderSet<Block>,
    ) -> Self {
        Self {
            probability,
            y,
            y_scale,
            lava_level,
            debug_settings,
            replaceable,
        }
    }
}

/// Port of `net.minecraft.world.level.levelgen.carver.CaveCarverConfiguration`.
#[derive(Debug, Clone, PartialEq)]
pub struct CaveCarverConfiguration {
    pub base: CarverConfiguration,
    pub horizontal_radius_multiplier: FloatProvider,
    pub vertical_radius_multiplier: FloatProvider,
    pub floor_level: FloatProvider,
}

impl CaveCarverConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.carver.CaveCarverConfiguration#<init>(float,HeightProvider,FloatProvider,VerticalAnchor,CarverDebugSettings,HolderSet,FloatProvider,FloatProvider,FloatProvider)
    pub fn new(
        probability: f32,
        y: HeightProvider,
        y_scale: FloatProvider,
        lava_level: VerticalAnchor,
        debug_settings: CarverDebugSettings,
        replaceable: HolderSet<Block>,
        horizontal_radius_multiplier: FloatProvider,
        vertical_radius_multiplier: FloatProvider,
        floor_level: FloatProvider,
    ) -> Self {
        Self {
            base: CarverConfiguration::new(
                probability,
                y,
                y_scale,
                lava_level,
                debug_settings,
                replaceable,
            ),
            horizontal_radius_multiplier,
            vertical_radius_multiplier,
            floor_level,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.CaveCarverConfiguration#<init>(float,HeightProvider,FloatProvider,VerticalAnchor,HolderSet,FloatProvider,FloatProvider,FloatProvider)
    pub fn new_with_default_debug(
        probability: f32,
        y: HeightProvider,
        y_scale: FloatProvider,
        lava_level: VerticalAnchor,
        replaceable: HolderSet<Block>,
        horizontal_radius_multiplier: FloatProvider,
        vertical_radius_multiplier: FloatProvider,
        floor_level: FloatProvider,
    ) -> Self {
        Self::new(
            probability,
            y,
            y_scale,
            lava_level,
            CarverDebugSettings::DEFAULT,
            replaceable,
            horizontal_radius_multiplier,
            vertical_radius_multiplier,
            floor_level,
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.CaveCarverConfiguration#<init>(CarverConfiguration,FloatProvider,FloatProvider,FloatProvider)
    pub fn from_base(
        base: CarverConfiguration,
        horizontal_radius_multiplier: FloatProvider,
        vertical_radius_multiplier: FloatProvider,
        floor_level: FloatProvider,
    ) -> Self {
        Self::new(
            base.probability,
            base.y,
            base.y_scale,
            base.lava_level,
            base.debug_settings,
            base.replaceable,
            horizontal_radius_multiplier,
            vertical_radius_multiplier,
            floor_level,
        )
    }
}

impl std::ops::Deref for CaveCarverConfiguration {
    type Target = CarverConfiguration;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_base() -> CarverConfiguration {
        CarverConfiguration::new(
            0.15f32,
            HeightProvider::uniform(-64, 128),
            FloatProvider::constant(1.0),
            VerticalAnchor::absolute(-54),
            CarverDebugSettings::DEFAULT,
            HolderSet::<Block>::empty(),
        )
    }

    #[test]
    fn full_constructor_preserves_all_fields() {
        let cfg = CaveCarverConfiguration::new(
            0.25f32,
            HeightProvider::uniform(-64, 192),
            FloatProvider::uniform(0.5, 1.5),
            VerticalAnchor::absolute(-54),
            CarverDebugSettings {
                debug_mode: true,
                air_state: 1,
                water_state: 2,
                lava_state: 3,
                barrier_state: 4,
            },
            HolderSet::<Block>::empty(),
            FloatProvider::constant(1.0),
            FloatProvider::constant(3.0),
            FloatProvider::constant(-0.7),
        );

        assert_eq!(cfg.probability.to_bits(), 0.25f32.to_bits());
        assert_eq!(cfg.y, HeightProvider::uniform(-64, 192));
        assert_eq!(cfg.y_scale, FloatProvider::uniform(0.5, 1.5));
        assert_eq!(cfg.lava_level, VerticalAnchor::absolute(-54));
        assert!(cfg.debug_settings.debug_mode);
        assert_eq!(cfg.debug_settings.barrier_state, 4);
        assert_eq!(cfg.horizontal_radius_multiplier.to_bits(), 1.0f32.to_bits());
        assert_eq!(cfg.vertical_radius_multiplier.to_bits(), 3.0f32.to_bits());
        assert_eq!(cfg.floor_level.to_bits(), (-0.7f32).to_bits());
    }

    #[test]
    fn default_debug_constructor_uses_default_settings() {
        let cfg = CaveCarverConfiguration::new_with_default_debug(
            0.1f32,
            HeightProvider::uniform(0, 64),
            FloatProvider::constant(2.0),
            VerticalAnchor::absolute(8),
            HolderSet::<Block>::empty(),
            FloatProvider::constant(0.0),
            FloatProvider::constant(-1.0),
            FloatProvider::constant(1.0),
        );

        assert_eq!(cfg.debug_settings, CarverDebugSettings::DEFAULT);
        assert_eq!(cfg.horizontal_radius_multiplier.to_bits(), 0.0f32.to_bits());
        assert_eq!(
            cfg.vertical_radius_multiplier.to_bits(),
            (-1.0f32).to_bits()
        );
        assert_eq!(cfg.floor_level.to_bits(), 1.0f32.to_bits());
    }

    #[test]
    fn from_base_copies_base_fields() {
        let base = sample_base();
        let cfg = CaveCarverConfiguration::from_base(
            base.clone(),
            FloatProvider::uniform(0.75, 1.25),
            FloatProvider::constant(2.5),
            FloatProvider::constant(-1.0),
        );

        assert_eq!(cfg.probability.to_bits(), base.probability.to_bits());
        assert_eq!(cfg.y, base.y);
        assert_eq!(cfg.y_scale, base.y_scale);
        assert_eq!(cfg.lava_level, base.lava_level);
        assert_eq!(cfg.debug_settings, base.debug_settings);
        assert_eq!(cfg.replaceable, base.replaceable);
        assert_eq!(cfg.horizontal_radius_multiplier, FloatProvider::uniform(0.75, 1.25));
        assert_eq!(cfg.vertical_radius_multiplier.to_bits(), 2.5f32.to_bits());
        assert_eq!(cfg.floor_level.to_bits(), (-1.0f32).to_bits());
    }

    #[test]
    fn float_bits_preserve_negative_zero_and_nan_payload() {
        let neg_zero = FloatProvider::constant(-0.0f32);
        let nan = FloatProvider::constant(f32::from_bits(0x7FC0_1234));
        let cfg = CaveCarverConfiguration::from_base(
            sample_base(),
            neg_zero,
            nan,
            FloatProvider::constant(f32::NEG_INFINITY),
        );

        assert_eq!(
            cfg.horizontal_radius_multiplier.to_bits(),
            (-0.0f32).to_bits()
        );
        assert_ne!(
            cfg.horizontal_radius_multiplier.to_bits(),
            0.0f32.to_bits()
        );
        assert_eq!(cfg.vertical_radius_multiplier.to_bits(), 0x7FC0_1234);
        assert_eq!(cfg.floor_level.to_bits(), f32::NEG_INFINITY.to_bits());
    }

    #[test]
    fn deref_exposes_base_configuration_fields() {
        let cfg = CaveCarverConfiguration::from_base(
            sample_base(),
            FloatProvider::constant(1.0),
            FloatProvider::constant(1.0),
            FloatProvider::constant(0.0),
        );

        let base: &CarverConfiguration = &cfg;
        assert_eq!(base.probability.to_bits(), cfg.probability.to_bits());
        assert_eq!(base.y, cfg.y);
        assert_eq!(base.lava_level, cfg.lava_level);
    }
}
