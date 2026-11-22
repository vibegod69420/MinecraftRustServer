use std::marker::PhantomData;

/// Minimal MapCodec stub for a self-contained port.
#[derive(Debug)]
pub struct MapCodec<P>(PhantomData<fn() -> P>);

impl<P> MapCodec<P> {
    pub const fn stub() -> Self {
        Self(PhantomData)
    }
}

impl<P> Clone for MapCodec<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P> Copy for MapCodec<P> {}

impl<P> Default for MapCodec<P> {
    fn default() -> Self {
        Self::stub()
    }
}

impl<P> PartialEq for MapCodec<P> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<P> Eq for MapCodec<P> {}

// Marker types for trunk placer variants referenced by TrunkPlacerType constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StraightTrunkPlacer;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForkingTrunkPlacer;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GiantTrunkPlacer;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MegaJungleTrunkPlacer;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DarkOakTrunkPlacer;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FancyTrunkPlacer;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BendingTrunkPlacer;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpwardsBranchingTrunkPlacer;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CherryTrunkPlacer;

#[derive(Debug, Clone)]
pub struct TrunkPlacerType<P> {
    codec: MapCodec<P>,
    _marker: PhantomData<fn() -> P>,
}

impl<P> TrunkPlacerType<P> {
    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.TrunkPlacerType#<init>
    const fn new(codec: MapCodec<P>) -> Self {
        Self {
            codec,
            _marker: PhantomData,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.TrunkPlacerType#register
    const fn register(_name: &'static str, codec: MapCodec<P>) -> Self {
        // Registry.register(BuiltInRegistries.TRUNK_PLACER_TYPE, name, new TrunkPlacerType<>(codec))
        // Registry side effect omitted in this self-contained port.
        Self::new(codec)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.TrunkPlacerType#codec
    pub const fn codec(&self) -> &MapCodec<P> {
        &self.codec
    }
}

pub const STRAIGHT_TRUNK_PLACER: TrunkPlacerType<StraightTrunkPlacer> =
    TrunkPlacerType::register("straight_trunk_placer", MapCodec::stub());
pub const FORKING_TRUNK_PLACER: TrunkPlacerType<ForkingTrunkPlacer> =
    TrunkPlacerType::register("forking_trunk_placer", MapCodec::stub());
pub const GIANT_TRUNK_PLACER: TrunkPlacerType<GiantTrunkPlacer> =
    TrunkPlacerType::register("giant_trunk_placer", MapCodec::stub());
pub const MEGA_JUNGLE_TRUNK_PLACER: TrunkPlacerType<MegaJungleTrunkPlacer> =
    TrunkPlacerType::register("mega_jungle_trunk_placer", MapCodec::stub());
pub const DARK_OAK_TRUNK_PLACER: TrunkPlacerType<DarkOakTrunkPlacer> =
    TrunkPlacerType::register("dark_oak_trunk_placer", MapCodec::stub());
pub const FANCY_TRUNK_PLACER: TrunkPlacerType<FancyTrunkPlacer> =
    TrunkPlacerType::register("fancy_trunk_placer", MapCodec::stub());
pub const BENDING_TRUNK_PLACER: TrunkPlacerType<BendingTrunkPlacer> =
    TrunkPlacerType::register("bending_trunk_placer", MapCodec::stub());
pub const UPWARDS_BRANCHING_TRUNK_PLACER: TrunkPlacerType<UpwardsBranchingTrunkPlacer> =
    TrunkPlacerType::register("upwards_branching_trunk_placer", MapCodec::stub());
pub const CHERRY_TRUNK_PLACER: TrunkPlacerType<CherryTrunkPlacer> =
    TrunkPlacerType::register("cherry_trunk_placer", MapCodec::stub());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_codec() {
        let t: TrunkPlacerType<StraightTrunkPlacer> =
            TrunkPlacerType::register("test", MapCodec::stub());
        let c = t.codec();
        assert_eq!(*c, MapCodec::stub());
    }

    #[test]
    fn test_constants_exist_and_have_codecs() {
        let _ = STRAIGHT_TRUNK_PLACER.codec();
        let _ = FORKING_TRUNK_PLACER.codec();
        let _ = GIANT_TRUNK_PLACER.codec();
        let _ = MEGA_JUNGLE_TRUNK_PLACER.codec();
        let _ = DARK_OAK_TRUNK_PLACER.codec();
        let _ = FANCY_TRUNK_PLACER.codec();
        let _ = BENDING_TRUNK_PLACER.codec();
        let _ = UPWARDS_BRANCHING_TRUNK_PLACER.codec();
        let _ = CHERRY_TRUNK_PLACER.codec();
    }

    #[test]
    fn test_map_codec_clone_default_eq() {
        let c: MapCodec<StraightTrunkPlacer> = MapCodec::default();
        let c2 = c;
        let c3 = c.clone();
        assert_eq!(c2, c3);
    }

    #[test]
    fn test_new_constructor() {
        let t = TrunkPlacerType::<FancyTrunkPlacer>::new(MapCodec::stub());
        assert_eq!(*t.codec(), MapCodec::<FancyTrunkPlacer>::stub());
    }
}
