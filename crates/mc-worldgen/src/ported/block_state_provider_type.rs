//! Self-contained port of `net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProviderType`.

use std::marker::PhantomData;
use std::sync::LazyLock;

pub trait BlockStateProvider {}

#[derive(Clone, Debug)]
pub struct MapCodec<P>(PhantomData<P>);

impl<P> MapCodec<P> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<P> Default for MapCodec<P> {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! stub_provider {
    ($name:ident) => {
        #[derive(Clone, Debug, Default)]
        pub struct $name;
        impl BlockStateProvider for $name {}
        impl $name {
            pub fn codec() -> MapCodec<$name> {
                MapCodec::new()
            }
        }
    };
}

stub_provider!(SimpleStateProvider);
stub_provider!(WeightedStateProvider);
stub_provider!(NoiseThresholdProvider);
stub_provider!(NoiseProvider);
stub_provider!(DualNoiseProvider);
stub_provider!(RotatedBlockProvider);
stub_provider!(RandomizedIntStateProvider);

pub struct BlockStateProviderType<P: BlockStateProvider> {
    codec: MapCodec<P>,
}

impl<P: BlockStateProvider> BlockStateProviderType<P> {
    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProviderType#register
    fn register(name: &str, codec: MapCodec<P>) -> Self {
        let _ = name;
        Self::new(codec)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProviderType#<init>
    fn new(codec: MapCodec<P>) -> Self {
        Self { codec }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProviderType#codec
    pub fn codec(&self) -> &MapCodec<P> {
        &self.codec
    }
}

pub static SIMPLE_STATE_PROVIDER: LazyLock<BlockStateProviderType<SimpleStateProvider>> =
    LazyLock::new(|| BlockStateProviderType::register("simple_state_provider", SimpleStateProvider::codec()));
pub static WEIGHTED_STATE_PROVIDER: LazyLock<BlockStateProviderType<WeightedStateProvider>> =
    LazyLock::new(|| BlockStateProviderType::register("weighted_state_provider", WeightedStateProvider::codec()));
pub static NOISE_THRESHOLD_PROVIDER: LazyLock<BlockStateProviderType<NoiseThresholdProvider>> =
    LazyLock::new(|| BlockStateProviderType::register("noise_threshold_provider", NoiseThresholdProvider::codec()));
pub static NOISE_PROVIDER: LazyLock<BlockStateProviderType<NoiseProvider>> =
    LazyLock::new(|| BlockStateProviderType::register("noise_provider", NoiseProvider::codec()));
pub static DUAL_NOISE_PROVIDER: LazyLock<BlockStateProviderType<DualNoiseProvider>> =
    LazyLock::new(|| BlockStateProviderType::register("dual_noise_provider", DualNoiseProvider::codec()));
pub static ROTATED_BLOCK_PROVIDER: LazyLock<BlockStateProviderType<RotatedBlockProvider>> =
    LazyLock::new(|| BlockStateProviderType::register("rotated_block_provider", RotatedBlockProvider::codec()));
pub static RANDOMIZED_INT_STATE_PROVIDER: LazyLock<BlockStateProviderType<RandomizedIntStateProvider>> =
    LazyLock::new(|| {
        BlockStateProviderType::register(
            "randomized_int_state_provider",
            RandomizedIntStateProvider::codec(),
        )
    });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_codec_accessible() {
        let t = BlockStateProviderType::register("test", MapCodec::<SimpleStateProvider>::new());
        let _codec: &MapCodec<SimpleStateProvider> = t.codec();
    }

    #[test]
    fn constants_are_initialized() {
        let _ = &*SIMPLE_STATE_PROVIDER;
        let _ = &*WEIGHTED_STATE_PROVIDER;
        let _ = &*NOISE_THRESHOLD_PROVIDER;
        let _ = &*NOISE_PROVIDER;
        let _ = &*DUAL_NOISE_PROVIDER;
        let _ = &*ROTATED_BLOCK_PROVIDER;
        let _ = &*RANDOMIZED_INT_STATE_PROVIDER;
    }

    #[test]
    fn codec_getter_returns_codec_reference() {
        let c1: &MapCodec<SimpleStateProvider> = SIMPLE_STATE_PROVIDER.codec();
        let c2: &MapCodec<SimpleStateProvider> = SIMPLE_STATE_PROVIDER.codec();
        assert!(std::ptr::eq(c1, c2));
    }

    #[test]
    fn different_provider_types_are_distinct() {
        fn type_name_of<T>(_: &T) -> &'static str {
            std::any::type_name::<T>()
        }
        assert_ne!(
            type_name_of(&*SIMPLE_STATE_PROVIDER),
            type_name_of(&*NOISE_PROVIDER)
        );
    }
}
