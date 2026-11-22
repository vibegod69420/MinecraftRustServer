#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExplosionParticleInfo<P> {
    particle: P,
    scaling: f32,
    speed: f32,
}

impl<P> ExplosionParticleInfo<P> {
    // JAVA: net.minecraft.core.particles.ExplosionParticleInfo#<init>
    pub fn new(particle: P, scaling: f32, speed: f32) -> Self {
        Self {
            particle,
            scaling,
            speed,
        }
    }

    // JAVA: net.minecraft.core.particles.ExplosionParticleInfo#particle
    pub fn particle(&self) -> &P {
        &self.particle
    }

    // JAVA: net.minecraft.core.particles.ExplosionParticleInfo#scaling
    pub fn scaling(&self) -> f32 {
        self.scaling
    }

    // JAVA: net.minecraft.core.particles.ExplosionParticleInfo#speed
    pub fn speed(&self) -> f32 {
        self.speed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct DummyParticle(u32);

    #[test]
    fn new_and_accessors() {
        let info = ExplosionParticleInfo::new(DummyParticle(7), 1.0, 2.0);
        assert_eq!(info.particle(), &DummyParticle(7));
        assert_eq!(info.scaling().to_bits(), 1.0f32.to_bits());
        assert_eq!(info.speed().to_bits(), 2.0f32.to_bits());
    }

    #[test]
    fn default_codec_like_values() {
        let info = ExplosionParticleInfo::new(DummyParticle(0), 1.0, 1.0);
        assert_eq!(info.scaling().to_bits(), 1.0f32.to_bits());
        assert_eq!(info.speed().to_bits(), 1.0f32.to_bits());
    }

    #[test]
    fn special_float_bits_preserved() {
        let nan = f32::from_bits(0x7FC0_0002);
        let neg_zero = f32::from_bits(0x8000_0000);
        let info = ExplosionParticleInfo::new(DummyParticle(1), nan, neg_zero);
        assert_eq!(info.scaling().to_bits(), nan.to_bits());
        assert_eq!(info.speed().to_bits(), neg_zero.to_bits());
    }

    #[test]
    fn extreme_and_negative_float_values() {
        let info = ExplosionParticleInfo::new(DummyParticle(2), f32::MAX, -f32::MIN_POSITIVE);
        assert_eq!(info.scaling().to_bits(), f32::MAX.to_bits());
        assert_eq!(info.speed().to_bits(), (-f32::MIN_POSITIVE).to_bits());
    }
}
