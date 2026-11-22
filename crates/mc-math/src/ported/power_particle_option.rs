#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParticleType {
    pub id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PowerParticleOption {
    particle_type: ParticleType,
    power: f32,
}

impl PowerParticleOption {
    // JAVA: net.minecraft.core.particles.PowerParticleOption#<init>
    fn new(particle_type: ParticleType, power: f32) -> Self {
        Self {
            particle_type,
            power,
        }
    }

    // JAVA: net.minecraft.core.particles.PowerParticleOption#getType
    pub fn get_type(&self) -> ParticleType {
        self.particle_type
    }

    // JAVA: net.minecraft.core.particles.PowerParticleOption#getPower
    pub fn get_power(&self) -> f32 {
        self.power
    }

    // JAVA: net.minecraft.core.particles.PowerParticleOption#create
    pub fn create(particle_type: ParticleType, power: f32) -> Self {
        Self::new(particle_type, power)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pt(id: u32) -> ParticleType {
        ParticleType { id }
    }

    #[test]
    fn create_and_getters_roundtrip() {
        let opt = PowerParticleOption::create(pt(7), 1.5);
        assert_eq!(opt.get_type(), pt(7));
        assert_eq!(opt.get_power().to_bits(), 1.5f32.to_bits());
    }

    #[test]
    fn default_power_one_preserves_bits() {
        let opt = PowerParticleOption::create(pt(0), 1.0);
        assert_eq!(opt.get_power().to_bits(), 1.0f32.to_bits());
    }

    #[test]
    fn negative_power_preserves_bits() {
        let opt = PowerParticleOption::create(pt(1), -2.25);
        assert_eq!(opt.get_power().to_bits(), (-2.25f32).to_bits());
    }

    #[test]
    fn nan_power_preserves_bits() {
        let nan = f32::from_bits(0x7FC0_0001);
        let opt = PowerParticleOption::create(pt(2), nan);
        assert_eq!(opt.get_power().to_bits(), nan.to_bits());
    }

    #[test]
    fn infinite_power_values() {
        let pos_inf = PowerParticleOption::create(pt(3), f32::INFINITY);
        let neg_inf = PowerParticleOption::create(pt(3), f32::NEG_INFINITY);
        assert_eq!(pos_inf.get_power().to_bits(), f32::INFINITY.to_bits());
        assert_eq!(neg_inf.get_power().to_bits(), f32::NEG_INFINITY.to_bits());
    }
}
