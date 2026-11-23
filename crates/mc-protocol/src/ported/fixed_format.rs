use std::marker::PhantomData;
use std::sync::LazyLock;

/// Minimal stub for `net.minecraft.network.RegistryFriendlyByteBuf`.
#[derive(Debug, Default)]
pub struct RegistryFriendlyByteBuf;

/// Minimal stub for `net.minecraft.network.chat.MutableComponent`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MutableComponent {
    text: String,
}

impl MutableComponent {
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// Minimal stub for `net.minecraft.network.chat.Component`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Component {
    text: String,
}

impl Component {
    pub fn literal(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    // JAVA: net.minecraft.network.chat.Component#copy
    pub fn copy(&self) -> MutableComponent {
        MutableComponent {
            text: self.text.clone(),
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

    // JAVA: com.mojang.serialization.Codec#fieldOf
    pub fn field_of(self, _name: &str) -> MapCodec<T> {
        let _ = self;
        MapCodec {
            _marker: PhantomData,
        }
    }
}

/// Minimal stub for `com.mojang.serialization.MapCodec`.
pub struct MapCodec<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> MapCodec<T> {
    // JAVA: com.mojang.serialization.MapCodec#xmap
    pub fn xmap<U, F, G>(self, _to: F, _from: G) -> MapCodec<U>
    where
        F: Fn(T) -> U,
        G: Fn(U) -> T,
    {
        let _ = self;
        MapCodec {
            _marker: PhantomData,
        }
    }
}

/// Minimal stub for `net.minecraft.network.codec.StreamCodec`.
pub struct StreamCodec<B, T> {
    _marker: PhantomData<fn(B) -> T>,
}

impl<B, T> StreamCodec<B, T> {
    fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    // JAVA: net.minecraft.network.codec.StreamCodec#composite
    pub fn composite<C, F, G>(_codec: StreamCodec<B, C>, _getter: F, _ctor: G) -> StreamCodec<B, T>
    where
        F: Fn(&T) -> C,
        G: Fn(C) -> T,
    {
        StreamCodec {
            _marker: PhantomData,
        }
    }
}

/// Minimal stub for `net.minecraft.network.chat.ComponentSerialization`.
pub struct ComponentSerialization;

impl ComponentSerialization {
    pub fn codec() -> Codec<Component> {
        Codec::new()
    }

    pub fn trusted_stream_codec() -> StreamCodec<RegistryFriendlyByteBuf, Component> {
        StreamCodec::new()
    }
}

/// Minimal stub for `net.minecraft.network.chat.numbers.NumberFormatType`.
pub struct NumberFormatType<T: 'static> {
    map_codec: MapCodec<T>,
    stream_codec: StreamCodec<RegistryFriendlyByteBuf, T>,
}

impl<T: 'static> NumberFormatType<T> {
    // JAVA: net.minecraft.network.chat.numbers.NumberFormatType#mapCodec
    pub fn map_codec(&self) -> &MapCodec<T> {
        &self.map_codec
    }

    // JAVA: net.minecraft.network.chat.numbers.NumberFormatType#streamCodec
    pub fn stream_codec(&self) -> &StreamCodec<RegistryFriendlyByteBuf, T> {
        &self.stream_codec
    }
}

/// Port of `net.minecraft.network.chat.numbers.FixedFormat`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedFormat {
    value: Component,
}

impl FixedFormat {
    // JAVA: net.minecraft.network.chat.numbers.FixedFormat#<init>
    pub fn new(value: Component) -> Self {
        Self { value }
    }

    // JAVA: net.minecraft.network.chat.numbers.FixedFormat#value
    pub fn value(&self) -> &Component {
        &self.value
    }

    // JAVA: net.minecraft.network.chat.numbers.FixedFormat#format
    pub fn format(&self, _value: i32) -> MutableComponent {
        self.value.copy()
    }

    // JAVA: net.minecraft.network.chat.numbers.FixedFormat#type
    pub fn get_type(&self) -> &'static NumberFormatType<FixedFormat> {
        &TYPE
    }
}

pub static TYPE: LazyLock<NumberFormatType<FixedFormat>> = LazyLock::new(|| {
    let map_codec = ComponentSerialization::codec()
        .field_of("value")
        .xmap(FixedFormat::new, |f: FixedFormat| f.value().clone());
    let stream_codec = StreamCodec::composite(
        ComponentSerialization::trusted_stream_codec(),
        |f: &FixedFormat| f.value().clone(),
        FixedFormat::new,
    );
    NumberFormatType {
        map_codec,
        stream_codec,
    }
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_returns_copy_of_component_value() {
        let fmt = FixedFormat::new(Component::literal("hello"));
        let out = fmt.format(0);
        assert_eq!(out.text(), "hello");
        assert_eq!(fmt.value(), &Component::literal("hello"));
    }

    #[test]
    fn format_ignores_positive_and_negative_inputs() {
        let fmt = FixedFormat::new(Component::literal("score"));
        assert_eq!(fmt.format(42).text(), "score");
        assert_eq!(fmt.format(-1).text(), "score");
        assert_eq!(fmt.format(-123_456).text(), "score");
    }

    #[test]
    fn format_handles_i32_extremes() {
        let fmt = FixedFormat::new(Component::literal("x"));
        assert_eq!(fmt.format(i32::MIN).text(), "x");
        assert_eq!(fmt.format(i32::MAX).text(), "x");
    }

    #[test]
    fn format_handles_float_bit_patterns() {
        let fmt = FixedFormat::new(Component::literal("f"));
        let nan_bits = f32::NAN.to_bits() as i32;
        let inf_bits = f32::INFINITY.to_bits() as i32;
        let neg_inf_bits = f32::NEG_INFINITY.to_bits() as i32;
        assert_eq!(fmt.format(nan_bits).text(), "f");
        assert_eq!(fmt.format(inf_bits).text(), "f");
        assert_eq!(fmt.format(neg_inf_bits).text(), "f");
    }

    #[test]
    fn type_descriptor_is_accessible() {
        let fmt = FixedFormat::new(Component::literal("t"));
        let t = fmt.get_type();
        let _map = t.map_codec();
        let _stream = t.stream_codec();
    }
}
