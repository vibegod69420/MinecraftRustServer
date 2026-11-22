//! Port of `net.minecraft.world.level.levelgen.heightproviders.HeightProvider`.
//!
//! Codec and registry infrastructure are represented as self-contained placeholders.

use std::marker::PhantomData;

/// Placeholder for Mojang serialization `Codec<T>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Codec<T>(PhantomData<fn() -> T>);

impl<T> Codec<T> {
    pub const fn placeholder() -> Self {
        Self(PhantomData)
    }
}

/// Placeholder for `CONSTANT_OR_DISPATCH_CODEC`.
pub const CONSTANT_OR_DISPATCH_CODEC: Codec<()> = Codec::placeholder();
/// Placeholder for `CODEC`.
pub const CODEC: Codec<()> = Codec::placeholder();

/// Minimal world generation context stub for self-contained compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldGenerationContext {
    pub min_gen_y: i32,
    pub gen_depth: i32,
}

/// Minimal random source stub for self-contained compilation.
pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

impl<R: RandomSource + ?Sized> RandomSource for &mut R {
    fn next_int(&mut self, bound: i32) -> i32 {
        (**self).next_int(bound)
    }
}

/// Minimal height provider type identifier for self-contained compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeightProviderType {
    Constant,
    Uniform,
    BiasedToBottom,
    VeryBiasedToBottom,
    Trapezoid,
    WeightedList,
}

/// Port of `net.minecraft.world.level.levelgen.heightproviders.HeightProvider`.
pub trait HeightProvider {
    // JAVA: net.minecraft.world.level.levelgen.heightproviders.HeightProvider#sample
    fn sample(&self, random: &mut dyn RandomSource, context: &WorldGenerationContext) -> i32;

    // JAVA: net.minecraft.world.level.levelgen.heightproviders.HeightProvider#getType
    fn get_type(&self) -> HeightProviderType;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedHeight(i32);

    impl HeightProvider for FixedHeight {
        fn sample(
            &self,
            _random: &mut dyn RandomSource,
            _context: &WorldGenerationContext,
        ) -> i32 {
            self.0
        }

        fn get_type(&self) -> HeightProviderType {
            HeightProviderType::Constant
        }
    }

    struct DummyRandom;

    impl RandomSource for DummyRandom {
        fn next_int(&mut self, _bound: i32) -> i32 {
            0
        }
    }

    fn ctx() -> WorldGenerationContext {
        WorldGenerationContext {
            min_gen_y: -64,
            gen_depth: 384,
        }
    }

    #[test]
    fn trait_sample_and_type() {
        let p = FixedHeight(7);
        let mut rng = DummyRandom;
        assert_eq!(p.sample(&mut rng, &ctx()), 7);
        assert_eq!(p.get_type(), HeightProviderType::Constant);
    }

    #[test]
    fn sample_negative_and_extreme_values() {
        let mut rng = DummyRandom;
        let c = ctx();
        assert_eq!(FixedHeight(i32::MIN).sample(&mut rng, &c), i32::MIN);
        assert_eq!(FixedHeight(i32::MAX).sample(&mut rng, &c), i32::MAX);
        assert_eq!(FixedHeight(-1).sample(&mut rng, &c), -1);
    }

    #[test]
    fn codec_placeholders_exist() {
        let _dispatch = CONSTANT_OR_DISPATCH_CODEC;
        let _codec = CODEC;
        assert_eq!(CONSTANT_OR_DISPATCH_CODEC, Codec::<()>::placeholder());
        assert_eq!(CODEC, Codec::<()>::placeholder());
    }

    #[test]
    fn height_provider_type_equality() {
        assert_eq!(HeightProviderType::Constant, HeightProviderType::Constant);
        assert_ne!(HeightProviderType::Constant, HeightProviderType::WeightedList);
    }
}
