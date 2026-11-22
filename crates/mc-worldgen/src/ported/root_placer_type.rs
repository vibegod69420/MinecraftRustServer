use std::marker::PhantomData;
use std::sync::LazyLock;

/// Placeholder for `net.minecraft.world.level.levelgen.feature.rootplacers.RootPlacer`.
pub trait RootPlacer {}

/// Placeholder for an unported root placer referenced by `MANGROVE_ROOT_PLACER`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MangroveRootPlacer;

impl RootPlacer for MangroveRootPlacer {}

impl MangroveRootPlacer {
    /// Placeholder for `MangroveRootPlacer.CODEC`.
    pub fn codec() -> MapCodec<Self> {
        MapCodec::of("mangrove_root_placer")
    }
}

/// Placeholder for `com.mojang.serialization.MapCodec`.
#[derive(Debug, PartialEq, Eq)]
pub struct MapCodec<P: ?Sized> {
    name: &'static str,
    _marker: PhantomData<fn() -> P>,
}

impl<P: ?Sized> Clone for MapCodec<P> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            _marker: PhantomData,
        }
    }
}

impl<P: ?Sized> MapCodec<P> {
    /// Creates a named placeholder codec for self-contained testing/porting.
    pub fn of(name: &'static str) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }

    /// Returns the placeholder codec name.
    pub fn name(&self) -> &'static str {
        self.name
    }
}

/// Port of `net.minecraft.world.level.levelgen.feature.rootplacers.RootPlacerType`.
#[derive(Debug, Clone)]
pub struct RootPlacerType<P: RootPlacer> {
    codec: MapCodec<P>,
}

impl<P: RootPlacer> RootPlacerType<P> {
    // JAVA: net.minecraft.world.level.levelgen.feature.rootplacers.RootPlacerType#register
    fn register(_name: &str, codec: MapCodec<P>) -> Self {
        // Full Java registers into BuiltInRegistries.ROOT_PLACER_TYPE.
        // Registry side effects are omitted for self-containment.
        Self::new(codec)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.rootplacers.RootPlacerType#<init>
    fn new(codec: MapCodec<P>) -> Self {
        Self { codec }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.rootplacers.RootPlacerType#codec
    pub fn codec(&self) -> &MapCodec<P> {
        &self.codec
    }
}

// JAVA: net.minecraft.world.level.levelgen.feature.rootplacers.RootPlacerType#MANGROVE_ROOT_PLACER
pub static MANGROVE_ROOT_PLACER: LazyLock<RootPlacerType<MangroveRootPlacer>> =
    LazyLock::new(|| RootPlacerType::register("mangrove_root_placer", MangroveRootPlacer::codec()));

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct DummyPlacer;
    impl RootPlacer for DummyPlacer {}

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct OtherPlacer;
    impl RootPlacer for OtherPlacer {}

    #[test]
    fn register_and_codec_round_trip() {
        let codec = MapCodec::<DummyPlacer>::of("dummy");
        let placer_type = RootPlacerType::register("dummy", codec.clone());
        assert_eq!(placer_type.codec(), &codec);
        assert_eq!(placer_type.codec().name(), "dummy");
    }

    #[test]
    fn mangrove_root_placer_static_has_expected_codec() {
        assert_eq!(
            MANGROVE_ROOT_PLACER.codec().name(),
            "mangrove_root_placer"
        );
    }

    #[test]
    fn map_codec_clone_preserves_name_and_equality() {
        let codec = MapCodec::<DummyPlacer>::of("test_codec");
        let cloned = codec.clone();
        assert_eq!(codec, cloned);
        assert_eq!(cloned.name(), "test_codec");
    }

    #[test]
    fn map_codec_names_distinguish_values() {
        let a = MapCodec::<DummyPlacer>::of("a");
        let b = MapCodec::<DummyPlacer>::of("b");
        assert_ne!(a, b);
    }

    #[test]
    fn different_root_placer_types_can_hold_different_codecs() {
        let dummy = RootPlacerType::register("dummy", MapCodec::<DummyPlacer>::of("dummy"));
        let other = RootPlacerType::register("other", MapCodec::<OtherPlacer>::of("other"));
        assert_eq!(dummy.codec().name(), "dummy");
        assert_eq!(other.codec().name(), "other");
    }
}
