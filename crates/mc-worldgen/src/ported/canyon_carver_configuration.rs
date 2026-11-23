// Self-contained stubs for unported dependencies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CarverDebugSettings {
    pub debug_mode: bool,
    pub air_state: BlockState,
    pub water_state: BlockState,
    pub lava_state: BlockState,
    pub barrier_state: BlockState,
}

impl CarverDebugSettings {
    fn default_settings() -> Self {
        Self {
            debug_mode: false,
            air_state: BlockState {
                name: "minecraft:acacia_button".to_string(),
            },
            water_state: BlockState {
                name: "minecraft:candle".to_string(),
            },
            lava_state: BlockState {
                name: "minecraft:orange_stained_glass".to_string(),
            },
            barrier_state: BlockState {
                name: "minecraft:glass".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatProvider {
    pub min: f32,
    pub max: f32,
}

impl FloatProvider {
    pub fn constant(value: f32) -> Self {
        Self {
            min: value,
            max: value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HeightProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VerticalAnchor;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HolderSetBlock {
    pub entries: Vec<String>,
}

/// Minimal self-contained base carver configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct CarverConfiguration {
    pub probability: f32,
    pub y: HeightProvider,
    pub y_scale: FloatProvider,
    pub lava_level: VerticalAnchor,
    pub debug_settings: CarverDebugSettings,
    pub replaceable: HolderSetBlock,
}

/// Port of `net.minecraft.world.level.levelgen.carver.CanyonCarverConfiguration.CanyonShapeConfiguration`.
#[derive(Debug, Clone, PartialEq)]
pub struct CanyonShapeConfiguration {
    pub distance_factor: FloatProvider,
    pub thickness: FloatProvider,
    pub width_smoothness: i32,
    pub horizontal_radius_factor: FloatProvider,
    pub vertical_radius_default_factor: f32,
    pub vertical_radius_center_factor: f32,
}

impl CanyonShapeConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.carver.CanyonCarverConfiguration$CanyonShapeConfiguration#<init>(FloatProvider, FloatProvider, int, FloatProvider, float, float)
    pub fn new(
        distance_factor: FloatProvider,
        thickness: FloatProvider,
        width_smoothness: i32,
        horizontal_radius_factor: FloatProvider,
        vertical_radius_default_factor: f32,
        vertical_radius_center_factor: f32,
    ) -> Self {
        Self {
            width_smoothness,
            horizontal_radius_factor,
            vertical_radius_default_factor,
            vertical_radius_center_factor,
            distance_factor,
            thickness,
        }
    }
}

/// Port of `net.minecraft.world.level.levelgen.carver.CanyonCarverConfiguration`.
#[derive(Debug, Clone, PartialEq)]
pub struct CanyonCarverConfiguration {
    pub base: CarverConfiguration,
    pub vertical_rotation: FloatProvider,
    pub shape: CanyonShapeConfiguration,
}

impl CanyonCarverConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.carver.CanyonCarverConfiguration#<init>(float, HeightProvider, FloatProvider, VerticalAnchor, CarverDebugSettings, HolderSet, FloatProvider, CanyonCarverConfiguration.CanyonShapeConfiguration)
    pub fn new(
        probability: f32,
        y: HeightProvider,
        y_scale: FloatProvider,
        lava_level: VerticalAnchor,
        debug_settings: CarverDebugSettings,
        replaceable: HolderSetBlock,
        vertical_rotation: FloatProvider,
        shape: CanyonShapeConfiguration,
    ) -> Self {
        Self {
            base: CarverConfiguration {
                probability,
                y,
                y_scale,
                lava_level,
                debug_settings,
                replaceable,
            },
            vertical_rotation,
            shape,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.carver.CanyonCarverConfiguration#<init>(CarverConfiguration, FloatProvider, CanyonCarverConfiguration.CanyonShapeConfiguration)
    pub fn from_base(
        base: CarverConfiguration,
        vertical_rotation: FloatProvider,
        shape: CanyonShapeConfiguration,
    ) -> Self {
        Self::new(
            base.probability,
            base.y,
            base.y_scale,
            base.lava_level,
            base.debug_settings,
            base.replaceable,
            vertical_rotation,
            shape,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_shape() -> CanyonShapeConfiguration {
        CanyonShapeConfiguration::new(
            FloatProvider::constant(1.0),
            FloatProvider::constant(3.0),
            2,
            FloatProvider::constant(1.0),
            1.0,
            0.0,
        )
    }

    fn sample_base() -> CarverConfiguration {
        CarverConfiguration {
            probability: 0.02,
            y: HeightProvider,
            y_scale: FloatProvider::constant(1.0),
            lava_level: VerticalAnchor,
            debug_settings: CarverDebugSettings::default_settings(),
            replaceable: HolderSetBlock::default(),
        }
    }

    #[test]
    fn shape_configuration_sets_fields() {
        let shape = sample_shape();
        assert_eq!(shape.distance_factor, FloatProvider::constant(1.0));
        assert_eq!(shape.thickness, FloatProvider::constant(3.0));
        assert_eq!(shape.width_smoothness, 2);
        assert_eq!(shape.horizontal_radius_factor, FloatProvider::constant(1.0));
        assert_eq!(shape.vertical_radius_default_factor.to_bits(), 1.0f32.to_bits());
        assert_eq!(shape.vertical_radius_center_factor.to_bits(), 0.0f32.to_bits());
    }

    #[test]
    fn new_sets_all_fields() {
        let shape = sample_shape();
        let cfg = CanyonCarverConfiguration::new(
            0.01,
            HeightProvider,
            FloatProvider::constant(1.0),
            VerticalAnchor,
            CarverDebugSettings::default_settings(),
            HolderSetBlock {
                entries: vec!["minecraft:stone".to_string()],
            },
            FloatProvider::constant(0.0),
            shape.clone(),
        );
        assert_eq!(cfg.base.probability.to_bits(), 0.01f32.to_bits());
        assert_eq!(cfg.vertical_rotation, FloatProvider::constant(0.0));
        assert_eq!(cfg.shape, shape);
        assert_eq!(cfg.base.replaceable.entries.len(), 1);
    }

    #[test]
    fn from_base_copies_base_fields() {
        let base = sample_base();
        let shape = sample_shape();
        let cfg = CanyonCarverConfiguration::from_base(
            base.clone(),
            FloatProvider::constant(0.5),
            shape.clone(),
        );
        assert_eq!(cfg.base.probability.to_bits(), base.probability.to_bits());
        assert_eq!(cfg.base.y_scale, base.y_scale);
        assert_eq!(cfg.base.debug_settings, base.debug_settings);
        assert_eq!(cfg.vertical_rotation, FloatProvider::constant(0.5));
        assert_eq!(cfg.shape, shape);
    }

    #[test]
    fn shape_preserves_negative_and_special_float_bits() {
        let shape = CanyonShapeConfiguration::new(
            FloatProvider::constant(-1.0),
            FloatProvider::constant(f32::INFINITY),
            -7,
            FloatProvider::constant(0.0),
            -0.0,
            f32::NEG_INFINITY,
        );
        assert_eq!(shape.width_smoothness, -7);
        assert_eq!(shape.distance_factor.min.to_bits(), (-1.0f32).to_bits());
        assert_eq!(shape.thickness.min.to_bits(), f32::INFINITY.to_bits());
        assert_eq!(
            shape.vertical_radius_default_factor.to_bits(),
            (-0.0f32).to_bits()
        );
        assert_eq!(
            shape.vertical_radius_center_factor.to_bits(),
            f32::NEG_INFINITY.to_bits()
        );
    }
}
