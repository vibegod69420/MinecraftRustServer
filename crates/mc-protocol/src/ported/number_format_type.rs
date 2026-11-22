use std::fmt::Debug;

/// Minimal placeholder for `net.minecraft.network.chat.MutableComponent`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MutableComponent {
    pub text: String,
}

/// Minimal placeholder for `net.minecraft.network.chat.numbers.NumberFormat`.
/// Canonical interface is ported in `number_format.rs`.
pub trait NumberFormat: Debug {}

/// Minimal placeholder for `com.mojang.serialization.MapCodec`.
#[derive(Debug, Clone)]
pub struct MapCodec<T> {
    pub id: String,
    _marker: std::marker::PhantomData<T>,
}

impl<T> MapCodec<T> {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> PartialEq for MapCodec<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for MapCodec<T> {}

/// Minimal placeholder for `net.minecraft.network.RegistryFriendlyByteBuf`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RegistryFriendlyByteBuf;

/// Minimal placeholder for `net.minecraft.network.codec.StreamCodec`.
#[derive(Debug, Clone)]
pub struct StreamCodec<B, T> {
    pub id: String,
    _marker: std::marker::PhantomData<(B, T)>,
}

impl<B, T> StreamCodec<B, T> {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<B, T> PartialEq for StreamCodec<B, T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<B, T> Eq for StreamCodec<B, T> {}

/// JAVA: net.minecraft.network.chat.numbers.NumberFormatType
pub trait NumberFormatType<T: NumberFormat>: Debug {
    /// JAVA: net.minecraft.network.chat.numbers.NumberFormatType#mapCodec
    fn map_codec(&self) -> MapCodec<T>;

    /// JAVA: net.minecraft.network.chat.numbers.NumberFormatType#streamCodec
    fn stream_codec(&self) -> StreamCodec<RegistryFriendlyByteBuf, T>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct DummyNumberFormat;

    impl NumberFormat for DummyNumberFormat {}

    #[derive(Debug)]
    struct DummyNumberFormatType;

    impl NumberFormatType<DummyNumberFormat> for DummyNumberFormatType {
        /// JAVA: net.minecraft.network.chat.numbers.NumberFormatType#mapCodec
        fn map_codec(&self) -> MapCodec<DummyNumberFormat> {
            MapCodec::new("dummy_map")
        }

        /// JAVA: net.minecraft.network.chat.numbers.NumberFormatType#streamCodec
        fn stream_codec(&self) -> StreamCodec<RegistryFriendlyByteBuf, DummyNumberFormat> {
            StreamCodec::new("dummy_stream")
        }
    }

    #[test]
    fn map_codec_identity_and_id() {
        let t = DummyNumberFormatType;
        let codec = t.map_codec();
        assert_eq!(codec, MapCodec::<DummyNumberFormat>::new("dummy_map"));
        assert_eq!(codec.id, "dummy_map");
    }

    #[test]
    fn stream_codec_identity_and_id() {
        let t = DummyNumberFormatType;
        let codec = t.stream_codec();
        assert_eq!(
            codec,
            StreamCodec::<RegistryFriendlyByteBuf, DummyNumberFormat>::new("dummy_stream")
        );
        assert_eq!(codec.id, "dummy_stream");
    }

    #[test]
    fn codecs_with_different_ids_are_not_equal() {
        let a = MapCodec::<DummyNumberFormat>::new("a");
        let b = MapCodec::<DummyNumberFormat>::new("b");
        assert_ne!(a, b);

        let sa = StreamCodec::<RegistryFriendlyByteBuf, DummyNumberFormat>::new("a");
        let sb = StreamCodec::<RegistryFriendlyByteBuf, DummyNumberFormat>::new("b");
        assert_ne!(sa, sb);
    }

    #[test]
    fn registry_friendly_byte_buf_placeholder_default() {
        let buf = RegistryFriendlyByteBuf::default();
        let buf2 = RegistryFriendlyByteBuf;
        assert_eq!(buf, buf2);
        let cloned = buf.clone();
        assert_eq!(cloned, buf2);
    }
}
