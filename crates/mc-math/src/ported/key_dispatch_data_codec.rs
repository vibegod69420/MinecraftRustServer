//! Faithful port of `net.minecraft.util.KeyDispatchDataCodec` (Minecraft Java 1.21.11).
//!
//! This is a small structural record wrapper around a map codec. Because the full
//! Mojang serialization stack is not ported here, this module provides a minimal,
//! self-contained `MapCodec<A>` stand-in that preserves the generic shape required
//! by [`KeyDispatchDataCodec`].

use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// Minimal self-contained stand-in for `com.mojang.serialization.MapCodec<A>`.
///
/// The real Mojang serialization codec infrastructure is outside the scope of this
/// module. This stub keeps the generic parameter and an opaque identifier so tests
/// can distinguish instances while remaining fully self-contained.
pub struct MapCodec<A> {
    id: i64,
    _marker: PhantomData<fn(A) -> A>,
}

impl<A> MapCodec<A> {
    /// Creates a placeholder map codec with the given opaque identifier.
    ///
    /// This is a test/support constructor for the self-contained stub, not a ported
    /// Minecraft Java method.
    pub fn with_id(id: i64) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }

    /// Returns the opaque identifier associated with this stub codec.
    ///
    /// This is a test/support accessor for the self-contained stub, not a ported
    /// Minecraft Java method.
    pub fn id(&self) -> i64 {
        self.id
    }
}

impl<A> Clone for MapCodec<A> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _marker: PhantomData,
        }
    }
}

impl<A> PartialEq for MapCodec<A> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<A> Eq for MapCodec<A> {}

impl<A> Hash for MapCodec<A> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<A> fmt::Debug for MapCodec<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MapCodec").field("id", &self.id).finish()
    }
}

/// Faithful port of `net.minecraft.util.KeyDispatchDataCodec<A>`.
///
/// Java record shape:
/// `public record KeyDispatchDataCodec<A>(MapCodec<A> codec)`
pub struct KeyDispatchDataCodec<A> {
    codec: MapCodec<A>,
}

impl<A> KeyDispatchDataCodec<A> {
    // JAVA: net.minecraft.util.KeyDispatchDataCodec#codec
    pub fn codec(&self) -> &MapCodec<A> {
        &self.codec
    }

    // JAVA: net.minecraft.util.KeyDispatchDataCodec#of
    pub fn of(codec: MapCodec<A>) -> Self {
        Self { codec }
    }
}

impl<A> Clone for KeyDispatchDataCodec<A> {
    fn clone(&self) -> Self {
        Self {
            codec: self.codec.clone(),
        }
    }
}

impl<A> PartialEq for KeyDispatchDataCodec<A> {
    fn eq(&self, other: &Self) -> bool {
        self.codec == other.codec
    }
}

impl<A> Eq for KeyDispatchDataCodec<A> {}

impl<A> Hash for KeyDispatchDataCodec<A> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.codec.hash(state);
    }
}

impl<A> fmt::Debug for KeyDispatchDataCodec<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyDispatchDataCodec")
            .field("codec", &self.codec)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn hash_value<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn of_stores_codec() {
        let codec = MapCodec::<u32>::with_id(42);
        let dispatch = KeyDispatchDataCodec::of(codec.clone());
        assert_eq!(dispatch.codec(), &codec);
    }

    #[test]
    fn codec_accessor_returns_component_reference() {
        let codec = MapCodec::<String>::with_id(7);
        let dispatch = KeyDispatchDataCodec::of(codec.clone());
        let got: &MapCodec<String> = dispatch.codec();
        assert_eq!(got, &codec);
        assert_eq!(got.id(), 7);
    }

    #[test]
    fn clone_preserves_equality() {
        let dispatch = KeyDispatchDataCodec::of(MapCodec::<i32>::with_id(-99));
        let cloned = dispatch.clone();
        assert_eq!(dispatch, cloned);
    }

    #[test]
    fn different_codecs_are_not_equal() {
        let a = KeyDispatchDataCodec::of(MapCodec::<u8>::with_id(1));
        let b = KeyDispatchDataCodec::of(MapCodec::<u8>::with_id(2));
        assert_ne!(a, b);
    }

    #[test]
    fn debug_contains_type_and_codec_information() {
        let dispatch = KeyDispatchDataCodec::of(MapCodec::<()>::with_id(123));
        let dbg = format!("{dispatch:?}");
        assert!(dbg.contains("KeyDispatchDataCodec"));
        assert!(dbg.contains("MapCodec"));
        assert!(dbg.contains("123"));
    }

    #[test]
    fn hash_is_consistent_for_equal_values() {
        let a = KeyDispatchDataCodec::of(MapCodec::<u64>::with_id(555));
        let b = KeyDispatchDataCodec::of(MapCodec::<u64>::with_id(555));
        assert_eq!(a, b);
        assert_eq!(hash_value(&a), hash_value(&b));
    }

    #[test]
    fn negative_integer_edge_cases() {
        let min_dispatch = KeyDispatchDataCodec::of(MapCodec::<i64>::with_id(i64::MIN));
        assert_eq!(min_dispatch.codec().id(), i64::MIN);

        let neg_one_dispatch = KeyDispatchDataCodec::of(MapCodec::<i32>::with_id(-1));
        assert_eq!(neg_one_dispatch.codec().id(), -1);

        let neg_shifted = -1_i64 << (40_i32 & 63);
        let shifted_dispatch = KeyDispatchDataCodec::of(MapCodec::<i64>::with_id(neg_shifted));
        assert_eq!(shifted_dispatch.codec().id(), neg_shifted);
    }

    #[test]
    fn overflow_wrapping_edge_cases() {
        let wrapped_i64 = i64::MAX.wrapping_add(1);
        assert_eq!(wrapped_i64, i64::MIN);
        let dispatch_i64 =
            KeyDispatchDataCodec::of(MapCodec::<u128>::with_id(wrapped_i64));
        assert_eq!(dispatch_i64.codec().id(), i64::MIN);

        let wrapped_u32 = u32::MAX.wrapping_add(1);
        assert_eq!(wrapped_u32, 0_u32);
        let dispatch_u32 =
            KeyDispatchDataCodec::of(MapCodec::<u32>::with_id(i64::from(wrapped_u32)));
        assert_eq!(dispatch_u32.codec().id(), 0_i64);

        let wrapped_mul = 3_i64.wrapping_mul(i64::MAX);
        let dispatch_mul = KeyDispatchDataCodec::of(MapCodec::<i64>::with_id(wrapped_mul));
        assert_eq!(dispatch_mul.codec().id(), wrapped_mul);
    }

    #[test]
    fn float_bit_pattern_edge_cases() {
        let nan_f32_bits = f32::NAN.to_bits() as i64;
        let nan_f32_dispatch = KeyDispatchDataCodec::of(MapCodec::<f32>::with_id(nan_f32_bits));
        assert_eq!(nan_f32_dispatch.codec().id(), nan_f32_bits);

        let neg_zero_f64_bits = (-0.0_f64).to_bits() as i64;
        let pos_zero_f64_bits = 0.0_f64.to_bits() as i64;
        assert_ne!(neg_zero_f64_bits, pos_zero_f64_bits);

        let neg_zero_dispatch =
            KeyDispatchDataCodec::of(MapCodec::<f64>::with_id(neg_zero_f64_bits));
        let pos_zero_dispatch =
            KeyDispatchDataCodec::of(MapCodec::<f64>::with_id(pos_zero_f64_bits));
        assert_ne!(neg_zero_dispatch, pos_zero_dispatch);

        let inf_bits = f64::INFINITY.to_bits() as i64;
        let neg_inf_bits = f64::NEG_INFINITY.to_bits() as i64;
        let inf_dispatch = KeyDispatchDataCodec::of(MapCodec::<f64>::with_id(inf_bits));
        let neg_inf_dispatch = KeyDispatchDataCodec::of(MapCodec::<f64>::with_id(neg_inf_bits));
        assert_eq!(inf_dispatch.codec().id(), inf_bits);
        assert_eq!(neg_inf_dispatch.codec().id(), neg_inf_bits);
        assert_ne!(inf_dispatch, neg_inf_dispatch);

        let nan_f64_bits = f64::NAN.to_bits() as i64;
        let nan_f64_dispatch = KeyDispatchDataCodec::of(MapCodec::<f64>::with_id(nan_f64_bits));
        assert_eq!(nan_f64_dispatch.codec().id(), nan_f64_bits);
    }

    #[test]
    fn generic_type_parameter_is_preserved() {
        fn assert_codec_type<A: 'static>(_dispatch: &KeyDispatchDataCodec<A>) {}

        let dispatch_vec = KeyDispatchDataCodec::of(MapCodec::<Vec<u8>>::with_id(0));
        assert_codec_type::<Vec<u8>>(&dispatch_vec);

        let dispatch_tuple = KeyDispatchDataCodec::of(MapCodec::<(i32, f64)>::with_id(-7));
        assert_codec_type::<(i32, f64)>(&dispatch_tuple);
        assert_eq!(dispatch_tuple.codec().id(), -7);
    }
}
