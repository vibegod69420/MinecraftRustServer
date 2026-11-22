use std::io::{self, Read, Write};
use std::sync::LazyLock;

const SELF_SIZE_IN_BYTES: i32 = 9;

/// Minimal account counter stub for self-contained NBT loading.
#[derive(Debug, Default)]
pub struct NbtAccounter {
    pub bytes_accounted: u64,
}

impl NbtAccounter {
    // JAVA: net.minecraft.nbt.NbtAccounter#accountBytes
    pub fn account_bytes(&mut self, bytes: u64) -> Result<(), io::Error> {
        self.bytes_accounted = self.bytes_accounted.saturating_add(bytes);
        Ok(())
    }
}

/// Minimal stream visitor result for self-contained parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamTagValueResult {
    Continue,
    Break,
}

/// Minimal tag visitor stub.
pub trait TagVisitor {
    // JAVA: net.minecraft.nbt.TagVisitor#visitByte
    fn visit_byte(&mut self, tag: &ByteTag);
}

/// Minimal stream tag visitor stub.
pub trait StreamTagVisitor {
    // JAVA: net.minecraft.nbt.StreamTagVisitor#visit
    fn visit(&mut self, value: i8) -> StreamTagValueResult;
}

/// Port of `net.minecraft.nbt.ByteTag`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ByteTag {
    pub value: i8,
}

// JAVA: net.minecraft.nbt.ByteTag$Cache#<clinit>
fn build_cache() -> [ByteTag; 256] {
    std::array::from_fn(|i| ByteTag {
        value: ((i as i32) - 128) as i8,
    })
}

static CACHE: LazyLock<[ByteTag; 256]> = LazyLock::new(build_cache);

pub static ZERO: LazyLock<ByteTag> = LazyLock::new(|| ByteTag::value_of(0));
pub static ONE: LazyLock<ByteTag> = LazyLock::new(|| ByteTag::value_of(1));

impl ByteTag {
    // JAVA: net.minecraft.nbt.ByteTag#<init>
    pub fn new(value: i8) -> Self {
        Self { value }
    }

    // JAVA: net.minecraft.nbt.ByteTag#valueOf
    pub fn value_of(value: i8) -> Self {
        let idx = (128i32).wrapping_add(value as i32) as usize;
        CACHE[idx]
    }

    // JAVA: net.minecraft.nbt.ByteTag#valueOf
    pub fn value_of_bool(value: bool) -> Self {
        if value {
            Self::value_of(1)
        } else {
            Self::value_of(0)
        }
    }

    // JAVA: net.minecraft.nbt.ByteTag#write
    pub fn write(&self, output: &mut impl Write) -> io::Result<()> {
        output.write_all(&[self.value as u8])
    }

    // JAVA: net.minecraft.nbt.ByteTag#sizeInBytes
    pub fn size_in_bytes(&self) -> i32 {
        SELF_SIZE_IN_BYTES
    }

    // JAVA: net.minecraft.nbt.ByteTag#getId
    pub fn get_id(&self) -> u8 {
        1
    }

    // JAVA: net.minecraft.nbt.ByteTag#copy
    pub fn copy(&self) -> Self {
        *self
    }

    // JAVA: net.minecraft.nbt.ByteTag#accept
    pub fn accept_tag_visitor(&self, visitor: &mut impl TagVisitor) {
        visitor.visit_byte(self);
    }

    // JAVA: net.minecraft.nbt.ByteTag#longValue
    pub fn long_value(&self) -> i64 {
        self.value as i64
    }

    // JAVA: net.minecraft.nbt.ByteTag#intValue
    pub fn int_value(&self) -> i32 {
        self.value as i32
    }

    // JAVA: net.minecraft.nbt.ByteTag#shortValue
    pub fn short_value(&self) -> i16 {
        self.value as i16
    }

    // JAVA: net.minecraft.nbt.ByteTag#byteValue
    pub fn byte_value(&self) -> i8 {
        self.value
    }

    // JAVA: net.minecraft.nbt.ByteTag#doubleValue
    pub fn double_value(&self) -> f64 {
        self.value as f64
    }

    // JAVA: net.minecraft.nbt.ByteTag#floatValue
    pub fn float_value(&self) -> f32 {
        self.value as f32
    }

    // JAVA: net.minecraft.nbt.ByteTag#box
    pub fn boxed_number(&self) -> i8 {
        self.value
    }

    // JAVA: net.minecraft.nbt.ByteTag#accept
    pub fn accept_stream_visitor(&self, visitor: &mut impl StreamTagVisitor) -> StreamTagValueResult {
        visitor.visit(self.value)
    }

    // JAVA: net.minecraft.nbt.ByteTag#toString
    pub fn to_snbt_string(&self) -> String {
        format!("{}b", self.value)
    }
}

impl std::fmt::Display for ByteTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_snbt_string())
    }
}

/// Port of `ByteTag.TYPE` (anonymous `TagType.StaticSize<ByteTag>`).
pub struct ByteTagType;

impl ByteTagType {
    // JAVA: net.minecraft.nbt.ByteTag$1#readAccounted
    fn read_accounted(input: &mut impl Read, accounter: &mut NbtAccounter) -> io::Result<i8> {
        accounter.account_bytes(9)?;
        let mut buf = [0u8; 1];
        input.read_exact(&mut buf)?;
        Ok(buf[0] as i8)
    }

    // JAVA: net.minecraft.nbt.ByteTag$1#load
    pub fn load(input: &mut impl Read, accounter: &mut NbtAccounter) -> io::Result<ByteTag> {
        Ok(ByteTag::value_of(Self::read_accounted(input, accounter)?))
    }

    // JAVA: net.minecraft.nbt.ByteTag$1#parse
    pub fn parse(
        input: &mut impl Read,
        visitor: &mut impl StreamTagVisitor,
        accounter: &mut NbtAccounter,
    ) -> io::Result<StreamTagValueResult> {
        Ok(visitor.visit(Self::read_accounted(input, accounter)?))
    }

    // JAVA: net.minecraft.nbt.ByteTag$1#size
    pub fn size() -> i32 {
        1
    }

    // JAVA: net.minecraft.nbt.ByteTag$1#getName
    pub fn get_name() -> &'static str {
        "BYTE"
    }

    // JAVA: net.minecraft.nbt.ByteTag$1#getPrettyName
    pub fn get_pretty_name() -> &'static str {
        "TAG_Byte"
    }
}

pub static TYPE: ByteTagType = ByteTagType;

#[cfg(test)]
mod tests {
    use super::*;

    struct ByteCollector(i8);
    impl StreamTagVisitor for ByteCollector {
        fn visit(&mut self, value: i8) -> StreamTagValueResult {
            self.0 = value;
            StreamTagValueResult::Continue
        }
    }

    struct TagByteCollector(i8);
    impl TagVisitor for TagByteCollector {
        fn visit_byte(&mut self, tag: &ByteTag) {
            self.0 = tag.value;
        }
    }

    #[test]
    fn value_of_cache_bounds_and_bool() {
        assert_eq!(ByteTag::value_of(i8::MIN).value, -128);
        assert_eq!(ByteTag::value_of(i8::MAX).value, 127);
        assert_eq!(ByteTag::value_of(0).value, 0);
        assert_eq!(ByteTag::value_of_bool(false), *ZERO);
        assert_eq!(ByteTag::value_of_bool(true), *ONE);
    }

    #[test]
    fn numeric_conversions_negative_and_overflow_semantics() {
        let tag = ByteTag::value_of(-1);
        assert_eq!(tag.byte_value(), -1i8);
        assert_eq!(tag.short_value(), -1i16);
        assert_eq!(tag.int_value(), -1i32);
        assert_eq!(tag.long_value(), -1i64);
        assert_eq!(tag.boxed_number(), -1i8);
        let wrapped_idx = (128i32).wrapping_add(i8::MIN as i32) as usize;
        assert_eq!(wrapped_idx, 0);
    }

    #[test]
    fn float_and_double_bit_patterns() {
        let tag = ByteTag::value_of(-128);
        assert_eq!(tag.float_value().to_bits(), (-128.0f32).to_bits());
        assert_eq!(tag.double_value().to_bits(), (-128.0f64).to_bits());
        let zero = ByteTag::value_of(0);
        assert_eq!(zero.float_value().to_bits(), 0.0f32.to_bits());
        assert_eq!(zero.double_value().to_bits(), 0.0f64.to_bits());
    }

    #[test]
    fn write_load_parse_and_metadata() {
        let tag = ByteTag::value_of(42);
        let mut buf = Vec::new();
        tag.write(&mut buf).unwrap();
        assert_eq!(buf, vec![42u8]);
        assert_eq!(tag.size_in_bytes(), 9);
        assert_eq!(tag.get_id(), 1);
        assert_eq!(ByteTagType::size(), 1);
        assert_eq!(ByteTagType::get_name(), "BYTE");
        assert_eq!(ByteTagType::get_pretty_name(), "TAG_Byte");

        let mut acc = NbtAccounter::default();
        let loaded = ByteTagType::load(&mut &buf[..], &mut acc).unwrap();
        assert_eq!(loaded, tag);
        assert_eq!(acc.bytes_accounted, 9);

        let mut acc2 = NbtAccounter::default();
        let mut visitor = ByteCollector(0);
        let res = ByteTagType::parse(&mut &buf[..], &mut visitor, &mut acc2).unwrap();
        assert_eq!(res, StreamTagValueResult::Continue);
        assert_eq!(visitor.0, 42);
    }

    #[test]
    fn visitors_copy_and_snbt_string() {
        let tag = ByteTag::value_of(-5);
        let mut tv = TagByteCollector(0);
        tag.accept_tag_visitor(&mut tv);
        assert_eq!(tv.0, -5);

        let mut sv = ByteCollector(0);
        assert_eq!(tag.accept_stream_visitor(&mut sv), StreamTagValueResult::Continue);
        assert_eq!(sv.0, -5);

        let copied = tag.copy();
        assert_eq!(copied, tag);
        assert_eq!(tag.to_snbt_string(), "-5b");
        assert_eq!(tag.to_string(), "-5b");
    }
}
