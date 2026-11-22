#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParticleLimit {
    limit: i32,
}

impl ParticleLimit {
    pub const SPORE_BLOSSOM: Self = Self { limit: 1000 };

    // JAVA: net.minecraft.core.particles.ParticleLimit#<init>
    pub fn new(limit: i32) -> Self {
        Self { limit }
    }

    // JAVA: net.minecraft.core.particles.ParticleLimit#limit
    pub fn limit(&self) -> i32 {
        self.limit
    }
}

#[cfg(test)]
mod tests {
    use super::ParticleLimit;

    #[test]
    fn spore_blossom_constant() {
        assert_eq!(ParticleLimit::SPORE_BLOSSOM.limit(), 1000);
        assert_eq!(ParticleLimit::SPORE_BLOSSOM, ParticleLimit::new(1000));
    }

    #[test]
    fn new_and_accessor() {
        let p = ParticleLimit::new(42);
        assert_eq!(p.limit(), 42);
    }

    #[test]
    fn negative_and_extreme_values() {
        let neg = ParticleLimit::new(-1);
        assert_eq!(neg.limit(), -1);
        let min = ParticleLimit::new(i32::MIN);
        assert_eq!(min.limit(), i32::MIN);
        let max = ParticleLimit::new(i32::MAX);
        assert_eq!(max.limit(), i32::MAX);
    }

    #[test]
    fn copy_eq_hash_semantics() {
        let a = ParticleLimit::new(7);
        let b = a;
        assert_eq!(a, b);
        assert_ne!(a, ParticleLimit::new(8));
    }
}
