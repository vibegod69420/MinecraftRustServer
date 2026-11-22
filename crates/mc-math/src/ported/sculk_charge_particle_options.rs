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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SculkChargeParticleOptions {
    roll: f32,
}

impl SculkChargeParticleOptions {
    pub fn codec_placeholder() -> MapCodec<Self> {
        MapCodec::placeholder()
    }

    pub fn stream_codec_placeholder() -> StreamCodec<RegistryFriendlyByteBuf, Self> {
        StreamCodec::placeholder()
    }

    // JAVA: net.minecraft.core.particles.SculkChargeParticleOptions#<init>
    pub fn new(roll: f32) -> Self {
        Self { roll }
    }

    // JAVA: net.minecraft.core.particles.SculkChargeParticleOptions#roll
    pub fn roll(&self) -> f32 {
        self.roll
    }

    // JAVA: net.minecraft.core.particles.SculkChargeParticleOptions#getType
    pub fn get_type(&self) -> ParticleTypeRef {
        ParticleTypeRef {
            id: "minecraft:sculk_charge",
        }
    }
}

impl ParticleOptions for SculkChargeParticleOptions {}

#[cfg(test)]
mod tests {
    use super::SculkChargeParticleOptions;

    #[test]
    fn stores_roll() {
        let o = SculkChargeParticleOptions::new(1.25);
        assert_eq!(o.roll().to_bits(), 1.25_f32.to_bits());
    }

    #[test]
    fn negative_and_zero_roll() {
        let z = SculkChargeParticleOptions::new(0.0);
        assert_eq!(z.roll().to_bits(), 0.0_f32.to_bits());
        let n = SculkChargeParticleOptions::new(-3.5);
        assert_eq!(n.roll().to_bits(), (-3.5_f32).to_bits());
    }

    #[test]
    fn preserves_float_bits_for_nan() {
        let bits = 0x7FC0_0001_u32;
        let v = f32::from_bits(bits);
        let o = SculkChargeParticleOptions::new(v);
        assert_eq!(o.roll().to_bits(), bits);
    }

    #[test]
    fn get_type_is_sculk_charge() {
        let o = SculkChargeParticleOptions::new(0.0);
        assert_eq!(o.get_type().id, "minecraft:sculk_charge");
    }

    #[test]
    fn copy_and_eq() {
        let a = SculkChargeParticleOptions::new(2.0);
        let b = a;
        assert_eq!(a, b);
        assert_ne!(a, SculkChargeParticleOptions::new(2.1));
    }
}
