use std::fmt;
use std::hash::{Hash, Hasher};
use std::io::{self, Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::NbtError;

pub const SELF_SIZE_IN_BYTES: i32 = 24;

/// Minimal accounting shim for NBT read operations.
#[derive(Debug, Clone, Copy)]
pub struct NbtAccounter {
    quota: u64,
    used: u64,
}

impl NbtAccounter {
    pub fn new(quota: u64) -> Self {
        Self { quota, used: 0 }
    }

    pub fn unlimited() -> Self {
        Self::new(u64::MAX)
    }

    pub fn account_bytes(&mut self, bytes: u64) -> Result<(), NbtError> {
        self.used = self.used.saturating_add(bytes);
        if self.used > self.quota {
            return Err(NbtError::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                "NBT accounter exceeded",
            )));
        }
        Ok(())
    }

    pub fn account_bytes_mul(&mut self, bytes_per: u64, count: u64) -> Result<(), NbtError> {
        self.account_bytes(bytes_per.saturating_mul(count))
    }
}

/// Minimal numeric tag interface needed by `ByteArrayTag#setTag`/`addTag`.
pub trait NumericTag {
    fn byte_value(&self) -> i8;
}

/// Minimal tag interface for `instanceof NumericTag` checks.
pub trait TagLike {
    fn as_numeric(&self) -> Option<&dyn NumericTag>;
}

/// Minimal `ByteTag` shim for `ByteArrayTag#get`/`remove`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteTag(pub i8);

impl ByteTag {
    pub fn value_of(value: i8) -> Self {
        Self(value)
    }
}

impl NumericTag for ByteTag {
    fn byte_value(&self) -> i8 {
        self.0
    }
}

impl TagLike for ByteTag {
    fn as_numeric(&self) -> Option<&dyn NumericTag> {
        Some(self)
    }
}

/// Simple non-numeric tag shim for tests and negative cases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonNumericTag;

impl TagLike for NonNumericTag {
    fn as_numeric(&self) -> Option<&dyn NumericTag> {
        None
    }
}

pub trait StreamTagVisitor {
    type ValueResult;
    fn visit(&mut self, data: &[i8]) -> Self::ValueResult;
}

pub trait TagVisitor {
    fn visit_byte_array(&mut self, tag: &ByteArrayTag);
}

/// Minimal string visitor shim used by `ByteArrayTag#toString`.
#[derive(Debug, Default, Clone)]
pub struct StringTagVisitor {
    out: String,
}

impl StringTagVisitor {
    pub fn new() -> Self {
        Self {
            out: String::new(),
        }
    }

    pub fn visit_byte_array(&mut self, tag: &ByteArrayTag) {
        self.out.push_str("[B;");
        for (i, &b) in tag.data.iter().enumerate() {
            if i == 0 {
                self.out.push(' ');
            } else {
                self.out.push_str(", ");
            }
            self.out.push_str(&b.to_string());
            self.out.push('B');
        }
        self.out.push(']');
    }

    pub fn build(self) -> String {
        self.out
    }
}

/// Port of `net.minecraft.nbt.ByteArrayTag$TYPE`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ByteArrayTagType;

pub const TYPE: ByteArrayTagType = ByteArrayTagType;

impl ByteArrayTagType {
    // JAVA: net.minecraft.nbt.ByteArrayTag#readAccounted
    fn read_accounted<R: Read>(
        reader: &mut R,
        accounter: &mut NbtAccounter,
    ) -> Result<Vec<i8>, NbtError> {
        accounter.account_bytes(24_u64)?;
        let len = reader.read_i32::<BigEndian>()?;
        if len < 0 {
            return Err(NbtError::NegativeLength(len));
        }
        accounter.account_bytes_mul(1_u64, len as u64)?;
        let mut buf = vec![0_u8; len as usize];
        reader.read_exact(&mut buf)?;
        Ok(buf.into_iter().map(|b| b as i8).collect())
    }

    // JAVA: net.minecraft.nbt.ByteArrayTag#load
    pub fn load<R: Read>(
        reader: &mut R,
        accounter: &mut NbtAccounter,
    ) -> Result<ByteArrayTag, NbtError> {
        Ok(ByteArrayTag::new(Self::read_accounted(reader, accounter)?))
    }

    // JAVA: net.minecraft.nbt.ByteArrayTag#parse
    pub fn parse<R: Read, V: StreamTagVisitor>(
        reader: &mut R,
        visitor: &mut V,
        accounter: &mut NbtAccounter,
    ) -> Result<V::ValueResult, NbtError> {
        let data = Self::read_accounted(reader, accounter)?;
        Ok(visitor.visit(&data))
    }

    // JAVA: net.minecraft.nbt.ByteArrayTag#skip
    pub fn skip<R: Read>(reader: &mut R, _accounter: &mut NbtAccounter) -> Result<(), NbtError> {
        let len = reader.read_i32::<BigEndian>()?;
        let to_skip = len.wrapping_mul(1);
        if to_skip < 0 {
            return Err(NbtError::NegativeLength(to_skip));
        }
        let mut remaining = to_skip as usize;
        let mut buf = [0_u8; 8192];
        while remaining > 0 {
            let n = remaining.min(buf.len());
            reader.read_exact(&mut buf[..n])?;
            remaining -= n;
        }
        Ok(())
    }

    // JAVA: net.minecraft.nbt.ByteArrayTag#getName
    pub fn get_name(&self) -> &'static str {
        "BYTE[]"
    }

    // JAVA: net.minecraft.nbt.ByteArrayTag#getPrettyName
    pub fn get_pretty_name(&self) -> &'static str {
        "TAG_Byte_Array"
    }
}

/// Port of `net.minecraft.nbt.ByteArrayTag`.
#[derive(Clone, Eq)]
pub struct ByteArrayTag {
    pub data: Vec<i8>,
}

impl ByteArrayTag {
    // JAVA: net.minecraft.nbt.ByteArrayTag#<init>
    pub fn new(data: Vec<i8>) -> Self {
        Self { data }
    }

    // JAVA: net.minecraft.nbt.ByteArrayTag#write
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), NbtError> {
        writer.write_i32::<BigEndian>(self.data.len() as i32)?;
        for &b in &self.data {
            writer.write_u8(b as u8)?;
        }
        Ok(())
    }

    // JAVA: net.minecraft.nbt.ByteArrayTag#sizeInBytes
    pub fn size_in_bytes(&self) -> i32 {
        24_i32.wrapping_add(1_i32.wrapping_mul(self.data.len() as i32))
    }

    // JAVA: net.minecraft.nbt.ByteArrayTag#getId
    pub fn get_id(&self) -> u8 {
        7
    }

    // JAVA: net.minecraft.nbt.ByteArrayTag#getType
    pub fn get_type(&self) -> ByteArrayTagType {
        TYPE
    }

    // JAVA: net.minecraft.nbt.ByteArrayTag#toString
    pub fn to_string_value(&self) -> String {
        let mut visitor = StringTagVisitor::new();
        visitor.visit_byte_array(self);
        visitor.build()
    }

    // JAVA: net.minecraft.nbt.ByteArrayTag#copy
    pub fn copy(&self) -> ByteArrayTag {
        let mut arr = vec![0_i8; self.data.len()];
        arr.copy_from_slice(&self.data);
        ByteArrayTag::new(arr)
    }

    // JAVA: net.minecraft.nbt.ByteArrayTag#hashCode
    pub fn hash_code(&self) -> i32 {
        let mut result: i32 = 1;
        for &b in &self.data {
            result = 31_i32.wrapping_mul(result).wrapping_add(b as i32);
        }
        result
    }

    // JAVA: net.minecraft.nbt.ByteArrayTag#accept
    pub fn accept_tag_visitor<V: TagVisitor>(&self, visitor: &mut V) {
        visitor.visit_byte_array(self);
    }

    // JAVA: net.minecraft.nbt.ByteArrayTag#getAsByteArray
    pub fn get_as_byte_array(&self) -> &[i8] {
        &self.data
    }

    // JAVA: net.minecraft.nbt.ByteArrayTag#size
    pub fn size(&self) -> i32 {
        self.data.len() as i32
    }

    // JAVA: net.minecraft.nbt.ByteArrayTag#get
    pub fn get(&self, index: i32) -> ByteTag {
        ByteTag::value_of(self.data[index as usize])
    }

    // JAVA: net.minecraft.nbt.ByteArrayTag#setTag
    pub fn set_tag(&mut self, index: i32, tag: &dyn TagLike) -> bool {
        if let Some(numeric) = tag.as_numeric() {
            self.data[index as usize] = numeric.byte_value();
            true
        } else {
            false
        }
    }

    // JAVA: net.minecraft.nbt.ByteArrayTag#addTag
    pub fn add_tag(&mut self, index: i32, tag: &dyn TagLike) -> bool {
        if let Some(numeric) = tag.as_numeric() {
            self.data.insert(index as usize, numeric.byte_value());
            true
        } else {
            false
        }
    }

    // JAVA: net.minecraft.nbt.ByteArrayTag#remove
    pub fn remove(&mut self, index: i32) -> ByteTag {
        let value = self.data.remove(index as usize);
        ByteTag::value_of(value)
    }

    // JAVA: net.minecraft.nbt.ByteArrayTag#clear
    pub fn clear(&mut self) {
        self.data = Vec::new();
    }

    // JAVA: net.minecraft.nbt.ByteArrayTag#asByteArray
    pub fn as_byte_array(&self) -> Option<&[i8]> {
        Some(&self.data)
    }

    // JAVA: net.minecraft.nbt.ByteArrayTag#accept
    pub fn accept_stream_tag_visitor<V: StreamTagVisitor>(&self, visitor: &mut V) -> V::ValueResult {
        visitor.visit(&self.data)
    }
}

impl PartialEq for ByteArrayTag {
    // JAVA: net.minecraft.nbt.ByteArrayTag#equals
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Hash for ByteArrayTag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i32(self.hash_code());
    }
}

impl fmt::Debug for ByteArrayTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ByteArrayTag")
            .field("data", &self.data)
            .finish()
    }
}

impl fmt::Display for ByteArrayTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_string_value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[derive(Default)]
    struct RecordingTagVisitor {
        seen: Option<Vec<i8>>,
    }

    impl TagVisitor for RecordingTagVisitor {
        fn visit_byte_array(&mut self, tag: &ByteArrayTag) {
            self.seen = Some(tag.get_as_byte_array().to_vec());
        }
    }

    struct CollectStreamVisitor;

    impl StreamTagVisitor for CollectStreamVisitor {
        type ValueResult = Vec<i8>;
        fn visit(&mut self, data: &[i8]) -> Self::ValueResult {
            data.to_vec()
        }
    }

    #[test]
    fn write_load_roundtrip_preserves_negative_bytes() {
        let tag = ByteArrayTag::new(vec![0, 1, -1, 127, -128]);
        let mut buf = Vec::new();
        tag.write(&mut buf).unwrap();

        let mut cursor = Cursor::new(buf);
        let mut acc = NbtAccounter::unlimited();
        let loaded = ByteArrayTagType::load(&mut cursor, &mut acc).unwrap();

        assert_eq!(loaded.get_as_byte_array(), &[0, 1, -1, 127, -128]);
        assert_eq!(loaded.get_id(), 7);
        assert_eq!(loaded.size(), 5);
    }

    #[test]
    fn size_in_bytes_matches_java_formula() {
        let empty = ByteArrayTag::new(Vec::new());
        let tag = ByteArrayTag::new(vec![1, 2, 3]);
        assert_eq!(empty.size_in_bytes(), 24);
        assert_eq!(tag.size_in_bytes(), 27);

        // Java int arithmetic wraps; verify the same wrapping formula.
        let len = i32::MAX;
        let wrapped = 24_i32.wrapping_add(1_i32.wrapping_mul(len));
        assert_eq!(wrapped, 24_i32.wrapping_add(i32::MAX));
    }

    #[test]
    fn mutation_methods_match_java_semantics() {
        let mut tag = ByteArrayTag::new(vec![10, 20, 30]);
        assert!(tag.set_tag(1, &ByteTag::value_of(-5)));
        assert_eq!(tag.get(1), ByteTag::value_of(-5));

        assert!(!tag.set_tag(0, &NonNumericTag));
        assert_eq!(tag.get_as_byte_array(), &[10, -5, 30]);

        assert!(tag.add_tag(1, &ByteTag::value_of(99)));
        assert!(!tag.add_tag(1, &NonNumericTag));
        assert_eq!(tag.get_as_byte_array(), &[10, 99, -5, 30]);

        let removed = tag.remove(2);
        assert_eq!(removed, ByteTag::value_of(-5));
        assert_eq!(tag.get_as_byte_array(), &[10, 99, 30]);

        tag.clear();
        assert_eq!(tag.size(), 0);
        assert_eq!(tag.get_as_byte_array(), &[] as &[i8]);
    }

    #[test]
    fn equals_hash_copy_and_to_string() {
        let a = ByteArrayTag::new(vec![1, -2, 3]);
        let b = ByteArrayTag::new(vec![1, -2, 3]);
        let c = ByteArrayTag::new(vec![3, 2, 1]);

        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.hash_code(), b.hash_code());

        // Arrays.hashCode(byte[]) semantics: result = 31 * result + element
        let mut expected: i32 = 1;
        for &v in &[1_i8, -2, 3] {
            expected = 31_i32.wrapping_mul(expected).wrapping_add(v as i32);
        }
        assert_eq!(a.hash_code(), expected);

        let mut copied = a.copy();
        copied.set_tag(0, &ByteTag::value_of(9));
        assert_eq!(a.get(0), ByteTag::value_of(1));
        assert_eq!(copied.get(0), ByteTag::value_of(9));

        assert_eq!(a.to_string_value(), "[B; 1B, -2B, 3B]");
        assert_eq!(a.to_string(), "[B; 1B, -2B, 3B]");
    }

    #[test]
    fn negative_length_and_parse_skip() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(-1_i32).to_be_bytes());
        let mut cursor = Cursor::new(bytes);
        let mut acc = NbtAccounter::unlimited();
        let err = ByteArrayTagType::load(&mut cursor, &mut acc).unwrap_err();
        assert!(matches!(err, NbtError::NegativeLength(-1)));

        let tag = ByteArrayTag::new(vec![7, -8]);
        let mut buf = Vec::new();
        tag.write(&mut buf).unwrap();

        let mut parse_cursor = Cursor::new(buf.clone());
        let mut parse_acc = NbtAccounter::unlimited();
        let mut stream_visitor = CollectStreamVisitor;
        let parsed = ByteArrayTagType::parse(&mut parse_cursor, &mut stream_visitor, &mut parse_acc)
            .unwrap();
        assert_eq!(parsed, vec![7, -8]);

        let mut skip_cursor = Cursor::new(buf);
        let mut skip_acc = NbtAccounter::unlimited();
        ByteArrayTagType::skip(&mut skip_cursor, &mut skip_acc).unwrap();
        assert_eq!(skip_cursor.position() as usize, 4 + 2);
    }

    #[test]
    fn visitors_accessors_and_type_metadata() {
        let tag = ByteArrayTag::new(vec![42, -1]);
        assert_eq!(tag.get_id(), 7);
        assert_eq!(tag.size(), 2);
        assert_eq!(tag.get_as_byte_array(), &[42, -1]);
        assert_eq!(tag.as_byte_array(), Some(&[42_i8, -1][..]));

        let mut tag_visitor = RecordingTagVisitor::default();
        tag.accept_tag_visitor(&mut tag_visitor);
        assert_eq!(tag_visitor.seen.as_deref(), Some(&[42_i8, -1][..]));

        let mut stream_visitor = CollectStreamVisitor;
        let out = tag.accept_stream_tag_visitor(&mut stream_visitor);
        assert_eq!(out, vec![42, -1]);

        let ty = tag.get_type();
        assert_eq!(ty.get_name(), "BYTE[]");
        assert_eq!(ty.get_pretty_name(), "TAG_Byte_Array");
    }
}
