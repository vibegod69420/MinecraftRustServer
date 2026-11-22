//! Port of `net.minecraft.network.protocol.CodecModifier`.

/// Minimal self-contained placeholder for `net.minecraft.network.codec.StreamCodec`.
pub trait StreamCodec<B, V> {}

/// Port of the Java functional interface `CodecModifier<B, V, C>`.
pub trait CodecModifier<B, V, C> {
    // JAVA: net.minecraft.network.protocol.CodecModifier#apply
    fn apply(
        &self,
        codec: Box<dyn StreamCodec<B, V>>,
        context: C,
    ) -> Box<dyn StreamCodec<B, V>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    struct IntCodec {
        value: i32,
    }
    impl StreamCodec<Vec<u8>, i32> for IntCodec {}

    struct IdentityModifier;
    impl CodecModifier<Vec<u8>, i32, ()> for IdentityModifier {
        fn apply(
            &self,
            codec: Box<dyn StreamCodec<Vec<u8>, i32>>,
            _context: (),
        ) -> Box<dyn StreamCodec<Vec<u8>, i32>> {
            codec
        }
    }

    struct ReplaceModifier;
    impl CodecModifier<Vec<u8>, i32, i32> for ReplaceModifier {
        fn apply(
            &self,
            _codec: Box<dyn StreamCodec<Vec<u8>, i32>>,
            context: i32,
        ) -> Box<dyn StreamCodec<Vec<u8>, i32>> {
            Box::new(IntCodec { value: context })
        }
    }

    struct ContextCaptureModifier {
        seen: Rc<Cell<i32>>,
    }
    impl CodecModifier<Vec<u8>, i32, i32> for ContextCaptureModifier {
        fn apply(
            &self,
            codec: Box<dyn StreamCodec<Vec<u8>, i32>>,
            context: i32,
        ) -> Box<dyn StreamCodec<Vec<u8>, i32>> {
            self.seen.set(context);
            codec
        }
    }

    #[test]
    fn identity_modifier_returns_codec() {
        let modifier = IdentityModifier;
        let codec: Box<dyn StreamCodec<Vec<u8>, i32>> = Box::new(IntCodec { value: 42 });
        let _result = modifier.apply(codec, ());
    }

    #[test]
    fn replace_modifier_produces_new_codec() {
        let modifier = ReplaceModifier;
        let codec: Box<dyn StreamCodec<Vec<u8>, i32>> = Box::new(IntCodec { value: 1 });
        let _result = modifier.apply(codec, 99);
    }

    #[test]
    fn modifier_receives_context_value() {
        let seen = Rc::new(Cell::new(0_i32));
        let modifier = ContextCaptureModifier { seen: seen.clone() };
        let codec: Box<dyn StreamCodec<Vec<u8>, i32>> = Box::new(IntCodec { value: 0 });
        let _result = modifier.apply(codec, -12345);
        assert_eq!(seen.get(), -12345);
    }

    #[test]
    fn modifier_receives_extreme_context_values() {
        let seen = Rc::new(Cell::new(0_i32));
        let modifier = ContextCaptureModifier { seen: seen.clone() };

        let codec: Box<dyn StreamCodec<Vec<u8>, i32>> = Box::new(IntCodec { value: 0 });
        let _ = modifier.apply(codec, i32::MAX);
        assert_eq!(seen.get(), i32::MAX);

        let codec: Box<dyn StreamCodec<Vec<u8>, i32>> = Box::new(IntCodec { value: 0 });
        let _ = modifier.apply(codec, i32::MIN);
        assert_eq!(seen.get(), i32::MIN);
    }

    #[test]
    fn modifier_can_be_used_as_trait_object() {
        let modifier: Box<dyn CodecModifier<Vec<u8>, i32, ()>> = Box::new(IdentityModifier);
        let codec: Box<dyn StreamCodec<Vec<u8>, i32>> = Box::new(IntCodec { value: 7 });
        let _result = modifier.apply(codec, ());
    }
}
