use std::fmt;
use std::hash::{Hash, Hasher};
use std::io::{self, Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::NbtError;

pub const SELF_SIZE_IN_BYTES: i32 = 24;

/// Minimal NBT accounter stub for self-contained I/O accounting.
#[derive(Debug, Clone)]
pub struct NbtAccounter {
    remaining: i64,
}

impl NbtAccounter {
    pub fn new(quota: i64) -> Self {
        Self { remaining: quota }
    }

    pub fn unlimited() -> Self {
        Self {
            remaining: i64::MAX,
        }
    }

    pub fn account_bytes(&mut self, bytes: i64) -> Result<(), NbtError> {
        self.remaining = self.remaining.saturating_sub(bytes);
        if self.remaining < 0 {
            return Err(NbtError::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                "NBT tag size limit exceeded",
            )));
        }
        Ok(())
    }

    pub fn account_bytes_mul(&mut self, per_element: i64, count: i32) -> Result<(), NbtError> {
        if count < 0 {
            return Err(NbtError::NegativeLength(count));
        }
        let bytes = per_element.saturating_mul(i64::from(count));
        self.account_bytes(bytes)
    }
}

/// Minimal tag trait to support `setTag`/`addTag` numeric checks.
pub trait Tag {
    fn as_numeric(&self) -> Option<&dyn NumericTag>;
}

/// Minimal numeric tag trait to support `NumericTag#intValue`.
pub trait NumericTag: Tag {
    fn int_value(&self) -> i32;
}

/// Minimal `IntTag` helper for `get`/`remove` return values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntTag {
    value: i32,
}

impl IntTag {
    // JAVA: net.minecraft.nbt.IntTag#valueOf
    pub fn value_of(value: i32) -> Self {
        Self { value }
    }

    pub fn value(&self) -> i32 {
        self.value
    }
}

impl Tag for IntTag {
    fn as_numeric(&self) -> Option<&dyn NumericTag> {
        Some(self)
    }
}

impl NumericTag for IntTag {
    fn int_value(&self) -> i32 {
        self.value
    }
}

/// Minimal tag visitor for `accept(TagVisitor)`.
pub trait TagVisitor {
    fn visit_int_array(&mut self, tag: &IntArrayTag);
}

/// Minimal stream tag visitor value result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamTagVisitorValueResult {
    Continue,
    Break,
    Halt,
}

/// Minimal stream tag visitor for `parse`/`accept(StreamTagVisitor)`.
pub trait StreamTagVisitor {
    fn visit(&mut self, data: &[i32]) -> StreamTagVisitorValueResult;
}

/// Type marker/handler for `IntArrayTag` TYPE operations.
#[derive(Debug, Clone, Copy, Default)]
pub struct IntArrayTagType;

pub const TYPE: IntArrayTagType = IntArrayTagType;

// JAVA: net.minecraft.nbt.IntArrayTag#readAccounted
fn read_accounted<R: Read>(
    reader: &mut R,
    accounter: &mut NbtAccounter,
) -> Result<Vec<i32>, NbtError> {
    accounter.account_bytes(24)?;
    let len = reader.read_i32::<BigEndian>()?;
    accounter.account_bytes_mul(4, len)?;
    let mut data = Vec::with_capacity(len as usize);
    for _ in 0..len {
        data.push(reader.read_i32::<BigEndian>()?);
    }
    Ok(data)
}

fn skip_bytes<R: Read>(reader: &mut R, n: i32) -> Result<(), NbtError> {
    if n <= 0 {
        return Ok(());
    }
    let mut remaining = n as usize;
    let mut buf = [0u8; 4096];
    while remaining > 0 {
        let to_read = remaining.min(buf.len());
        reader.read_exact(&mut buf[..to_read])?;
        remaining -= to_read;
    }
    Ok(())
}

impl IntArrayTagType {
    // JAVA: net.minecraft.nbt.IntArrayTag#load
    pub fn load<R: Read>(
        &self,
        reader: &mut R,
        accounter: &mut NbtAccounter,
    ) -> Result<IntArrayTag, NbtError> {
        Ok(IntArrayTag {
            data: read_accounted(reader, accounter)?,
        })
    }

    // JAVA: net.minecraft.nbt.IntArrayTag#parse
    pub fn parse<R: Read, V: StreamTagVisitor>(
        &self,
        reader: &mut R,
        visitor: &mut V,
        accounter: &mut NbtAccounter,
    ) -> Result<StreamTagVisitorValueResult, NbtError> {
        let data = read_accounted(reader, accounter)?;
        Ok(visitor.visit(&data))
    }

    // JAVA: net.minecraft.nbt.IntArrayTag#skip
    pub fn skip<R: Read>(
        &self,
        reader: &mut R,
        _accounter: &mut NbtAccounter,
    ) -> Result<(), NbtError> {
        let len = reader.read_i32::<BigEndian>()?;
        let bytes = len.wrapping_mul(4);
        skip_bytes(reader, bytes)
    }

    // JAVA: net.minecraft.nbt.IntArrayTag#getName
    pub fn get_name(&self) -> &'static str {
        "INT[]"
    }

    // JAVA: net.minecraft.nbt.IntArrayTag#getPrettyName
    pub fn get_pretty_name(&self) -> &'static str {
        "TAG_Int_Array"
    }
}

#[derive(Clone, Debug)]
pub struct IntArrayTag {
    data: Vec<i32>,
}

impl IntArrayTag {
    // JAVA: net.minecraft.nbt.IntArrayTag#<init>
    pub fn new(data: Vec<i32>) -> Self {
        Self { data }
    }

    // JAVA: net.minecraft.nbt.IntArrayTag#write
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), NbtError> {
        writer.write_i32::<BigEndian>(self.data.len() as i32)?;
        for &v in &self.data {
            writer.write_i32::<BigEndian>(v)?;
        }
        Ok(())
    }

    // JAVA: net.minecraft.nbt.IntArrayTag#sizeInBytes
    pub fn size_in_bytes(&self) -> i32 {
        24i32.wrapping_add(4i32.wrapping_mul(self.data.len() as i32))
    }

    // JAVA: net.minecraft.nbt.IntArrayTag#getId
    pub fn get_id(&self) -> u8 {
        11
    }

    // JAVA: net.minecraft.nbt.IntArrayTag#getType
    pub fn get_type(&self) -> IntArrayTagType {
        TYPE
    }

    // JAVA: net.minecraft.nbt.IntArrayTag#copy
    pub fn copy(&self) -> IntArrayTag {
        let mut arr = vec![0i32; self.data.len()];
        arr.copy_from_slice(&self.data);
        IntArrayTag { data: arr }
    }

    // JAVA: net.minecraft.nbt.IntArrayTag#hashCode
    pub fn hash_code(&self) -> i32 {
        let mut result: i32 = 1;
        for &element in &self.data {
            result = 31i32.wrapping_mul(result).wrapping_add(element);
        }
        result
    }

    // JAVA: net.minecraft.nbt.IntArrayTag#getAsIntArray
    pub fn get_as_int_array(&self) -> &[i32] {
        &self.data
    }

    // JAVA: net.minecraft.nbt.IntArrayTag#accept
    pub fn accept_tag_visitor<V: TagVisitor>(&self, visitor: &mut V) {
        visitor.visit_int_array(self);
    }

    // JAVA: net.minecraft.nbt.IntArrayTag#size
    pub fn size(&self) -> i32 {
        self.data.len() as i32
    }

    // JAVA: net.minecraft.nbt.IntArrayTag#get
    pub fn get(&self, index: i32) -> IntTag {
        IntTag::value_of(self.data[index as usize])
    }

    // JAVA: net.minecraft.nbt.IntArrayTag#setTag
    pub fn set_tag(&mut self, index: i32, tag: &dyn Tag) -> bool {
        if let Some(n) = tag.as_numeric() {
            self.data[index as usize] = n.int_value();
            true
        } else {
            false
        }
    }

    // JAVA: net.minecraft.nbt.IntArrayTag#addTag
    pub fn add_tag(&mut self, index: i32, tag: &dyn Tag) -> bool {
        if let Some(n) = tag.as_numeric() {
            self.data.insert(index as usize, n.int_value());
            true
        } else {
            false
        }
    }

    // JAVA: net.minecraft.nbt.IntArrayTag#remove
    pub fn remove(&mut self, index: i32) -> IntTag {
        let value = self.data.remove(index as usize);
        IntTag::value_of(value)
    }

    // JAVA: net.minecraft.nbt.IntArrayTag#clear
    pub fn clear(&mut self) {
        self.data = Vec::new();
    }

    // JAVA: net.minecraft.nbt.IntArrayTag#asIntArray
    pub fn as_int_array(&self) -> Option<&[i32]> {
        Some(&self.data)
    }

    // JAVA: net.minecraft.nbt.IntArrayTag#accept
    pub fn accept_stream_visitor<V: StreamTagVisitor>(
        &self,
        visitor: &mut V,
    ) -> StreamTagVisitorValueResult {
        visitor.visit(&self.data)
    }
}

impl fmt::Display for IntArrayTag {
    // JAVA: net.minecraft.nbt.IntArrayTag#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[I;")?;
        for (i, v) in self.data.iter().enumerate() {
            if i != 0 {
                f.write_str(",")?;
            }
            write!(f, " {}", v)?;
        }
        f.write_str("]")
    }
}

impl PartialEq for IntArrayTag {
    // JAVA: net.minecraft.nbt.IntArrayTag#equals
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for IntArrayTag {}

impl Hash for IntArrayTag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i32(self.hash_code());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[derive(Clone, Copy, Debug)]
    struct SimpleNumeric(i32);
    impl Tag for SimpleNumeric {
        fn as_numeric(&self) -> Option<&dyn NumericTag> {
            Some(self)
        }
    }
    impl NumericTag for SimpleNumeric {
        fn int_value(&self) -> i32 {
            self.0
        }
    }

    #[derive(Clone, Copy, Debug)]
    struct NonNumeric;
    impl Tag for NonNumeric {
        fn as_numeric(&self) -> Option<&dyn NumericTag> {
            None
        }
    }

    struct CollectVisitor(Vec<i32>);
    impl TagVisitor for CollectVisitor {
        fn visit_int_array(&mut self, tag: &IntArrayTag) {
            self.0 = tag.get_as_int_array().to_vec();
        }
    }

    struct StreamCollectVisitor(Vec<i32>);
    impl StreamTagVisitor for StreamCollectVisitor {
        fn visit(&mut self, data: &[i32]) -> StreamTagVisitorValueResult {
            self.0 = data.to_vec();
            StreamTagVisitorValueResult::Continue
        }
    }

    #[test]
    fn test_write_load_roundtrip_and_type_meta() {
        let tag = IntArrayTag::new(vec![1, -2, i32::MAX, i32::MIN]);
        let mut buf = Vec::new();
        tag.write(&mut buf).unwrap();
        let mut cursor = Cursor::new(&buf);
        let mut acc = NbtAccounter::unlimited();
        let loaded = TYPE.load(&mut cursor, &mut acc).unwrap();
        assert_eq!(loaded, tag);
        assert_eq!(loaded.get_id(), 11);
        assert_eq!(loaded.size(), 4);
        assert_eq!(TYPE.get_name(), "INT[]");
        assert_eq!(TYPE.get_pretty_name(), "TAG_Int_Array");
        assert_eq!(loaded.to_string(), "[I; 1, -2, 2147483647, -2147483648]");
    }

    #[test]
    fn test_hash_code_overflow_negative_and_equality() {
        let tag = IntArrayTag::new(vec![i32::MIN, -1, 0, i32::MAX]);
        let mut expected: i32 = 1;
        for &e in &[i32::MIN, -1i32, 0, i32::MAX] {
            expected = 31i32.wrapping_mul(expected).wrapping_add(e);
        }
        assert_eq!(tag.hash_code(), expected);

        let copy = tag.copy();
        assert_eq!(tag, copy);
        assert_ne!(
            IntArrayTag::new(vec![1, 2]),
            IntArrayTag::new(vec![2, 1])
        );
        assert_eq!(tag.as_int_array(), Some(tag.get_as_int_array()));
    }

    #[test]
    fn test_set_add_remove_clear_and_numeric_checks() {
        let mut tag = IntArrayTag::new(vec![10, 20, 30]);
        assert!(tag.set_tag(1, &SimpleNumeric(-5)));
        assert!(!tag.set_tag(0, &NonNumeric));
        assert_eq!(tag.get(1).value(), -5);

        assert!(tag.add_tag(1, &SimpleNumeric(99)));
        assert!(!tag.add_tag(0, &NonNumeric));
        assert_eq!(tag.get_as_int_array(), &[10, 99, -5, 30]);

        let removed = tag.remove(2);
        assert_eq!(removed.value(), -5);
        assert_eq!(tag.get_as_int_array(), &[10, 99, 30]);

        tag.clear();
        assert_eq!(tag.size(), 0);
        assert_eq!(tag.get_as_int_array(), &[]);
    }

    #[test]
    fn test_skip_negative_length_and_float_bits_preserved() {
        let bits = vec![
            f32::NAN.to_bits() as i32,
            f32::INFINITY.to_bits() as i32,
            f32::NEG_INFINITY.to_bits() as i32,
            (-0.0f32).to_bits() as i32,
        ];
        let tag = IntArrayTag::new(bits.clone());
        let mut buf = Vec::new();
        tag.write(&mut buf).unwrap();

        let mut skip_cursor = Cursor::new(&buf);
        let mut acc = NbtAccounter::unlimited();
        TYPE.skip(&mut skip_cursor, &mut acc).unwrap();
        assert_eq!(skip_cursor.position() as usize, buf.len());

        let mut load_cursor = Cursor::new(&buf);
        let mut acc2 = NbtAccounter::unlimited();
        let loaded = TYPE.load(&mut load_cursor, &mut acc2).unwrap();
        assert_eq!(loaded.get_as_int_array(), bits.as_slice());

        let mut neg_buf = Vec::new();
        neg_buf.write_i32::<BigEndian>(-1).unwrap();
        let mut neg_cursor = Cursor::new(&neg_buf);
        let mut acc3 = NbtAccounter::unlimited();
        assert!(TYPE.load(&mut neg_cursor, &mut acc3).is_err());
    }

    #[test]
    fn test_size_in_bytes_and_visitors() {
        let tag = IntArrayTag::new(vec![1, 2, 3]);
        assert_eq!(tag.size_in_bytes(), 24i32.wrapping_add(4i32.wrapping_mul(3)));

        let mut tv = CollectVisitor(Vec::new());
        tag.accept_tag_visitor(&mut tv);
        assert_eq!(tv.0, vec![1, 2, 3]);

        let mut sv = StreamCollectVisitor(Vec::new());
        let res = tag.accept_stream_visitor(&mut sv);
        assert_eq!(res, StreamTagVisitorValueResult::Continue);
        assert_eq!(sv.0, vec![1, 2, 3]);

        let mut parse_cursor = Cursor::new({
            let mut b = Vec::new();
            tag.write(&mut b).unwrap();
            b
        });
        let mut acc = NbtAccounter::unlimited();
        let mut sv2 = StreamCollectVisitor(Vec::new());
        let parse_res = TYPE.parse(&mut parse_cursor, &mut sv2, &mut acc).unwrap();
        assert_eq!(parse_res, StreamTagVisitorValueResult::Continue);
        assert_eq!(sv2.0, vec![1, 2, 3]);
    }
}
