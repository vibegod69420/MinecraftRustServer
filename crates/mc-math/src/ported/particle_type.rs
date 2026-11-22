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

impl<T> Default for MapCodec<T> {
    fn default() -> Self {
        Self::placeholder()
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

impl<B, T> Default for StreamCodec<B, T> {
    fn default() -> Self {
        Self::placeholder()
    }
}

/// Marker for unported `net.minecraft.core.particles.ParticleOptions`.
pub trait ParticleOptions {}

/// Faithful base for `net.minecraft.core.particles.ParticleType`.
/// Abstract codec methods are represented by [`ParticleTypeCodec`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParticleType<T: ParticleOptions + ?Sized> {
    override_limiter: bool,
    _phantom: PhantomData<fn() -> T>,
}

impl<T: ParticleOptions + ?Sized> ParticleType<T> {
    // JAVA: net.minecraft.core.particles.ParticleType#<init>
    pub fn new(override_limiter: bool) -> Self {
        Self {
            override_limiter,
            _phantom: PhantomData,
        }
    }

    // JAVA: net.minecraft.core.particles.ParticleType#getOverrideLimiter
    pub fn get_override_limiter(&self) -> bool {
        self.override_limiter
    }
}

/// Abstract codec contract from `ParticleType`.
pub trait ParticleTypeCodec<T: ParticleOptions> {
    // JAVA: net.minecraft.core.particles.ParticleType#codec
    fn codec(&self) -> &MapCodec<T>;

    // JAVA: net.minecraft.core.particles.ParticleType#streamCodec
    fn stream_codec(&self) -> &StreamCodec<RegistryFriendlyByteBuf, T>;
}

#[cfg(test)]
mod tests {
    use super::{ParticleOptions, ParticleType};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct DummyOption;
    impl ParticleOptions for DummyOption {}

    #[test]
    fn override_limiter_true() {
        let pt: ParticleType<DummyOption> = ParticleType::new(true);
        assert!(pt.get_override_limiter());
    }

    #[test]
    fn override_limiter_false() {
        let pt: ParticleType<DummyOption> = ParticleType::new(false);
        assert!(!pt.get_override_limiter());
    }

    #[test]
    fn copy_and_eq() {
        let a: ParticleType<DummyOption> = ParticleType::new(true);
        let b = a;
        assert_eq!(a, b);
        let c: ParticleType<DummyOption> = ParticleType::new(false);
        assert_ne!(a, c);
    }
}
