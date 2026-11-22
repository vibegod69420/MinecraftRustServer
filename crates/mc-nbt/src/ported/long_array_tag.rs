// === FILE: long_array_tag.rs ===
use std::fmt;
use std::io::{self, Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::NbtError;

pub const SELF_SIZE_IN_BYTES: i32 = 24;

/// Minimal NBT accounter for size/quota tracking during deserialization.
#[derive(Debug, Clone)]
pub struct NbtAccounter {
    quota: u64,
    usage: u64,
}

impl NbtAccounter {
    pub fn new(quota: u64) -> Self {
        Self { quota, usage: 0 }
    }

    pub fn unlimited_heap() -> Self {
        Self::new(u64::MAX / 2)
    }

    pub fn account_bytes(&mut self, bytes: u64) -> Result<(), NbtError> {
        self.usage = self.usage.saturating_add(bytes);
        if self.usage > self.quota {
            return Err(NbtError::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                "NBT tag size exceeded limit",
            )));
        }
        Ok(())
    }

    pub fn account_bytes_mul(&mut self, bytes_per: u64, count: i32) -> Result<(), NbtError> {
        if count < 0 {
            return Err(NbtError::NegativeLength(count));
        }
        let total = bytes_per.saturating_mul(count as u64);
        self.account_bytes(total)
    }
}

/// Minimal trait for tag mutation operations (`setTag`/`addTag`).
/// A numeric tag returns `Some(long_value)`, non-numeric returns `None`.
pub trait TagLike {
    fn as_numeric_long(&self) -> Option<i64>;
}

/// Minimal LongTag representation used by `get`/`remove`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LongTag {
    data: i64,
}

impl LongTag {
    // JAVA: net.minecraft.nbt.LongTag#valueOf
    pub fn value_of(value: i64) -> Self {
        Self { data: value }
    }

    pub fn long_value(&self) -> i64 {
        self.data
    }
}

impl TagLike for LongTag {
    fn as_numeric_long(&self) -> Option<i64> {
        Some(self.data)
    }
}

/// Minimal TagVisitor for `accept(TagVisitor)`.
pub trait TagVisitor {
    fn visit_long_array(&mut self, tag: &LongArrayTag);
}

/// Minimal StreamTagVisitor for `accept(StreamTagVisitor)` and TYPE parse.
pub trait StreamTagVisitor {
    type ValueResult;
    fn visit_long_array(&mut self, data: &[i64]) -> Self::ValueResult;
}

fn skip_bytes<R: Read>(reader: &mut R, mut n: i64) -> Result<(), NbtError> {
    let mut buf = [0u8; 4096];
    while n > 0 {
        let to_read = std::cmp::min(n, buf.len() as i64) as usize;
        reader.read_exact(&mut buf[..to_read])?;
        n -= to_read as i64;
    }
    Ok(())
}

struct StringTagVisitor {
    builder: String,
}

impl StringTagVisitor {
    fn new() -> Self {
        Self {
            builder: String::new(),
        }
    }

    fn visit_long_array(&mut self, tag: &LongArrayTag) {
        self.builder.push_str("[L;");
        for (i, &v) in tag.data.iter().enumerate() {
            if i != 0 {
                self.builder.push(',');
            }
            self.builder.push(' ');
            self.builder.push_str(&v.to_string());
            self.builder.push('L');
        }
        self.builder.push(']');
    }

    fn build(self) -> String {
        self.builder
    }
}

/// TagType implementation for LongArrayTag (Java anonymous TYPE).
pub struct LongArrayTagType;

impl LongArrayTagType {
    // JAVA: net.minecraft.nbt.LongArrayTag#load
    pub fn load<R: Read>(
        &self,
        reader: &mut R,
        accounter: &mut NbtAccounter,
    ) -> Result<LongArrayTag, NbtError> {
        Ok(LongArrayTag::new(self.read_accounted(reader, accounter)?))
    }

    // JAVA: net.minecraft.nbt.LongArrayTag#parse
    pub fn parse<R: Read, V: StreamTagVisitor>(
        &self,
        reader: &mut R,
        visitor: &mut V,
        accounter: &mut NbtAccounter,
    ) -> Result<V::ValueResult, NbtError> {
        let data = self.read_accounted(reader, accounter)?;
        Ok(visitor.visit_long_array(&data))
    }

    // JAVA: net.minecraft.nbt.LongArrayTag#readAccounted
    fn read_accounted<R: Read>(
        &self,
        reader: &mut R,
        accounter: &mut NbtAccounter,
    ) -> Result<Vec<i64>, NbtError> {
        accounter.account_bytes(24)?;
        let len = reader.read_i32::<BigEndian>()?;
        if len < 0 {
            return Err(NbtError::NegativeLength(len));
        }
        accounter.account_bytes_mul(8, len)?;
        let mut data = vec![0i64; len as usize];
        for i in 0..len as usize {
            data[i] = reader.read_i64::<BigEndian>()?;
        }
        Ok(data)
    }

    // JAVA: net.minecraft.nbt.LongArrayTag#skip
    pub fn skip<R: Read>(
        &self,
        reader: &mut R,
        _accounter: &mut NbtAccounter,
    ) -> Result<(), NbtError> {
        let len = reader.read_i32::<BigEndian>()?;
        let bytes = len.wrapping_mul(8);
        if bytes > 0 {
            skip_bytes(reader, bytes as i64)?;
        }
        Ok(())
    }

    // JAVA: net.minecraft.nbt.LongArrayTag#getName
    pub fn get_name(&self) -> &'static str {
        "LONG[]"
    }

    // JAVA: net.minecraft.nbt.LongArrayTag#getPrettyName
    pub fn get_pretty_name(&self) -> &'static str {
        "TAG_Long_Array"
    }
}

pub static TYPE: LongArrayTagType = LongArrayTagType;

#[derive(Clone, Eq)]
pub struct LongArrayTag {
    data: Vec<i64>,
}

impl LongArrayTag {
    // JAVA: net.minecraft.nbt.LongArrayTag#<init>
    pub fn new(data: Vec<i64>) -> Self {
        Self { data }
    }

    // JAVA: net.minecraft.nbt.LongArrayTag#write
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), NbtError> {
        writer.write_i32::<BigEndian>(self.data.len() as i32)?;
        for &v in &self.data {
            writer.write_i64::<BigEndian>(v)?;
        }
        Ok(())
    }

    // JAVA: net.minecraft.nbt.LongArrayTag#sizeInBytes
    pub fn size_in_bytes(&self) -> i32 {
        24i32.wrapping_add(8i32.wrapping_mul(self.data.len() as i32))
    }

    // JAVA: net.minecraft.nbt.LongArrayTag#getId
    pub fn get_id(&self) -> u8 {
        12
    }

    // JAVA: net.minecraft.nbt.LongArrayTag#getType
    pub fn get_type(&self) -> &'static LongArrayTagType {
        &TYPE
    }

    // JAVA: net.minecraft.nbt.LongArrayTag#toString
    pub fn to_string_value(&self) -> String {
        let mut v = StringTagVisitor::new();
        v.visit_long_array(self);
        v.build()
    }

    // JAVA: net.minecraft.nbt.LongArrayTag#copy
    pub fn copy(&self) -> LongArrayTag {
        let mut data = vec![0i64; self.data.len()];
        data.copy_from_slice(&self.data);
        LongArrayTag::new(data)
    }

    // JAVA: net.minecraft.nbt.LongArrayTag#equals
    pub fn java_equals(&self, other: &LongArrayTag) -> bool {
        self.data == other.data
    }

    // JAVA: net.minecraft.nbt.LongArrayTag#hashCode
    pub fn hash_code(&self) -> i32 {
        let mut result: i32 = 1;
        for &element in &self.data {
            let element_hash =
                ((element as u64 ^ (element as u64 >> 32)) as u32) as i32;
            result = 31i32.wrapping_mul(result).wrapping_add(element_hash);
        }
        result
    }

    // JAVA: net.minecraft.nbt.LongArrayTag#accept
    pub fn accept<V: TagVisitor>(&self, visitor: &mut V) {
        visitor.visit_long_array(self);
    }

    // JAVA: net.minecraft.nbt.LongArrayTag#getAsLongArray
    pub fn get_as_long_array(&self) -> &[i64] {
        &self.data
    }

    // JAVA: net.minecraft.nbt.LongArrayTag#size
    pub fn size(&self) -> i32 {
        self.data.len() as i32
    }

    // JAVA: net.minecraft.nbt.LongArrayTag#get
    pub fn get(&self, index: i32) -> LongTag {
        LongTag::value_of(self.data[index as usize])
    }

    // JAVA: net.minecraft.nbt.LongArrayTag#setTag
    pub fn set_tag<T: TagLike>(&mut self, index: i32, tag: &T) -> bool {
        if let Some(v) = tag.as_numeric_long() {
            self.data[index as usize] = v;
            true
        } else {
            false
        }
    }

    // JAVA: net.minecraft.nbt.LongArrayTag#addTag
    pub fn add_tag<T: TagLike>(&mut self, index: i32, tag: &T) -> bool {
        if let Some(v) = tag.as_numeric_long() {
            self.data.insert(index as usize, v);
            true
        } else {
            false
        }
    }

    // JAVA: net.minecraft.nbt.LongArrayTag#remove
    pub fn remove(&mut self, index: i32) -> LongTag {
        let v = self.data[index as usize];
        self.data.remove(index as usize);
        LongTag::value_of(v)
    }

    // JAVA: net.minecraft.nbt.LongArrayTag#clear
    pub fn clear(&mut self) {
        self.data = Vec::new();
    }

    // JAVA: net.minecraft.nbt.LongArrayTag#asLongArray
    pub fn as_long_array(&self) -> Option<&[i64]> {
        Some(&self.data)
    }

    // JAVA: net.minecraft.nbt.LongArrayTag#accept
    pub fn accept_stream<V: StreamTagVisitor>(&self, visitor: &mut V) -> V::ValueResult {
        visitor.visit_long_array(&self.data)
    }
}

impl PartialEq for LongArrayTag {
    fn eq(&self, other: &Self) -> bool {
        self.java_equals(other)
    }
}

impl fmt::Debug for LongArrayTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LongArrayTag")
            .field("data", &self.data)
            .finish()
    }
}

impl fmt::Display for LongArrayTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_string_value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    struct NumTag(i64);
    impl TagLike for NumTag {
        fn as_numeric_long(&self) -> Option<i64> {
            Some(self.0)
        }
    }

    struct StrTag;
    impl TagLike for StrTag {
        fn as_numeric_long(&self) -> Option<i64> {
            None
        }
    }

    struct CollectVisitor;
    impl StreamTagVisitor for CollectVisitor {
        type ValueResult = Vec<i64>;
        fn visit_long_array(&mut self, data: &[i64]) -> Self::ValueResult {
            data.to_vec()
        }
    }

    struct SumVisitor {
        sum: i64,
    }
    impl TagVisitor for SumVisitor {
        fn visit_long_array(&mut self, tag: &LongArrayTag) {
            for &v in tag.get_as_long_array() {
                self.sum = self.sum.wrapping_add(v);
            }
        }
    }

    #[test]
    fn roundtrip_write_read_and_visitors() {
        let tag = LongArrayTag::new(vec![1, -2, i64::MAX, i64::MIN]);
        let mut buf = Vec::new();
        tag.write(&mut buf).unwrap();

        let mut cursor = io::Cursor::new(&buf);
        let mut acc = NbtAccounter::unlimited_heap();
        let loaded = TYPE.load(&mut cursor, &mut acc).unwrap();
        assert_eq!(loaded.get_as_long_array(), &[1, -2, i64::MAX, i64::MIN]);
        assert_eq!(loaded.size(), 4);
        assert_eq!(loaded.get_id(), 12);
        assert_eq!(loaded.get_type().get_name(), "LONG[]");
        assert_eq!(loaded.get_type().get_pretty_name(), "TAG_Long_Array");

        let mut cursor2 = io::Cursor::new(&buf);
        let mut acc2 = NbtAccounter::unlimited_heap();
        let mut cv = CollectVisitor;
        let parsed = TYPE.parse(&mut cursor2, &mut cv, &mut acc2).unwrap();
        assert_eq!(parsed, vec![1, -2, i64::MAX, i64::MIN]);

        let mut sv = SumVisitor { sum: 0 };
        loaded.accept(&mut sv);
        let expected_sum = 1i64
            .wrapping_add(-2)
            .wrapping_add(i64::MAX)
            .wrapping_add(i64::MIN);
        assert_eq!(sv.sum, expected_sum);

        let collected = loaded.accept_stream(&mut CollectVisitor);
        assert_eq!(collected, loaded.get_as_long_array());
    }

    #[test]
    fn size_hash_and_equality() {
        let tag = LongArrayTag::new(vec![1, 2, 3]);
        assert_eq!(tag.size_in_bytes(), 24 + 8 * 3);
        // Java Arrays.hashCode(long[]): 1 -> 32 -> 994 -> 30817
        assert_eq!(tag.hash_code(), 30817);

        let same = LongArrayTag::new(vec![1, 2, 3]);
        let diff = LongArrayTag::new(vec![1, 2, 4]);
        assert!(tag.java_equals(&same));
        assert_eq!(tag, same);
        assert_ne!(tag, diff);
    }

    #[test]
    fn set_add_remove_clear() {
        let mut tag = LongArrayTag::new(vec![10, 20, 30]);
        assert!(tag.set_tag(1, &NumTag(99)));
        assert_eq!(tag.get_as_long_array(), &[10, 99, 30]);
        assert!(!tag.set_tag(0, &StrTag));

        assert!(tag.add_tag(1, &NumTag(55)));
        assert_eq!(tag.get_as_long_array(), &[10, 55, 99, 30]);
        assert!(!tag.add_tag(0, &StrTag));

        let removed = tag.remove(2);
        assert_eq!(removed.long_value(), 99);
        assert_eq!(tag.get_as_long_array(), &[10, 55, 30]);

        tag.clear();
        assert_eq!(tag.size(), 0);
        assert!(tag.as_long_array().unwrap().is_empty());
    }

    #[test]
    fn overflow_negative_inputs_and_skip() {
        let tag = LongArrayTag::new(vec![i64::MIN, -1, 0, i64::MAX]);
        let expected_size = 24i32.wrapping_add(8i32.wrapping_mul(4));
        assert_eq!(tag.size_in_bytes(), expected_size);
        assert_eq!(tag.hash_code(), tag.copy().hash_code());

        // Negative length should error
        let mut buf = Vec::new();
        buf.extend_from_slice(&(-1i32).to_be_bytes());
        let mut cursor = io::Cursor::new(&buf);
        let mut acc = NbtAccounter::unlimited_heap();
        assert!(TYPE.load(&mut cursor, &mut acc).is_err());

        // skip should advance by len * 8 (wrapping)
        let mut skip_buf = Vec::new();
        skip_buf.extend_from_slice(&2i32.to_be_bytes());
        skip_buf.extend_from_slice(&1i64.to_be_bytes());
        skip_buf.extend_from_slice(&2i64.to_be_bytes());
        skip_buf.extend_from_slice(&0xFFu8.to_be_bytes());
        let mut skip_cursor = io::Cursor::new(&skip_buf);
        let mut skip_acc = NbtAccounter::unlimited_heap();
        TYPE.skip(&mut skip_cursor, &mut skip_acc).unwrap();
        let mut tail = [0u8; 1];
        skip_cursor.read_exact(&mut tail).unwrap();
        assert_eq!(tail[0], 0xFF);
    }

    #[test]
    fn float_bits_preserved_as_longs() {
        let f64_nan = f64::NAN.to_bits() as i64;
        let f32_neg_inf = f32::NEG_INFINITY.to_bits() as i64;
        let neg_zero_f64 = (-0.0f64).to_bits() as i64;
        let tag = LongArrayTag::new(vec![f64_nan, f32_neg_inf, neg_zero_f64, -1]);
        let mut buf = Vec::new();
        tag.write(&mut buf).unwrap();
        let mut cursor = io::Cursor::new(&buf);
        let mut acc = NbtAccounter::unlimited_heap();
        let loaded = TYPE.load(&mut cursor, &mut acc).unwrap();
        assert_eq!(loaded.get(0).long_value(), f64_nan);
        assert_eq!(
            f64::from_bits(loaded.get(0).long_value() as u64).to_bits(),
            f64::NAN.to_bits()
        );
        assert_eq!(loaded.get(1).long_value(), f32_neg_inf);
        assert_eq!(loaded.get(2).long_value(), neg_zero_f64);
        assert_eq!(loaded.get(3).long_value(), -1);
    }

    #[test]
    fn to_string_and_copy_independence() {
        let tag = LongArrayTag::new(vec![1, -2, 3]);
        assert_eq!(tag.to_string_value(), "[L; 1L, -2L, 3L]");
        assert_eq!(tag.to_string(), "[L; 1L, -2L, 3L]");

        let mut copied = tag.copy();
        assert_eq!(copied, tag);
        assert!(copied.java_equals(&tag));
        copied.set_tag(0, &NumTag(99));
        assert_ne!(copied.get_as_long_array(), tag.get_as_long_array());
        assert_eq!(tag.get_as_long_array(), &[1, -2, 3]);
    }
}
