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
    pub fn unit() -> Self {
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
    pub fn unit() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

/// Marker for unported `net.minecraft.core.particles.ParticleOptions`.
pub trait ParticleOptions {}

#[derive(Debug, Clone)]
pub struct SimpleParticleType {
    override_limiter: bool,
    codec: MapCodec<SimpleParticleType>,
    stream_codec: StreamCodec<RegistryFriendlyByteBuf, SimpleParticleType>,
}

impl SimpleParticleType {
    // JAVA: net.minecraft.core.particles.SimpleParticleType#<init>
    pub fn new(override_limiter: bool) -> Self {
        Self {
            override_limiter,
            codec: MapCodec::unit(),
            stream_codec: StreamCodec::unit(),
        }
    }

    // JAVA: net.minecraft.core.particles.SimpleParticleType#getType
    pub fn get_type(&self) -> &Self {
        self
    }

    // JAVA: net.minecraft.core.particles.SimpleParticleType#codec
    pub fn codec(&self) -> &MapCodec<SimpleParticleType> {
        &self.codec
    }

    // JAVA: net.minecraft.core.particles.SimpleParticleType#streamCodec
    pub fn stream_codec(&self) -> &StreamCodec<RegistryFriendlyByteBuf, SimpleParticleType> {
        &self.stream_codec
    }

    // JAVA: net.minecraft.core.particles.ParticleType#getOverrideLimiter
    pub fn get_override_limiter(&self) -> bool {
        self.override_limiter
    }
}

impl ParticleOptions for SimpleParticleType {}

#[cfg(test)]
mod tests {
    use super::SimpleParticleType;

    #[test]
    fn override_limiter_flags() {
        let t = SimpleParticleType::new(true);
        assert!(t.get_override_limiter());
        let f = SimpleParticleType::new(false);
        assert!(!f.get_override_limiter());
    }

    #[test]
    fn get_type_returns_self_reference() {
        let t = SimpleParticleType::new(false);
        let r = t.get_type();
        assert!(std::ptr::eq(&t, r));
    }

    #[test]
    fn codec_and_stream_codec_accessible() {
        let t = SimpleParticleType::new(true);
        let _c = t.codec();
        let _s = t.stream_codec();
        // Accessors return stable references to stored placeholders.
        assert!(std::ptr::eq(t.codec(), t.codec()));
        assert!(std::ptr::eq(t.stream_codec(), t.stream_codec()));
    }
}
