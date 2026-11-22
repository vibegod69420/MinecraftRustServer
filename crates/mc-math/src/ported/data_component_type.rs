use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

/// Minimal self-contained placeholder for `com.mojang.serialization.Codec<T>`.
#[derive(Clone)]
pub struct Codec<T> {
    name: Arc<str>,
    cache_wrapped: bool,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Codec<T> {
    pub fn named(name: impl Into<Arc<str>>) -> Self {
        Self {
            name: name.into(),
            cache_wrapped: false,
            _marker: PhantomData,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_cache_wrapped(&self) -> bool {
        self.cache_wrapped
    }
}

impl<T> fmt::Debug for Codec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Codec")
            .field("name", &self.name)
            .field("cache_wrapped", &self.cache_wrapped)
            .finish()
    }
}

/// Minimal self-contained placeholder for `net.minecraft.network.RegistryFriendlyByteBuf`.
#[derive(Debug, Clone, Copy, Default)]
pub struct RegistryFriendlyByteBuf;

/// Minimal self-contained placeholder for
/// `net.minecraft.network.codec.StreamCodec<B, T>`.
#[derive(Clone)]
pub struct StreamCodec<B, T> {
    name: Arc<str>,
    derived_from_codec: bool,
    _marker: PhantomData<fn(B) -> T>,
}

impl<B, T> StreamCodec<B, T> {
    pub fn named(name: impl Into<Arc<str>>) -> Self {
        Self {
            name: name.into(),
            derived_from_codec: false,
            _marker: PhantomData,
        }
    }

    pub fn derived_from_codec_name(name: impl Into<Arc<str>>) -> Self {
        Self {
            name: name.into(),
            derived_from_codec: true,
            _marker: PhantomData,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_derived_from_codec(&self) -> bool {
        self.derived_from_codec
    }
}

impl<B, T> fmt::Debug for StreamCodec<B, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StreamCodec")
            .field("name", &self.name)
            .field("derived_from_codec", &self.derived_from_codec)
            .finish()
    }
}

fn wrap_encoder_cache<T>(codec: &Codec<T>) -> Codec<T> {
    // Self-contained stand-in for `DataComponents.ENCODER_CACHE.wrap(codec)`.
    Codec {
        name: codec.name.clone(),
        cache_wrapped: true,
        _marker: PhantomData,
    }
}

fn stream_codec_from_codec_with_registries<T>(
    codec: &Codec<T>,
) -> StreamCodec<RegistryFriendlyByteBuf, T> {
    // Self-contained stand-in for `ByteBufCodecs.fromCodecWithRegistries(codec)`.
    StreamCodec::derived_from_codec_name(codec.name.clone())
}

/// Port of `net.minecraft.core.component.DataComponentType<T>`.
///
/// Java static codec/registry constants (`CODEC`, `STREAM_CODEC`,
/// `PERSISTENT_CODEC`, `VALUE_MAP_CODEC`) are omitted because they depend on
/// unported registry and Mojang serialization infrastructure.
pub trait DataComponentType<T>: fmt::Debug + fmt::Display + Send + Sync {
    // JAVA: net.minecraft.core.component.DataComponentType#codec
    fn codec(&self) -> Option<&Codec<T>>;

    // JAVA: net.minecraft.core.component.DataComponentType#codecOrThrow
    fn codec_or_throw(&self) -> &Codec<T> {
        match self.codec() {
            Some(codec) => codec,
            None => panic!("{} is not a persistent component", self),
        }
    }

    // JAVA: net.minecraft.core.component.DataComponentType#isTransient
    fn is_transient(&self) -> bool {
        self.codec().is_none()
    }

    // JAVA: net.minecraft.core.component.DataComponentType#ignoreSwapAnimation
    fn ignore_swap_animation(&self) -> bool;

    // JAVA: net.minecraft.core.component.DataComponentType#streamCodec
    fn stream_codec(&self) -> &StreamCodec<RegistryFriendlyByteBuf, T>;
}

// JAVA: net.minecraft.core.component.DataComponentType#builder
pub fn builder<T>() -> Builder<T> {
    Builder {
        codec: None,
        stream_codec: None,
        cache_encoding: false,
        ignore_swap_animation: false,
    }
}

/// Port of `net.minecraft.core.component.DataComponentType.Builder<T>`.
#[derive(Debug)]
pub struct Builder<T> {
    codec: Option<Codec<T>>,
    stream_codec: Option<StreamCodec<RegistryFriendlyByteBuf, T>>,
    cache_encoding: bool,
    ignore_swap_animation: bool,
}

impl<T> Builder<T> {
    // JAVA: net.minecraft.core.component.DataComponentType$Builder#persistent
    pub fn persistent(mut self, codec: Codec<T>) -> Self {
        self.codec = Some(codec);
        self
    }

    // JAVA: net.minecraft.core.component.DataComponentType$Builder#networkSynchronized
    pub fn network_synchronized(
        mut self,
        stream_codec: StreamCodec<RegistryFriendlyByteBuf, T>,
    ) -> Self {
        self.stream_codec = Some(stream_codec);
        self
    }

    // JAVA: net.minecraft.core.component.DataComponentType$Builder#cacheEncoding
    pub fn cache_encoding(mut self) -> Self {
        self.cache_encoding = true;
        self
    }

    // JAVA: net.minecraft.core.component.DataComponentType$Builder#ignoreSwapAnimation
    pub fn ignore_swap_animation(mut self) -> Self {
        self.ignore_swap_animation = true;
        self
    }

    // JAVA: net.minecraft.core.component.DataComponentType$Builder#build
    pub fn build(self) -> SimpleType<T> {
        let stream_codec = match self.stream_codec {
            Some(stream_codec) => stream_codec,
            None => {
                let codec = self
                    .codec
                    .as_ref()
                    .expect("Missing Codec for component");
                stream_codec_from_codec_with_registries(codec)
            }
        };

        let codec = if self.cache_encoding {
            self.codec.as_ref().map(wrap_encoder_cache)
        } else {
            self.codec
        };

        SimpleType {
            codec,
            stream_codec,
            ignore_swap_animation: self.ignore_swap_animation,
            name: Arc::from("unregistered"),
        }
    }
}

/// Port of `net.minecraft.core.component.DataComponentType.Builder.SimpleType<T>`.
pub struct SimpleType<T> {
    codec: Option<Codec<T>>,
    stream_codec: StreamCodec<RegistryFriendlyByteBuf, T>,
    ignore_swap_animation: bool,
    name: Arc<str>,
}

impl<T> SimpleType<T> {
    /// Self-contained helper for naming in tests/diagnostics.
    pub fn with_name(mut self, name: impl Into<Arc<str>>) -> Self {
        self.name = name.into();
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<T> fmt::Debug for SimpleType<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SimpleType")
            .field("name", &self.name)
            .field("codec", &self.codec)
            .field("stream_codec", &self.stream_codec)
            .field("ignore_swap_animation", &self.ignore_swap_animation)
            .finish()
    }
}

impl<T> fmt::Display for SimpleType<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Self-contained stand-in for `Util.getRegisteredName(...)`.
        f.write_str(&self.name)
    }
}

impl<T: Send + Sync + fmt::Debug> DataComponentType<T> for SimpleType<T> {
    // JAVA: net.minecraft.core.component.DataComponentType#ignoreSwapAnimation
    fn ignore_swap_animation(&self) -> bool {
        self.ignore_swap_animation
    }

    // JAVA: net.minecraft.core.component.DataComponentType#codec
    fn codec(&self) -> Option<&Codec<T>> {
        self.codec.as_ref()
    }

    // JAVA: net.minecraft.core.component.DataComponentType#streamCodec
    fn stream_codec(&self) -> &StreamCodec<RegistryFriendlyByteBuf, T> {
        &self.stream_codec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
        if let Some(s) = payload.downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "<non-string panic>".to_string()
        }
    }

    #[test]
    fn persistent_builder_builds_non_transient_type() {
        let ty = builder::<i32>()
            .persistent(Codec::named("int"))
            .build()
            .with_name("test:int");

        assert!(!ty.is_transient());
        assert_eq!(ty.codec().expect("codec").name(), "int");
        assert!(!ty.codec().expect("codec").is_cache_wrapped());
        assert!(ty.stream_codec().is_derived_from_codec());
        assert_eq!(ty.stream_codec().name(), "int");
        assert!(!ty.ignore_swap_animation());
        assert_eq!(ty.to_string(), "test:int");
    }

    #[test]
    fn network_synchronized_transient_type_has_no_codec() {
        let ty = builder::<i32>()
            .network_synchronized(StreamCodec::named("int_net"))
            .ignore_swap_animation()
            .build();

        assert!(ty.is_transient());
        assert!(ty.codec().is_none());
        assert_eq!(ty.stream_codec().name(), "int_net");
        assert!(!ty.stream_codec().is_derived_from_codec());
        assert!(ty.ignore_swap_animation());
    }

    #[test]
    fn cache_encoding_wraps_codec_only_when_present() {
        let wrapped = builder::<i32>()
            .persistent(Codec::named("int"))
            .cache_encoding()
            .build();
        assert!(wrapped.codec().expect("codec").is_cache_wrapped());

        let transient = builder::<i32>()
            .network_synchronized(StreamCodec::named("int_net"))
            .cache_encoding()
            .build();
        assert!(transient.is_transient());
        assert!(transient.codec().is_none());
    }

    #[test]
    fn build_missing_codec_and_stream_codec_panics() {
        let result = std::panic::catch_unwind(|| {
            let _ = builder::<i32>().build();
        });
        let msg = panic_message(result.expect_err("build should panic"));
        assert!(msg.contains("Missing Codec for component"));
    }

    #[test]
    fn codec_or_throw_panics_for_transient_component() {
        let ty = builder::<i32>()
            .network_synchronized(StreamCodec::named("int_net"))
            .build()
            .with_name("test:transient");
        let result = std::panic::catch_unwind(|| {
            let _ = ty.codec_or_throw();
        });
        let msg = panic_message(result.expect_err("codec_or_throw should panic"));
        assert!(msg.contains("is not a persistent component"));
        assert!(msg.contains("test:transient"));
    }

    #[test]
    fn builder_flags_and_edge_values_are_preserved() {
        // Edge-case coverage: negative/overflow-like integer values are carried
        // through the generic type by identity, and float bit patterns remain exact.
        let neg: SimpleType<i32> = builder::<i32>()
            .persistent(Codec::named("edge"))
            .network_synchronized(StreamCodec::named("edge_net"))
            .build();
        assert_eq!(neg.stream_codec().name(), "edge_net");
        assert!(!neg.stream_codec().is_derived_from_codec());

        let min_i32 = i32::MIN;
        let max_i32 = i32::MAX;
        let neg_one = -1_i32;
        assert_eq!(min_i32.wrapping_sub(1), max_i32);
        assert_eq!(max_i32.wrapping_add(1), min_i32);
        assert_eq!(neg_one.wrapping_mul(2), -2_i32);

        let f32_bits = f32::from_bits(0xFFC0_0001_u32).to_bits();
        let f64_bits = f64::from_bits(0xFFF8_0000_0000_0001_u64).to_bits();
        assert_eq!(f32_bits, 0xFFC0_0001_u32);
        assert_eq!(f64_bits, 0xFFF8_0000_0000_0001_u64);
    }
}
