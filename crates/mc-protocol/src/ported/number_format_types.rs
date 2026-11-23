use std::marker::PhantomData;
use std::sync::LazyLock;

/// Minimal stub for `net.minecraft.network.RegistryFriendlyByteBuf`.
#[derive(Debug, Default)]
pub struct RegistryFriendlyByteBuf;

/// Minimal stub for `com.mojang.serialization.MapCodec`.
pub struct MapCodec<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> MapCodec<T> {
    fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    // JAVA: com.mojang.serialization.MapCodec#codec
    pub fn codec(self) -> Codec<T> {
        let _ = self;
        Codec {
            _marker: PhantomData,
        }
    }
}

/// Minimal stub for `com.mojang.serialization.Codec`.
pub struct Codec<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Codec<T> {
    fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// Minimal stub for `net.minecraft.network.codec.StreamCodec`.
pub struct StreamCodec<B, T> {
    _marker: PhantomData<fn(B) -> T>,
}

impl<B, T: 'static> StreamCodec<B, T> {
    fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    // JAVA: net.minecraft.network.codec.StreamCodec#apply
    pub fn apply<U: 'static, F>(self, _f: F) -> StreamCodec<B, U>
    where
        F: FnOnce(Self) -> StreamCodec<B, U>,
    {
        let _ = self;
        StreamCodec {
            _marker: PhantomData,
        }
    }
}

/// Minimal placeholder for `net.minecraft.network.chat.numbers.NumberFormat`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberFormat;

/// Minimal stub for `net.minecraft.network.chat.numbers.NumberFormatType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumberFormatType {
    id: &'static str,
}

impl NumberFormatType {
    pub fn id(&self) -> &'static str {
        self.id
    }
}

/// Minimal stub for `net.minecraft.core.Registry`.
#[derive(Debug, Default)]
pub struct Registry<T> {
    entries: Vec<(&'static str, T)>,
}

impl<T: Clone> Registry<T> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    // JAVA: net.minecraft.core.Registry#register
    pub fn register(&mut self, name: &'static str, value: T) -> T {
        self.entries.push((name, value.clone()));
        value
    }

    pub fn get(&self, name: &str) -> Option<&T> {
        self.entries.iter().find(|(n, _)| *n == name).map(|(_, v)| v)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Local stub for `net.minecraft.network.chat.numbers.BlankFormat`.
pub struct BlankFormat;
impl BlankFormat {
    pub const TYPE: NumberFormatType = NumberFormatType { id: "blank" };
}

/// Local stub for `net.minecraft.network.chat.numbers.StyledFormat`.
pub struct StyledFormat;
impl StyledFormat {
    pub const TYPE: NumberFormatType = NumberFormatType { id: "styled" };
}

/// Local stub for `net.minecraft.network.chat.numbers.FixedFormat`.
pub struct FixedFormat;
impl FixedFormat {
    pub const TYPE: NumberFormatType = NumberFormatType { id: "fixed" };
}

pub static MAP_CODEC: LazyLock<MapCodec<NumberFormat>> = LazyLock::new(MapCodec::new);
pub static CODEC: LazyLock<Codec<NumberFormat>> = LazyLock::new(|| MapCodec::<NumberFormat>::new().codec());
pub static STREAM_CODEC: LazyLock<StreamCodec<RegistryFriendlyByteBuf, NumberFormat>> =
    LazyLock::new(StreamCodec::new);
pub static OPTIONAL_STREAM_CODEC: LazyLock<StreamCodec<RegistryFriendlyByteBuf, Option<NumberFormat>>> =
    LazyLock::new(StreamCodec::new);

// JAVA: net.minecraft.network.chat.numbers.NumberFormatTypes#bootstrap
pub fn bootstrap(registry: &mut Registry<NumberFormatType>) -> NumberFormatType {
    registry.register("blank", BlankFormat::TYPE);
    registry.register("styled", StyledFormat::TYPE);
    registry.register("fixed", FixedFormat::TYPE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_registers_three_types_in_order() {
        let mut reg = Registry::<NumberFormatType>::new();
        bootstrap(&mut reg);
        assert_eq!(reg.len(), 3);
        assert_eq!(reg.get("blank"), Some(&BlankFormat::TYPE));
        assert_eq!(reg.get("styled"), Some(&StyledFormat::TYPE));
        assert_eq!(reg.get("fixed"), Some(&FixedFormat::TYPE));
    }

    #[test]
    fn bootstrap_returns_fixed_type() {
        let mut reg = Registry::<NumberFormatType>::new();
        let ret = bootstrap(&mut reg);
        assert_eq!(ret, FixedFormat::TYPE);
        assert_eq!(ret.id(), "fixed");
    }

    #[test]
    fn registry_missing_lookup_returns_none() {
        let mut reg = Registry::<NumberFormatType>::new();
        bootstrap(&mut reg);
        assert_eq!(reg.get("unknown"), None);
        assert!(!reg.is_empty());
    }

    #[test]
    fn registry_register_returns_value() {
        let mut reg = Registry::<NumberFormatType>::new();
        let v = reg.register("blank", BlankFormat::TYPE);
        assert_eq!(v, BlankFormat::TYPE);
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn numeric_edge_cases_cover_overflow_negative_and_float_bits() {
        let mut reg = Registry::<i32>::new();
        reg.register("min", i32::MIN);
        reg.register("max", i32::MAX);
        reg.register("neg", -1);
        reg.register("nan_bits", f32::NAN.to_bits() as i32);
        reg.register("neg_inf_bits", f32::NEG_INFINITY.to_bits() as i32);

        assert_eq!(reg.get("min"), Some(&i32::MIN));
        assert_eq!(reg.get("max"), Some(&i32::MAX));
        assert_eq!(reg.get("neg"), Some(&-1));
        assert_eq!(reg.get("nan_bits"), Some(&(f32::NAN.to_bits() as i32)));
        assert_eq!(
            reg.get("neg_inf_bits"),
            Some(&(f32::NEG_INFINITY.to_bits() as i32))
        );
    }

    #[test]
    fn static_codec_fields_are_initialized() {
        let _ = &*MAP_CODEC;
        let _ = &*CODEC;
        let _ = &*STREAM_CODEC;
        let _ = &*OPTIONAL_STREAM_CODEC;
    }
}
