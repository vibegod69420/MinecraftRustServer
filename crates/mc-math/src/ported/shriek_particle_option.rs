use std::marker::PhantomData;

/// Placeholder for unported `net.minecraft.network.RegistryFriendlyByteBuf`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RegistryFriendlyByteBuf;

/// Placeholder for unported `com.mojang.serialization.MapCodec`.
#[derive(Debug, Clone)]
pub struct MapCodec<T> {
    _phantom: PhantomData<fn() -> T>,
}

impl<T> MapCodec<T> {
    pub fn placeholder() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

/// Placeholder for unported `net.minecraft.network.codec.StreamCodec`.
#[derive(Debug, Clone)]
pub struct StreamCodec<B, T> {
    _phantom: PhantomData<fn(B) -> T>,
}

impl<B, T> StreamCodec<B, T> {
    pub fn placeholder() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

/// Marker for unported `net.minecraft.core.particles.ParticleOptions`.
pub trait ParticleOptions {}

/// Placeholder for unported `net.minecraft.core.particles.ParticleType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParticleTypeRef {
    pub id: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShriekParticleOption {
    delay: i32,
}

impl ShriekParticleOption {
    pub fn codec_placeholder() -> MapCodec<Self> {
        MapCodec::placeholder()
    }

    pub fn stream_codec_placeholder() -> StreamCodec<RegistryFriendlyByteBuf, Self> {
        StreamCodec::placeholder()
    }

    // JAVA: net.minecraft.core.particles.ShriekParticleOption#<init>
    pub fn new(delay: i32) -> Self {
        Self { delay }
    }

    // JAVA: net.minecraft.core.particles.ShriekParticleOption#getType
    pub fn get_type(&self) -> ParticleTypeRef {
        ParticleTypeRef {
            id: "minecraft:shriek",
        }
    }

    // JAVA: net.minecraft.core.particles.ShriekParticleOption#getDelay
    pub fn get_delay(&self) -> i32 {
        self.delay
    }
}

impl ParticleOptions for ShriekParticleOption {}

#[cfg(test)]
mod tests {
    use super::ShriekParticleOption;

    #[test]
    fn stores_delay() {
        let o = ShriekParticleOption::new(10);
        assert_eq!(o.get_delay(), 10);
    }

    #[test]
    fn negative_and_zero_delay() {
        assert_eq!(ShriekParticleOption::new(0).get_delay(), 0);
        assert_eq!(ShriekParticleOption::new(-5).get_delay(), -5);
    }

    #[test]
    fn extreme_delay_values() {
        assert_eq!(ShriekParticleOption::new(i32::MIN).get_delay(), i32::MIN);
        assert_eq!(ShriekParticleOption::new(i32::MAX).get_delay(), i32::MAX);
    }

    #[test]
    fn get_type_is_shriek() {
        let o = ShriekParticleOption::new(1);
        assert_eq!(o.get_type().id, "minecraft:shriek");
    }

    #[test]
    fn copy_and_eq() {
        let a = ShriekParticleOption::new(3);
        let b = a;
        assert_eq!(a, b);
        assert_ne!(a, ShriekParticleOption::new(4));
    }
}
