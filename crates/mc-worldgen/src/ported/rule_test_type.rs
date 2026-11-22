use std::marker::PhantomData;

/// Placeholder for unported `net.minecraft.world.level.levelgen.structure.templatesystem.RuleTest`.
pub trait RuleTest: 'static {}

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
pub struct AlwaysTrueTest;
impl RuleTest for AlwaysTrueTest {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockMatchTest;
impl RuleTest for BlockMatchTest {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockStateMatchTest;
impl RuleTest for BlockStateMatchTest {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagMatchTest;
impl RuleTest for TagMatchTest {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandomBlockMatchTest;
impl RuleTest for RandomBlockMatchTest {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandomBlockStateMatchTest;
impl RuleTest for RandomBlockStateMatchTest {}

/// Port of `net.minecraft.world.level.levelgen.structure.templatesystem.RuleTestType`.
#[derive(Debug)]
pub struct RuleTestType<P: RuleTest + ?Sized> {
    name: &'static str,
    codec: MapCodec<P>,
}

impl<P: RuleTest + ?Sized> Clone for RuleTestType<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P: RuleTest + ?Sized> Copy for RuleTestType<P> {}

impl<P: RuleTest + ?Sized> PartialEq for RuleTestType<P> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.codec == other.codec
    }
}

impl<P: RuleTest + ?Sized> Eq for RuleTestType<P> {}

impl<P: RuleTest + ?Sized> RuleTestType<P> {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.RuleTestType#codec
    pub fn codec(&self) -> MapCodec<P> {
        self.codec
    }

    /// Returns the registered name used by this type descriptor.
    pub fn name(&self) -> &'static str {
        self.name
    }
}

// JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.RuleTestType#register
pub fn register<P: RuleTest>(name: &'static str, codec: MapCodec<P>) -> RuleTestType<P> {
    // Java: Registry.register(BuiltInRegistries.RULE_TEST, name, () -> codec)
    // Registry side effects are omitted in this self-contained module.
    RuleTestType { name, codec }
}

pub const ALWAYS_TRUE_TEST: RuleTestType<AlwaysTrueTest> = RuleTestType {
    name: "always_true",
    codec: MapCodec(PhantomData),
};

pub const BLOCK_TEST: RuleTestType<BlockMatchTest> = RuleTestType {
    name: "block_match",
    codec: MapCodec(PhantomData),
};

pub const BLOCKSTATE_TEST: RuleTestType<BlockStateMatchTest> = RuleTestType {
    name: "blockstate_match",
    codec: MapCodec(PhantomData),
};

pub const TAG_TEST: RuleTestType<TagMatchTest> = RuleTestType {
    name: "tag_match",
    codec: MapCodec(PhantomData),
};

pub const RANDOM_BLOCK_TEST: RuleTestType<RandomBlockMatchTest> = RuleTestType {
    name: "random_block_match",
    codec: MapCodec(PhantomData),
};

pub const RANDOM_BLOCKSTATE_TEST: RuleTestType<RandomBlockStateMatchTest> = RuleTestType {
    name: "random_blockstate_match",
    codec: MapCodec(PhantomData),
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_returns_type_with_name() {
        let codec = MapCodec::<AlwaysTrueTest>::of();
        let ty = register("always_true", codec);
        assert_eq!(ty.name(), "always_true");
        assert_eq!(ty.codec(), codec);
    }

    #[test]
    fn constants_have_expected_names() {
        assert_eq!(ALWAYS_TRUE_TEST.name(), "always_true");
        assert_eq!(BLOCK_TEST.name(), "block_match");
        assert_eq!(BLOCKSTATE_TEST.name(), "blockstate_match");
        assert_eq!(TAG_TEST.name(), "tag_match");
        assert_eq!(RANDOM_BLOCK_TEST.name(), "random_block_match");
        assert_eq!(RANDOM_BLOCKSTATE_TEST.name(), "random_blockstate_match");
    }

    #[test]
    fn type_equality_distinguishes_names() {
        let a = register("block_match", MapCodec::<BlockMatchTest>::of());
        assert_eq!(a, BLOCK_TEST);
        let b = register("block_match_other", MapCodec::<BlockMatchTest>::of());
        assert_ne!(a, b);
    }

    #[test]
    fn copy_and_clone_preserve_value() {
        let ty = RANDOM_BLOCKSTATE_TEST;
        let copied = ty;
        let cloned = ty.clone();
        assert_eq!(ty, copied);
        assert_eq!(ty, cloned);
        assert_eq!(ty.name(), "random_blockstate_match");
    }

    #[test]
    fn map_codec_placeholder_is_copy_eq() {
        let c1 = MapCodec::<TagMatchTest>::of();
        let c2 = c1;
        let c3 = c1.clone();
        assert_eq!(c1, c2);
        assert_eq!(c1, c3);
    }
}
