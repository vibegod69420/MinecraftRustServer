use crate::mth::clamp_f32;

pub const MIN_SCALE: f32 = 0.01;
pub const MAX_SCALE: f32 = 4.0;

/// Marker for unported `net.minecraft.core.particles.ParticleOptions`.
pub trait ParticleOptions {}

/// Validates the SCALE codec constraint from Java:
/// value must be within range [0.01;4.0].
pub fn validate_scale(value: f32) -> Result<f32, String> {
    if value >= MIN_SCALE && value <= MAX_SCALE {
        Ok(value)
    } else {
        Err(format!("Value must be within range [0.01;4.0]: {}", value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScalableParticleOptionsBase {
    scale: f32,
}

impl ScalableParticleOptionsBase {
    // JAVA: net.minecraft.core.particles.ScalableParticleOptionsBase#<init>
    pub fn new(scale: f32) -> Self {
        Self {
            scale: clamp_f32(scale, MIN_SCALE, MAX_SCALE),
        }
    }

    // JAVA: net.minecraft.core.particles.ScalableParticleOptionsBase#getScale
    pub fn get_scale(&self) -> f32 {
        self.scale
    }
}

impl ParticleOptions for ScalableParticleOptionsBase {}

#[cfg(test)]
mod tests {
    use super::{validate_scale, ScalableParticleOptionsBase, MAX_SCALE, MIN_SCALE};

    #[test]
    fn clamps_below_min() {
        let o = ScalableParticleOptionsBase::new(0.0);
        assert_eq!(o.get_scale().to_bits(), MIN_SCALE.to_bits());
        let o2 = ScalableParticleOptionsBase::new(-100.0);
        assert_eq!(o2.get_scale().to_bits(), MIN_SCALE.to_bits());
    }

    #[test]
    fn clamps_above_max() {
        let o = ScalableParticleOptionsBase::new(10.0);
        assert_eq!(o.get_scale().to_bits(), MAX_SCALE.to_bits());
    }

    #[test]
    fn preserves_in_range_value_bits() {
        let v = 1.5_f32;
        let o = ScalableParticleOptionsBase::new(v);
        assert_eq!(o.get_scale().to_bits(), v.to_bits());
    }

    #[test]
    fn exact_boundaries() {
        let min = ScalableParticleOptionsBase::new(MIN_SCALE);
        assert_eq!(min.get_scale().to_bits(), MIN_SCALE.to_bits());
        let max = ScalableParticleOptionsBase::new(MAX_SCALE);
        assert_eq!(max.get_scale().to_bits(), MAX_SCALE.to_bits());
    }

    #[test]
    fn validate_scale_matches_java_constraint() {
        assert_eq!(validate_scale(1.0).unwrap().to_bits(), 1.0_f32.to_bits());
        assert!(validate_scale(0.009).is_err());
        assert!(validate_scale(4.001).is_err());
        assert!(validate_scale(MIN_SCALE).is_ok());
        assert!(validate_scale(MAX_SCALE).is_ok());
    }
}
