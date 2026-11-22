#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParticleType {
    pub id: u32,
}

pub const TRAIL_PARTICLE_TYPE: ParticleType = ParticleType { id: 1 };

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrailParticleOption {
    target: Vec3,
    color: i32,
    duration: i32,
}

impl TrailParticleOption {
    // JAVA: net.minecraft.core.particles.TrailParticleOption#<init>
    pub fn new(target: Vec3, color: i32, duration: i32) -> Self {
        Self {
            target,
            color,
            duration,
        }
    }

    // JAVA: net.minecraft.core.particles.TrailParticleOption#target
    pub fn target(&self) -> Vec3 {
        self.target
    }

    // JAVA: net.minecraft.core.particles.TrailParticleOption#color
    pub fn color(&self) -> i32 {
        self.color
    }

    // JAVA: net.minecraft.core.particles.TrailParticleOption#duration
    pub fn duration(&self) -> i32 {
        self.duration
    }

    // JAVA: net.minecraft.core.particles.TrailParticleOption#getType
    pub fn get_type(&self) -> ParticleType {
        TRAIL_PARTICLE_TYPE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_accessors() {
        let target = Vec3::new(1.0, 2.0, 3.0);
        let opt = TrailParticleOption::new(target, 0x00FF_00FF, 20);
        assert_eq!(opt.target().x.to_bits(), 1.0f64.to_bits());
        assert_eq!(opt.target().y.to_bits(), 2.0f64.to_bits());
        assert_eq!(opt.target().z.to_bits(), 3.0f64.to_bits());
        assert_eq!(opt.color(), 0x00FF_00FF);
        assert_eq!(opt.duration(), 20);
    }

    #[test]
    fn get_type_returns_trail() {
        let opt = TrailParticleOption::new(Vec3::new(0.0, 0.0, 0.0), 0, 1);
        assert_eq!(opt.get_type(), TRAIL_PARTICLE_TYPE);
    }

    #[test]
    fn negative_and_extreme_int_values() {
        let opt = TrailParticleOption::new(Vec3::new(-1.0, 0.0, 1.0), -1, i32::MIN);
        assert_eq!(opt.color(), -1);
        assert_eq!(opt.duration(), i32::MIN);

        let opt_max = TrailParticleOption::new(Vec3::new(0.0, 0.0, 0.0), i32::MAX, i32::MAX);
        assert_eq!(opt_max.color(), i32::MAX);
        assert_eq!(opt_max.duration(), i32::MAX);
    }

    #[test]
    fn target_float_bits_preserved() {
        let nan = f64::from_bits(0x7FF8_0000_0000_0001);
        let neg_zero = f64::from_bits(0x8000_0000_0000_0000);
        let opt = TrailParticleOption::new(Vec3::new(nan, neg_zero, f64::INFINITY), 0, 0);
        assert_eq!(opt.target().x.to_bits(), nan.to_bits());
        assert_eq!(opt.target().y.to_bits(), neg_zero.to_bits());
        assert_eq!(opt.target().z.to_bits(), f64::INFINITY.to_bits());
    }
}
