/// Self-contained tag marker for primitive NBT tags.
// JAVA: net.minecraft.nbt.Tag
pub trait Tag: Clone {
    // JAVA: net.minecraft.nbt.Tag#copy
    fn copy_tag(&self) -> Self
    where
        Self: Sized,
    {
        self.clone()
    }
}

/// Primitive NBT tag marker.
/// In Java this is `sealed interface PrimitiveTag extends Tag permits NumericTag, StringTag`.
/// The default `copy()` returns `this`; in Rust we model copy as cloning the value.
// JAVA: net.minecraft.nbt.PrimitiveTag
pub trait PrimitiveTag: Tag {
    // JAVA: net.minecraft.nbt.PrimitiveTag#copy
    fn copy(&self) -> Self
    where
        Self: Sized,
    {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct DummyString(String);

    impl Tag for DummyString {}
    impl PrimitiveTag for DummyString {}

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct DummyInt(i32);

    impl Tag for DummyInt {}
    impl PrimitiveTag for DummyInt {}

    #[test]
    fn primitive_copy_returns_same_value() {
        let s = DummyString("hello".to_string());
        let c = PrimitiveTag::copy(&s);
        assert_eq!(s, c);
    }

    #[test]
    fn tag_copy_tag_returns_same_value() {
        let n = DummyInt(-42);
        let c = n.copy_tag();
        assert_eq!(n, c);
        assert_eq!(c.0, -42);
    }

    #[test]
    fn primitive_copy_preserves_float_like_bits_via_wrapper() {
        #[derive(Debug, Clone, Copy, PartialEq)]
        struct DummyFloat(f32);
        impl Tag for DummyFloat {}
        impl PrimitiveTag for DummyFloat {}

        let nan = DummyFloat(f32::NAN);
        let c = PrimitiveTag::copy(&nan);
        assert!(c.0.is_nan());
        let neg_zero = DummyFloat(-0.0f32);
        let c2 = PrimitiveTag::copy(&neg_zero);
        assert_eq!(c2.0.to_bits(), (-0.0f32).to_bits());
    }

    #[test]
    fn primitive_copy_is_independent_clone() {
        let mut s = DummyString("a".to_string());
        let c = PrimitiveTag::copy(&s);
        s.0.push('b');
        assert_eq!(c.0, "a");
        assert_eq!(s.0, "ab");
    }
}
