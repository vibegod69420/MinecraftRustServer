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
pub struct Component;

impl Component {
    // JAVA: net.minecraft.network.chat.Component#empty
    pub fn empty() -> MutableComponent {
        MutableComponent {
            text: String::new(),
        }
    }
}

/// Minimal stub for `com.mojang.serialization.MapCodec`.
pub struct MapCodec<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> MapCodec<T> {
    // JAVA: com.mojang.serialization.MapCodec#unit
    pub fn unit(_value: T) -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// Minimal stub for `net.minecraft.network.codec.StreamCodec`.
pub struct StreamCodec<B, T> {
    _marker: PhantomData<fn(B) -> T>,
}

impl<B, T> StreamCodec<B, T> {
    // JAVA: net.minecraft.network.codec.StreamCodec#unit
    pub fn unit(_value: T) -> Self {
        Self {
            _marker: PhantomData,
        }
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

/// Port of `net.minecraft.network.chat.numbers.BlankFormat`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlankFormat;

pub static INSTANCE: BlankFormat = BlankFormat;

pub static TYPE: LazyLock<NumberFormatType<BlankFormat>> = LazyLock::new(|| NumberFormatType {
    map_codec: MapCodec::unit(INSTANCE),
    stream_codec: StreamCodec::unit(INSTANCE),
});

impl BlankFormat {
    // JAVA: net.minecraft.network.chat.numbers.BlankFormat#format
    pub fn format(&self, _value: i32) -> MutableComponent {
        Component::empty()
    }

    // JAVA: net.minecraft.network.chat.numbers.BlankFormat#type
    pub fn get_type(&self) -> &'static NumberFormatType<BlankFormat> {
        &TYPE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_returns_empty_for_zero() {
        let c = INSTANCE.format(0);
        assert_eq!(c.text(), "");
    }

    #[test]
    fn format_returns_empty_for_positive_and_negative() {
        assert_eq!(INSTANCE.format(42).text(), "");
        assert_eq!(INSTANCE.format(-1).text(), "");
    }

    #[test]
    fn format_returns_empty_for_i32_extremes() {
        assert_eq!(INSTANCE.format(i32::MIN).text(), "");
        assert_eq!(INSTANCE.format(i32::MAX).text(), "");
    }

    #[test]
    fn format_handles_float_bit_patterns() {
        let nan_bits = f32::NAN.to_bits() as i32;
        let inf_bits = f32::INFINITY.to_bits() as i32;
        let neg_zero_bits = (-0.0f32).to_bits() as i32;
        assert_eq!(INSTANCE.format(nan_bits).text(), "");
        assert_eq!(INSTANCE.format(inf_bits).text(), "");
        assert_eq!(INSTANCE.format(neg_zero_bits).text(), "");
    }

    #[test]
    fn type_descriptor_is_accessible() {
        let t = INSTANCE.get_type();
        let _map = t.map_codec();
        let _stream = t.stream_codec();
    }
}
