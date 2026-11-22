use crate::NbtTag;

/// Trait for numeric NBT tags, mirroring `net.minecraft.nbt.NumericTag`.
/// In this self-contained module we provide an extension trait over `NbtTag`
/// and helper conversion methods matching Java semantics.
pub trait NumericTag {
    fn byte_value(&self) -> i8;
    fn short_value(&self) -> i16;
    fn int_value(&self) -> i32;
    fn long_value(&self) -> i64;
    fn float_value(&self) -> f32;
    fn double_value(&self) -> f64;
    fn box_number(&self) -> NumericBox;

    // JAVA: net.minecraft.nbt.NumericTag#asNumber
    fn as_number(&self) -> Option<NumericBox> {
        Some(self.box_number())
    }

    // JAVA: net.minecraft.nbt.NumericTag#asByte
    fn as_byte(&self) -> Option<i8> {
        Some(self.byte_value())
    }

    // JAVA: net.minecraft.nbt.NumericTag#asShort
    fn as_short(&self) -> Option<i16> {
        Some(self.short_value())
    }

    // JAVA: net.minecraft.nbt.NumericTag#asInt
    fn as_int(&self) -> Option<i32> {
        Some(self.int_value())
    }

    // JAVA: net.minecraft.nbt.NumericTag#asLong
    fn as_long(&self) -> Option<i64> {
        Some(self.long_value())
    }

    // JAVA: net.minecraft.nbt.NumericTag#asFloat
    fn as_float(&self) -> Option<f32> {
        Some(self.float_value())
    }

    // JAVA: net.minecraft.nbt.NumericTag#asDouble
    fn as_double(&self) -> Option<f64> {
        Some(self.double_value())
    }

    // JAVA: net.minecraft.nbt.NumericTag#asBoolean
    fn as_boolean(&self) -> Option<bool> {
        Some(self.byte_value() != 0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumericBox {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
}

impl NumericTag for NbtTag {
    // JAVA: net.minecraft.nbt.NumericTag#byteValue
    fn byte_value(&self) -> i8 {
        match self {
            NbtTag::Byte(v) => *v,
            NbtTag::Short(v) => *v as i8,
            NbtTag::Int(v) => *v as i8,
            NbtTag::Long(v) => *v as i8,
            NbtTag::Float(v) => *v as i8,
            NbtTag::Double(v) => *v as i8,
            _ => 0,
        }
    }

    // JAVA: net.minecraft.nbt.NumericTag#shortValue
    fn short_value(&self) -> i16 {
        match self {
            NbtTag::Byte(v) => *v as i16,
            NbtTag::Short(v) => *v,
            NbtTag::Int(v) => *v as i16,
            NbtTag::Long(v) => *v as i16,
            NbtTag::Float(v) => *v as i16,
            NbtTag::Double(v) => *v as i16,
            _ => 0,
        }
    }

    // JAVA: net.minecraft.nbt.NumericTag#intValue
    fn int_value(&self) -> i32 {
        match self {
            NbtTag::Byte(v) => *v as i32,
            NbtTag::Short(v) => *v as i32,
            NbtTag::Int(v) => *v,
            NbtTag::Long(v) => *v as i32,
            NbtTag::Float(v) => *v as i32,
            NbtTag::Double(v) => *v as i32,
            _ => 0,
        }
    }

    // JAVA: net.minecraft.nbt.NumericTag#longValue
    fn long_value(&self) -> i64 {
        match self {
            NbtTag::Byte(v) => *v as i64,
            NbtTag::Short(v) => *v as i64,
            NbtTag::Int(v) => *v as i64,
            NbtTag::Long(v) => *v,
            NbtTag::Float(v) => *v as i64,
            NbtTag::Double(v) => *v as i64,
            _ => 0,
        }
    }

    // JAVA: net.minecraft.nbt.NumericTag#floatValue
    fn float_value(&self) -> f32 {
        match self {
            NbtTag::Byte(v) => *v as f32,
            NbtTag::Short(v) => *v as f32,
            NbtTag::Int(v) => *v as f32,
            NbtTag::Long(v) => *v as f32,
            NbtTag::Float(v) => *v,
            NbtTag::Double(v) => *v as f32,
            _ => 0.0,
        }
    }

    // JAVA: net.minecraft.nbt.NumericTag#doubleValue
    fn double_value(&self) -> f64 {
        match self {
            NbtTag::Byte(v) => *v as f64,
            NbtTag::Short(v) => *v as f64,
            NbtTag::Int(v) => *v as f64,
            NbtTag::Long(v) => *v as f64,
            NbtTag::Float(v) => *v as f64,
            NbtTag::Double(v) => *v,
            _ => 0.0,
        }
    }

    // JAVA: net.minecraft.nbt.NumericTag#box
    fn box_number(&self) -> NumericBox {
        match self {
            NbtTag::Byte(v) => NumericBox::Byte(*v),
            NbtTag::Short(v) => NumericBox::Short(*v),
            NbtTag::Int(v) => NumericBox::Int(*v),
            NbtTag::Long(v) => NumericBox::Long(*v),
            NbtTag::Float(v) => NumericBox::Float(*v),
            NbtTag::Double(v) => NumericBox::Double(*v),
            _ => NumericBox::Int(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_conversions_and_defaults() {
        let tag = NbtTag::Byte(-1);
        assert_eq!(tag.byte_value(), -1);
        assert_eq!(tag.short_value(), -1);
        assert_eq!(tag.int_value(), -1);
        assert_eq!(tag.long_value(), -1);
        assert_eq!(tag.as_boolean(), Some(true));
        assert_eq!(tag.as_number(), Some(NumericBox::Byte(-1)));
    }

    #[test]
    fn long_to_int_wrapping_like_cast() {
        let tag = NbtTag::Long(i64::MAX);
        assert_eq!(tag.int_value(), i64::MAX as i32);
        assert_eq!(tag.byte_value(), i64::MAX as i8);
        assert_eq!(tag.short_value(), i64::MAX as i16);
    }

    #[test]
    fn float_double_bits_and_boolean() {
        let f = NbtTag::Float(f32::NAN);
        assert!(f.float_value().is_nan());
        assert_eq!(f.as_boolean(), Some(f.byte_value() != 0));

        let d = NbtTag::Double(-0.0);
        assert_eq!(d.double_value().to_bits(), (-0.0f64).to_bits());
        assert_eq!(d.as_float(), Some(-0.0f32));
        assert_eq!(d.as_boolean(), Some(false));
    }

    #[test]
    fn non_numeric_defaults() {
        let tag = NbtTag::String("x".to_string());
        assert_eq!(tag.byte_value(), 0);
        assert_eq!(tag.int_value(), 0);
        assert_eq!(tag.double_value(), 0.0);
        assert_eq!(tag.box_number(), NumericBox::Int(0));
    }
}
