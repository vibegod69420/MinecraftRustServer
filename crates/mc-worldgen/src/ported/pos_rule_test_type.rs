use std::marker::PhantomData;

/// Placeholder for unported `net.minecraft.world.level.levelgen.structure.templatesystem.PosRuleTest`.
pub trait PosRuleTest: 'static {}

/// Placeholder for unported `com.mojang.serialization.MapCodec`.
#[derive(Debug)]
pub struct MapCodec<P: ?Sized>(PhantomData<fn() -> P>);

impl<P: ?Sized> MapCodec<P> {
    /// Creates a placeholder codec.
    pub fn of() -> Self {
        Self(PhantomData)
    }
}

impl<P: ?Sized> Clone for MapCodec<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P: ?Sized> Copy for MapCodec<P> {}

impl<P: ?Sized> PartialEq for MapCodec<P> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<P: ?Sized> Eq for MapCodec<P> {}

// Placeholder concrete types referenced by registry constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PosAlwaysTrueTest;
impl PosRuleTest for PosAlwaysTrueTest {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinearPosTest;
impl PosRuleTest for LinearPosTest {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxisAlignedLinearPosTest;
impl PosRuleTest for AxisAlignedLinearPosTest {}

/// Port of `net.minecraft.world.level.levelgen.structure.templatesystem.PosRuleTestType`.
#[derive(Debug)]
pub struct PosRuleTestType<P: PosRuleTest + ?Sized> {
    name: &'static str,
    codec: MapCodec<P>,
}

impl<P: PosRuleTest + ?Sized> Clone for PosRuleTestType<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P: PosRuleTest + ?Sized> Copy for PosRuleTestType<P> {}

impl<P: PosRuleTest + ?Sized> PartialEq for PosRuleTestType<P> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.codec == other.codec
    }
}

impl<P: PosRuleTest + ?Sized> Eq for PosRuleTestType<P> {}

impl<P: PosRuleTest + ?Sized> PosRuleTestType<P> {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.PosRuleTestType#codec
    pub fn codec(&self) -> MapCodec<P> {
        self.codec
    }

    /// Returns the registered name used by this type descriptor.
    pub fn name(&self) -> &'static str {
        self.name
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.PosRuleTestType#register
pub fn register<P: PosRuleTest>(name: &'static str, codec: MapCodec<P>) -> PosRuleTestType<P> {
    // Java: Registry.register(BuiltInRegistries.POS_RULE_TEST, name, () -> codec)
    // Registry side effects are omitted in this self-contained module.
    PosRuleTestType { name, codec }
}

pub const ALWAYS_TRUE_TEST: PosRuleTestType<PosAlwaysTrueTest> = PosRuleTestType {
    name: "always_true",
    codec: MapCodec(PhantomData),
};

pub const LINEAR_POS_TEST: PosRuleTestType<LinearPosTest> = PosRuleTestType {
    name: "linear_pos",
    codec: MapCodec(PhantomData),
};

pub const AXIS_ALIGNED_LINEAR_POS_TEST: PosRuleTestType<AxisAlignedLinearPosTest> =
    PosRuleTestType {
        name: "axis_aligned_linear_pos",
        codec: MapCodec(PhantomData),
    };

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_returns_type_with_name() {
        let codec = MapCodec::<PosAlwaysTrueTest>::of();
        let ty = register("always_true", codec);
        assert_eq!(ty.name(), "always_true");
        assert_eq!(ty.codec(), codec);
    }

    #[test]
    fn constants_have_expected_names() {
        assert_eq!(ALWAYS_TRUE_TEST.name(), "always_true");
        assert_eq!(LINEAR_POS_TEST.name(), "linear_pos");
        assert_eq!(
            AXIS_ALIGNED_LINEAR_POS_TEST.name(),
            "axis_aligned_linear_pos"
        );
    }

    #[test]
    fn type_equality_is_name_based_with_codec_placeholder() {
        let a = register("linear_pos", MapCodec::<LinearPosTest>::of());
        let b = LINEAR_POS_TEST;
        assert_eq!(a, b);
        let c = register("other", MapCodec::<LinearPosTest>::of());
        assert_ne!(a, c);
    }

    #[test]
    fn copy_and_clone_preserve_value() {
        let ty = AXIS_ALIGNED_LINEAR_POS_TEST;
        let copied = ty;
        let cloned = ty.clone();
        assert_eq!(ty, copied);
        assert_eq!(ty, cloned);
        assert_eq!(ty.name(), "axis_aligned_linear_pos");
    }

    #[test]
    fn map_codec_placeholder_is_copy_eq() {
        let c1 = MapCodec::<PosAlwaysTrueTest>::of();
        let c2 = c1;
        let c3 = c1.clone();
        assert_eq!(c1, c2);
        assert_eq!(c1, c3);
    }
}
