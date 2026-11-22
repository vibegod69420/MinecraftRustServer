use std::fmt;
use std::marker::PhantomData;

/// Marker trait for Minecraft tree decorators.
pub trait TreeDecorator {}

/// Minimal self-contained stub for Mojang's `MapCodec<P>`.
pub struct MapCodec<P> {
    _marker: PhantomData<fn() -> P>,
}

impl<P> MapCodec<P> {
    /// Creates a new codec stub.
    pub const fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<P> Copy for MapCodec<P> {}

impl<P> Clone for MapCodec<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P> Default for MapCodec<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P> PartialEq for MapCodec<P> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<P> Eq for MapCodec<P> {}

impl<P> fmt::Debug for MapCodec<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MapCodec")
    }
}

macro_rules! define_decorators {
    ($($name:ident),* $(,)?) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
            pub struct $name;

            impl TreeDecorator for $name {}

            impl $name {
                pub const CODEC: MapCodec<Self> = MapCodec::new();
            }
        )*
    };
}

define_decorators!(
    TrunkVineDecorator,
    LeaveVineDecorator,
    PaleMossDecorator,
    CreakingHeartDecorator,
    CocoaDecorator,
    BeehiveDecorator,
    AlterGroundDecorator,
    AttachedToLeavesDecorator,
    PlaceOnGroundDecorator,
    AttachedToLogsDecorator,
);

/// Port of `net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecoratorType`.
pub struct TreeDecoratorType<P: TreeDecorator> {
    codec: MapCodec<P>,
}

impl<P: TreeDecorator> Copy for TreeDecoratorType<P> {}

impl<P: TreeDecorator> Clone for TreeDecoratorType<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P: TreeDecorator> PartialEq for TreeDecoratorType<P> {
    fn eq(&self, other: &Self) -> bool {
        self.codec == other.codec
    }
}

impl<P: TreeDecorator> Eq for TreeDecoratorType<P> {}

impl<P: TreeDecorator> fmt::Debug for TreeDecoratorType<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TreeDecoratorType")
            .field("codec", &self.codec)
            .finish()
    }
}

impl<P: TreeDecorator> TreeDecoratorType<P> {
    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecoratorType#<init>
    const fn new(codec: MapCodec<P>) -> Self {
        Self { codec }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecoratorType#codec
    pub fn codec(&self) -> &MapCodec<P> {
        &self.codec
    }
}

// JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecoratorType#register
const fn register<P: TreeDecorator>(_name: &str, codec: MapCodec<P>) -> TreeDecoratorType<P> {
    // In Java this registers into BuiltInRegistries.TREE_DECORATOR_TYPE.
    // The registry side effect is omitted in this self-contained port.
    TreeDecoratorType::new(codec)
}

pub static TRUNK_VINE: TreeDecoratorType<TrunkVineDecorator> =
    register("trunk_vine", TrunkVineDecorator::CODEC);
pub static LEAVE_VINE: TreeDecoratorType<LeaveVineDecorator> =
    register("leave_vine", LeaveVineDecorator::CODEC);
pub static PALE_MOSS: TreeDecoratorType<PaleMossDecorator> =
    register("pale_moss", PaleMossDecorator::CODEC);
pub static CREAKING_HEART: TreeDecoratorType<CreakingHeartDecorator> =
    register("creaking_heart", CreakingHeartDecorator::CODEC);
pub static COCOA: TreeDecoratorType<CocoaDecorator> =
    register("cocoa", CocoaDecorator::CODEC);
pub static BEEHIVE: TreeDecoratorType<BeehiveDecorator> =
    register("beehive", BeehiveDecorator::CODEC);
pub static ALTER_GROUND: TreeDecoratorType<AlterGroundDecorator> =
    register("alter_ground", AlterGroundDecorator::CODEC);
pub static ATTACHED_TO_LEAVES: TreeDecoratorType<AttachedToLeavesDecorator> =
    register("attached_to_leaves", AttachedToLeavesDecorator::CODEC);
pub static PLACE_ON_GROUND: TreeDecoratorType<PlaceOnGroundDecorator> =
    register("place_on_ground", PlaceOnGroundDecorator::CODEC);
pub static ATTACHED_TO_LOGS: TreeDecoratorType<AttachedToLogsDecorator> =
    register("attached_to_logs", AttachedToLogsDecorator::CODEC);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_codec() {
        let t = register("cocoa", CocoaDecorator::CODEC);
        assert_eq!(*t.codec(), CocoaDecorator::CODEC);
        let t2 = TreeDecoratorType::new(CocoaDecorator::CODEC);
        assert_eq!(t, t2);
    }

    #[test]
    fn constants_match_decorator_codecs() {
        assert_eq!(*TRUNK_VINE.codec(), TrunkVineDecorator::CODEC);
        assert_eq!(*LEAVE_VINE.codec(), LeaveVineDecorator::CODEC);
        assert_eq!(*PALE_MOSS.codec(), PaleMossDecorator::CODEC);
        assert_eq!(*CREAKING_HEART.codec(), CreakingHeartDecorator::CODEC);
        assert_eq!(*COCOA.codec(), CocoaDecorator::CODEC);
        assert_eq!(*BEEHIVE.codec(), BeehiveDecorator::CODEC);
        assert_eq!(*ALTER_GROUND.codec(), AlterGroundDecorator::CODEC);
        assert_eq!(
            *ATTACHED_TO_LEAVES.codec(),
            AttachedToLeavesDecorator::CODEC
        );
        assert_eq!(*PLACE_ON_GROUND.codec(), PlaceOnGroundDecorator::CODEC);
        assert_eq!(*ATTACHED_TO_LOGS.codec(), AttachedToLogsDecorator::CODEC);
    }

    #[test]
    fn copy_clone_and_equality() {
        let a = COCOA;
        let b = a;
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, c);

        let d = CocoaDecorator;
        let e = CocoaDecorator::default();
        assert_eq!(d, e);
    }

    #[test]
    fn map_codec_is_copy_eq_and_debuggable() {
        let a: MapCodec<TrunkVineDecorator> = MapCodec::new();
        let b = a;
        let c = MapCodec::default();
        assert_eq!(a, b);
        assert_eq!(a, c);
        assert_eq!(format!("{:?}", a), "MapCodec");
    }

    #[test]
    fn debug_and_java_numeric_semantics_sanity() {
        let dbg = format!("{:?}", COCOA);
        assert!(dbg.contains("TreeDecoratorType"));

        // Java integer semantics sanity checks.
        let max_i32 = i32::MAX;
        assert_eq!(max_i32.wrapping_add(1), i32::MIN);
        let neg: i32 = -5;
        assert_eq!(neg.wrapping_sub(10), -15);
        assert_eq!((-1i32) >> 31, -1);

        // Float bit-pattern sanity checks.
        assert_eq!((-0.0f32).to_bits(), 0x8000_0000u32);
        let nan_bits = f64::NAN.to_bits();
        assert!(f64::from_bits(nan_bits).is_nan());
    }
}
