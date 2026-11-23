#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Block {
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderSet<T> {
    pub entries: Vec<T>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeightProvider {
    pub min: i32,
    pub max: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatProvider {
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerticalAnchor {
    pub value: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarverDebugSettings {
    pub debug_mode: bool,
}

impl CarverDebugSettings {
    pub const DEFAULT: Self = Self { debug_mode: false };
}

impl Default for CarverDebugSettings {
    fn default() -> Self {
        Self::DEFAULT
    }
}

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
    // JAVA: net.minecraft.world.level.levelgen.carver.CarverConfiguration#<init>
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config(probability: f32) -> CarverConfiguration {
        CarverConfiguration::new(
            probability,
            HeightProvider { min: -64, max: 128 },
            FloatProvider { min: 0.0, max: 1.0 },
            VerticalAnchor { value: -56 },
            CarverDebugSettings::DEFAULT,
            HolderSet {
                entries: vec![Block { id: 1 }, Block { id: 2 }],
            },
        )
    }

    #[test]
    fn constructor_stores_all_fields() {
        let cfg = sample_config(0.1_f32);
        assert_eq!(cfg.probability.to_bits(), 0.1_f32.to_bits());
        assert_eq!(cfg.y, HeightProvider { min: -64, max: 128 });
        assert_eq!(cfg.y_scale, FloatProvider { min: 0.0, max: 1.0 });
        assert_eq!(cfg.lava_level, VerticalAnchor { value: -56 });
        assert_eq!(cfg.debug_settings, CarverDebugSettings::DEFAULT);
        assert_eq!(cfg.replaceable.entries.len(), 2);
    }

    #[test]
    fn probability_float_bits_preserved() {
        let nan_bits = 0x7FC0_0001_u32;
        let nan = f32::from_bits(nan_bits);
        let cfg = sample_config(nan);
        assert_eq!(cfg.probability.to_bits(), nan_bits);

        let neg_zero = f32::from_bits(0x8000_0000);
        let cfg2 = sample_config(neg_zero);
        assert_eq!(cfg2.probability.to_bits(), 0x8000_0000);
        assert!(cfg2.probability.is_sign_negative());

        let inf = f32::INFINITY;
        let cfg3 = sample_config(inf);
        assert_eq!(cfg3.probability.to_bits(), f32::INFINITY.to_bits());
    }

    #[test]
    fn negative_and_extreme_integer_values() {
        let cfg = CarverConfiguration::new(
            -1.0_f32,
            HeightProvider {
                min: i32::MIN,
                max: i32::MAX,
            },
            FloatProvider {
                min: f32::NEG_INFINITY,
                max: f32::MAX,
            },
            VerticalAnchor { value: i32::MIN },
            CarverDebugSettings { debug_mode: true },
            HolderSet { entries: vec![] },
        );
        assert_eq!(cfg.y.min, i32::MIN);
        assert_eq!(cfg.y.max, i32::MAX);
        assert_eq!(cfg.lava_level.value, i32::MIN);
        assert!(cfg.debug_settings.debug_mode);
        assert!(cfg.replaceable.entries.is_empty());
        assert_eq!(cfg.probability.to_bits(), (-1.0_f32).to_bits());
    }

    #[test]
    fn y_scale_float_provider_preserves_bits() {
        let min = f32::from_bits(0x0000_0001);
        let max = f32::from_bits(0x7F7F_FFFF);
        let cfg = CarverConfiguration::new(
            0.5_f32,
            HeightProvider { min: 0, max: 0 },
            FloatProvider { min, max },
            VerticalAnchor { value: 0 },
            CarverDebugSettings::DEFAULT,
            HolderSet {
                entries: vec![Block { id: u32::MAX }],
            },
        );
        assert_eq!(cfg.y_scale.min.to_bits(), 0x0000_0001);
        assert_eq!(cfg.y_scale.max.to_bits(), 0x7F7F_FFFF);
        assert_eq!(cfg.replaceable.entries[0].id, u32::MAX);
    }
}
