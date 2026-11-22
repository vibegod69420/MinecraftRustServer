use std::io::{self, Read, Write};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueResult {
    Continue,
    Break,
}

pub trait TagVisitor {
    fn visit_short(&mut self, tag: &ShortTag);
}

pub trait StreamTagVisitor {
    fn visit_short(&mut self, value: i16) -> ValueResult;
}

#[derive(Debug, Default)]
pub struct StringTagVisitor {
    out: String,
}

impl StringTagVisitor {
    pub fn new() -> Self {
        Self { out: String::new() }
    }

    pub fn build(self) -> String {
        self.out
    }
}

impl TagVisitor for StringTagVisitor {
    // JAVA: net.minecraft.nbt.StringTagVisitor#visitShort
    fn visit_short(&mut self, tag: &ShortTag) {
        self.out.push_str(&tag.value().to_string());
        self.out.push('s');
    }
}

impl StreamTagVisitor for StringTagVisitor {
    // JAVA: net.minecraft.nbt.StringTagVisitor#visitShort (stream)
    fn visit_short(&mut self, value: i16) -> ValueResult {
        self.out.push_str(&value.to_string());
        self.out.push('s');
        ValueResult::Continue
    }
}

pub struct NbtAccounter {
    quota: u64,
}

impl NbtAccounter {
    pub fn new(quota: u64) -> Self {
        Self { quota }
    }

    // JAVA: net.minecraft.nbt.NbtAccounter#accountBytes
    pub fn account_bytes(&mut self, bytes: u64) -> io::Result<()> {
        if self.quota < bytes {
            self.quota = 0;
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "NbtAccounter overflow",
            ));
        }
        self.quota -= bytes;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShortTag {
    value: i16,
}

impl ShortTag {
    const SELF_SIZE_IN_BYTES: i32 = 10;
    const CACHE_LOW: i16 = -128;
    const CACHE_HIGH: i16 = 1024;
    const CACHE_LEN: usize = (Self::CACHE_HIGH as i32 - Self::CACHE_LOW as i32 + 1) as usize;

    // JAVA: net.minecraft.nbt.ShortTag#<init>
    pub fn new(value: i16) -> Self {
        Self { value }
    }

    // JAVA: net.minecraft.nbt.ShortTag#valueOf
    pub fn value_of(value: i16) -> Self {
        if value >= Self::CACHE_LOW && value <= Self::CACHE_HIGH {
            short_cache()[(value as i32 - Self::CACHE_LOW as i32) as usize]
        } else {
            Self::new(value)
        }
    }

    // JAVA: net.minecraft.nbt.ShortTag#write
    pub fn write<W: Write>(&self, out: &mut W) -> io::Result<()> {
        out.write_all(&self.value.to_be_bytes())
    }

    // JAVA: net.minecraft.nbt.ShortTag#sizeInBytes
    pub fn size_in_bytes(&self) -> i32 {
        Self::SELF_SIZE_IN_BYTES
    }

    // JAVA: net.minecraft.nbt.ShortTag#getId
    pub fn get_id(&self) -> u8 {
        2
    }

    // JAVA: net.minecraft.nbt.ShortTag#getType
    pub fn get_type(&self) -> &'static ShortTagType {
        short_tag_type()
    }

    // JAVA: net.minecraft.nbt.ShortTag#copy
    pub fn copy(&self) -> Self {
        *self
    }

    // JAVA: net.minecraft.nbt.ShortTag#accept
    pub fn accept<V: TagVisitor>(&self, visitor: &mut V) {
        visitor.visit_short(self);
    }

    // JAVA: net.minecraft.nbt.ShortTag#longValue
    pub fn long_value(&self) -> i64 {
        self.value as i64
    }

    // JAVA: net.minecraft.nbt.ShortTag#intValue
    pub fn int_value(&self) -> i32 {
        self.value as i32
    }

    // JAVA: net.minecraft.nbt.ShortTag#shortValue
    pub fn short_value(&self) -> i16 {
        self.value
    }

    // JAVA: net.minecraft.nbt.ShortTag#byteValue
    pub fn byte_value(&self) -> i8 {
        (self.value as i32 & 0xFF) as u8 as i8
    }

    // JAVA: net.minecraft.nbt.ShortTag#doubleValue
    pub fn double_value(&self) -> f64 {
        self.value as f64
    }

    // JAVA: net.minecraft.nbt.ShortTag#floatValue
    pub fn float_value(&self) -> f32 {
        self.value as f32
    }

    // JAVA: net.minecraft.nbt.ShortTag#box
    pub fn box_value(&self) -> i16 {
        self.value
    }

    // JAVA: net.minecraft.nbt.ShortTag#accept
    pub fn accept_stream<V: StreamTagVisitor>(&self, visitor: &mut V) -> ValueResult {
        visitor.visit_short(self.value)
    }

    // JAVA: net.minecraft.nbt.ShortTag#toString
    pub fn to_string_value(&self) -> String {
        let mut v = StringTagVisitor::new();
        TagVisitor::visit_short(&mut v, self);
        v.build()
    }

    pub fn value(&self) -> i16 {
        self.value
    }
}

impl std::fmt::Display for ShortTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string_value())
    }
}

pub struct ShortTagType;

impl ShortTagType {
    // JAVA: net.minecraft.nbt.ShortTag$TYPE#load
    pub fn load<R: Read>(&self, input: &mut R, acc: &mut NbtAccounter) -> io::Result<ShortTag> {
        Ok(ShortTag::value_of(Self::read_accounted(input, acc)?))
    }

    // JAVA: net.minecraft.nbt.ShortTag$TYPE#parse
    pub fn parse<R: Read, V: StreamTagVisitor>(
        &self,
        input: &mut R,
        visitor: &mut V,
        acc: &mut NbtAccounter,
    ) -> io::Result<ValueResult> {
        Ok(visitor.visit_short(Self::read_accounted(input, acc)?))
    }

    // JAVA: net.minecraft.nbt.ShortTag$TYPE#readAccounted
    fn read_accounted<R: Read>(input: &mut R, acc: &mut NbtAccounter) -> io::Result<i16> {
        acc.account_bytes(10)?;
        let mut buf = [0u8; 2];
        input.read_exact(&mut buf)?;
        Ok(i16::from_be_bytes(buf))
    }

    // JAVA: net.minecraft.nbt.ShortTag$TYPE#size
    pub fn size(&self) -> i32 {
        2
    }

    // JAVA: net.minecraft.nbt.ShortTag$TYPE#getName
    pub fn get_name(&self) -> &'static str {
        "SHORT"
    }

    // JAVA: net.minecraft.nbt.ShortTag$TYPE#getPrettyName
    pub fn get_pretty_name(&self) -> &'static str {
        "TAG_Short"
    }
}

static SHORT_TAG_TYPE: OnceLock<ShortTagType> = OnceLock::new();

pub fn short_tag_type() -> &'static ShortTagType {
    SHORT_TAG_TYPE.get_or_init(|| ShortTagType)
}

static SHORT_CACHE: OnceLock<Vec<ShortTag>> = OnceLock::new();

fn short_cache() -> &'static [ShortTag] {
    SHORT_CACHE
        .get_or_init(|| {
            let mut v = Vec::with_capacity(ShortTag::CACHE_LEN);
            let mut i = 0i32;
            while i < ShortTag::CACHE_LEN as i32 {
                v.push(ShortTag::new((ShortTag::CACHE_LOW as i32 + i) as i16));
                i += 1;
            }
            v
        })
        .as_slice()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn value_of_cache_and_outside() {
        let a = ShortTag::value_of(-128);
        let b = ShortTag::value_of(-128);
        assert_eq!(a, b);
        assert_eq!(a.value(), -128);

        let c = ShortTag::value_of(1024);
        assert_eq!(c.value(), 1024);

        let d = ShortTag::value_of(1025);
        let e = ShortTag::value_of(-129);
        assert_eq!(d.value(), 1025);
        assert_eq!(e.value(), -129);
    }

    #[test]
    fn numeric_conversions_and_masking() {
        let tag = ShortTag::new(-1);
        assert_eq!(tag.long_value(), -1i64);
        assert_eq!(tag.int_value(), -1);
        assert_eq!(tag.short_value(), -1);
        assert_eq!(tag.byte_value(), -1i8); // 0xFF as i8
        assert_eq!(tag.double_value(), -1.0f64);
        assert_eq!(tag.float_value(), -1.0f32);
        assert_eq!(tag.box_value(), -1);

        let tag2 = ShortTag::new(0x1234);
        assert_eq!(tag2.byte_value(), 0x34u8 as i8);
    }

    #[test]
    fn write_read_and_type_metadata() {
        let tag = ShortTag::value_of(1234);
        let mut buf = Vec::new();
        tag.write(&mut buf).unwrap();
        assert_eq!(buf, vec![0x04, 0xD2]);
        assert_eq!(tag.get_id(), 2);
        assert_eq!(tag.size_in_bytes(), 10);
        assert_eq!(tag.get_type().get_name(), "SHORT");
        assert_eq!(tag.get_type().get_pretty_name(), "TAG_Short");
        assert_eq!(tag.get_type().size(), 2);

        let mut acc = NbtAccounter::new(10);
        let loaded = short_tag_type().load(&mut Cursor::new(buf), &mut acc).unwrap();
        assert_eq!(loaded, tag);
    }

    #[test]
    fn parse_to_string_and_stream_accept() {
        let tag = ShortTag::value_of(-7);
        assert_eq!(tag.to_string_value(), "-7s");
        assert_eq!(tag.to_string(), "-7s");

        let mut visitor = StringTagVisitor::new();
        let mut acc = NbtAccounter::new(10);
        let res = short_tag_type()
            .parse(&mut Cursor::new(vec![0xFF, 0xF9]), &mut visitor, &mut acc)
            .unwrap();
        assert_eq!(res, ValueResult::Continue);
        assert_eq!(visitor.build(), "-7s");

        let mut v2 = StringTagVisitor::new();
        assert_eq!(tag.accept_stream(&mut v2), ValueResult::Continue);
        assert_eq!(v2.build(), "-7s");
    }

    #[test]
    fn accounter_overflow_and_copy() {
        let mut acc = NbtAccounter::new(9);
        let err = short_tag_type().load(&mut Cursor::new(vec![0, 1]), &mut acc);
        assert!(err.is_err());
        let tag = ShortTag::value_of(5);
        assert_eq!(tag.copy(), tag);
    }
}
