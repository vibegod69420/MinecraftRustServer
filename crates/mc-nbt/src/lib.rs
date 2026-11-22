#![forbid(unsafe_code)]
//! Phase A faithful port of Minecraft 1.21.11 NBT binary I/O (`NbtIo`), tags 0..=12,
//! Java Modified UTF-8 (`DataInput`/`DataOutput`), and canonical structural comparison/hashing.

use std::collections::BTreeMap;
use std::io::{Cursor, Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use flate2::Compression;
use flate2::read::{GzDecoder, ZlibDecoder};
use flate2::write::{GzEncoder, ZlibEncoder};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NbtError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unknown NBT tag id: {0}")]
    UnknownTag(u8),
    #[error("Root tag must be Compound (10), got {0}")]
    InvalidRoot(u8),
    #[error("Negative list/array length: {0}")]
    NegativeLength(i32),
    #[error("Missing type on non-empty ListTag")]
    MissingListType,
    #[error("String byte length exceeds u16::MAX: {0}")]
    StringTooLong(usize),
    #[error("Malformed Java Modified UTF-8")]
    InvalidMutf8,
    #[error("Nesting depth exceeded {0}")]
    DepthExceeded(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TagId {
    End = 0,
    Byte = 1,
    Short = 2,
    Int = 3,
    Long = 4,
    Float = 5,
    Double = 6,
    ByteArray = 7,
    String = 8,
    List = 9,
    Compound = 10,
    IntArray = 11,
    LongArray = 12,
}

impl TagId {
    pub fn from_u8(v: u8) -> Result<Self, NbtError> {
        match v {
            0 => Ok(Self::End),
            1 => Ok(Self::Byte),
            2 => Ok(Self::Short),
            3 => Ok(Self::Int),
            4 => Ok(Self::Long),
            5 => Ok(Self::Float),
            6 => Ok(Self::Double),
            7 => Ok(Self::ByteArray),
            8 => Ok(Self::String),
            9 => Ok(Self::List),
            10 => Ok(Self::Compound),
            11 => Ok(Self::IntArray),
            12 => Ok(Self::LongArray),
            _ => Err(NbtError::UnknownTag(v)),
        }
    }
}

/// Canonical NBT value representation preserving insertion order in `Compound` via `Vec<(String, NbtTag)>`
/// while supporting order-independent structural canonicalization (`canonical_hash`).
/// Floats/doubles use exact `f32`/`f64` values, and equality checks compare float bits via `canonical_eq`.
#[derive(Debug, Clone, PartialEq)]
pub enum NbtTag {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List { elem_type: u8, items: Vec<NbtTag> },
    Compound(Vec<(String, NbtTag)>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl NbtTag {
    // JAVA: net.minecraft.nbt.FloatTag#valueOf
    pub fn float_value_of(v: f32) -> Self {
        if v == 0.0 { Self::Float(0.0) } else { Self::Float(v) }
    }

    // JAVA: net.minecraft.nbt.DoubleTag#valueOf
    pub fn double_value_of(v: f64) -> Self {
        if v == 0.0 { Self::Double(0.0) } else { Self::Double(v) }
    }

    pub fn id(&self) -> u8 {
        match self {
            Self::Byte(_) => 1,
            Self::Short(_) => 2,
            Self::Int(_) => 3,
            Self::Long(_) => 4,
            Self::Float(_) => 5,
            Self::Double(_) => 6,
            Self::ByteArray(_) => 7,
            Self::String(_) => 8,
            Self::List { .. } => 9,
            Self::Compound(_) => 10,
            Self::IntArray(_) => 11,
            Self::LongArray(_) => 12,
        }
    }

    pub fn compound_get(&self, key: &str) -> Option<&NbtTag> {
        match self {
            Self::Compound(entries) => entries.iter().rev().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// Structural canonical equality (order-independent for Compound keys, bitwise exact for floats/NaNs/-0.0).
    pub fn structural_eq(&self, other: &Self) -> bool {
        self.canonical_bytes() == other.canonical_bytes()
    }

    /// Deterministic canonical serialization independent of Compound entry ordering.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.write_canonical(&mut out);
        out
    }

    /// SHA-256 of canonical structural serialization (Method 3 & Method 5).
    pub fn canonical_sha256(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.canonical_bytes());
        hasher.finalize().into()
    }

    fn write_canonical(&self, out: &mut Vec<u8>) {
        out.push(self.id());
        match self {
            Self::Byte(v) => out.push(*v as u8),
            Self::Short(v) => out.extend_from_slice(&v.to_be_bytes()),
            Self::Int(v) => out.extend_from_slice(&v.to_be_bytes()),
            Self::Long(v) => out.extend_from_slice(&v.to_be_bytes()),
            Self::Float(v) => out.extend_from_slice(&v.to_bits().to_be_bytes()),
            Self::Double(v) => out.extend_from_slice(&v.to_bits().to_be_bytes()),
            Self::ByteArray(arr) => {
                out.extend_from_slice(&(arr.len() as i32).to_be_bytes());
                for &b in arr {
                    out.push(b as u8);
                }
            }
            Self::String(s) => {
                let bytes = s.as_bytes();
                out.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
                out.extend_from_slice(bytes);
            }
            Self::List { elem_type, items } => {
                out.push(*elem_type);
                out.extend_from_slice(&(items.len() as i32).to_be_bytes());
                for item in items {
                    item.write_canonical(out);
                }
            }
            Self::Compound(entries) => {
                let mut map = BTreeMap::new();
                for (k, v) in entries {
                    map.insert(k.as_str(), v);
                }
                out.extend_from_slice(&(map.len() as u32).to_be_bytes());
                for (k, v) in map {
                    let kb = k.as_bytes();
                    out.extend_from_slice(&(kb.len() as u32).to_be_bytes());
                    out.extend_from_slice(kb);
                    v.write_canonical(out);
                }
            }
            Self::IntArray(arr) => {
                out.extend_from_slice(&(arr.len() as i32).to_be_bytes());
                for &x in arr {
                    out.extend_from_slice(&x.to_be_bytes());
                }
            }
            Self::LongArray(arr) => {
                out.extend_from_slice(&(arr.len() as i32).to_be_bytes());
                for &x in arr {
                    out.extend_from_slice(&x.to_be_bytes());
                }
            }
        }
    }
}

/// Encode UTF-8 Rust `&str` to Java Modified UTF-8 (DataOutput.writeUTF).
pub fn encode_mutf8(s: &str) -> Result<Vec<u8>, NbtError> {
    let mut out = Vec::with_capacity(s.len());
    for unit in s.encode_utf16() {
        if unit >= 0x0001 && unit <= 0x007F {
            out.push(unit as u8);
        } else if unit > 0x07FF {
            out.push(0xE0 | ((unit >> 12) as u8 & 0x0F));
            out.push(0x80 | ((unit >> 6) as u8 & 0x3F));
            out.push(0x80 | (unit as u8 & 0x3F));
        } else {
            out.push(0xC0 | ((unit >> 6) as u8 & 0x1F));
            out.push(0x80 | (unit as u8 & 0x3F));
        }
    }
    if out.len() > u16::MAX as usize {
        return Err(NbtError::StringTooLong(out.len()));
    }
    Ok(out)
}

/// Decode Java Modified UTF-8 bytes (DataInput.readUTF).
pub fn decode_mutf8(bytes: &[u8]) -> Result<String, NbtError> {
    let mut units = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        match c >> 4 {
            0..=7 => {
                i += 1;
                units.push(c as u16);
            }
            12 | 13 => {
                if i + 1 >= bytes.len() {
                    return Err(NbtError::InvalidMutf8);
                }
                let c2 = bytes[i + 1];
                if (c2 & 0xC0) != 0x80 {
                    return Err(NbtError::InvalidMutf8);
                }
                i += 2;
                units.push((((c & 0x1F) as u16) << 6) | ((c2 & 0x3F) as u16));
            }
            14 => {
                if i + 2 >= bytes.len() {
                    return Err(NbtError::InvalidMutf8);
                }
                let c2 = bytes[i + 1];
                let c3 = bytes[i + 2];
                if (c2 & 0xC0) != 0x80 || (c3 & 0xC0) != 0x80 {
                    return Err(NbtError::InvalidMutf8);
                }
                i += 3;
                units.push(
                    (((c & 0x0F) as u16) << 12)
                        | (((c2 & 0x3F) as u16) << 6)
                        | ((c3 & 0x3F) as u16),
                );
            }
            _ => return Err(NbtError::InvalidMutf8),
        }
    }
    String::from_utf16(&units).map_err(|_| NbtError::InvalidMutf8)
}

fn read_mutf8_string<R: Read>(r: &mut R) -> Result<String, NbtError> {
    let len = r.read_u16::<BigEndian>()? as usize;
    let mut buf = vec![0_u8; len];
    r.read_exact(&mut buf)?;
    decode_mutf8(&buf)
}

fn write_mutf8_string<W: Write>(w: &mut W, s: &str) -> Result<(), NbtError> {
    let encoded = encode_mutf8(s)?;
    w.write_u16::<BigEndian>(encoded.len() as u16)?;
    w.write_all(&encoded)?;
    Ok(())
}

// JAVA: net.minecraft.nbt.NbtIo#read(DataInput)
pub fn read_nbt<R: Read>(r: &mut R) -> Result<(String, NbtTag), NbtError> {
    let root_id = r.read_u8()?;
    if root_id != 10 {
        return Err(NbtError::InvalidRoot(root_id));
    }
    let name = read_mutf8_string(r)?;
    let payload = read_tag_payload(r, TagId::Compound, 0)?;
    Ok((name, payload))
}

// JAVA: net.minecraft.nbt.NbtIo#readCompressed(InputStream,NbtAccounter)
pub fn read_nbt_compressed(bytes: &[u8]) -> Result<(String, NbtTag), NbtError> {
    let mut dec = GzDecoder::new(Cursor::new(bytes));
    read_nbt(&mut dec)
}

// JAVA: net.minecraft.nbt.NbtIo#write(CompoundTag,DataOutput)
pub fn write_nbt<W: Write>(w: &mut W, name: &str, root: &NbtTag) -> Result<(), NbtError> {
    let id = root.id();
    if id != 10 {
        return Err(NbtError::InvalidRoot(id));
    }
    w.write_u8(10)?;
    write_mutf8_string(w, name)?;
    write_tag_payload(w, root)
}

// JAVA: net.minecraft.nbt.NbtIo#writeCompressed(CompoundTag,OutputStream)
pub fn write_nbt_compressed(name: &str, root: &NbtTag) -> Result<Vec<u8>, NbtError> {
    let mut enc = GzEncoder::new(Vec::new(), Compression::default());
    write_nbt(&mut enc, name, root)?;
    Ok(enc.finish()?)
}

/// Zlib-compressed NBT (used by Anvil chunk payloads with compression type 2).
pub fn read_nbt_zlib(bytes: &[u8]) -> Result<(String, NbtTag), NbtError> {
    let mut dec = ZlibDecoder::new(Cursor::new(bytes));
    read_nbt(&mut dec)
}

pub fn write_nbt_zlib(name: &str, root: &NbtTag) -> Result<Vec<u8>, NbtError> {
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
    write_nbt(&mut enc, name, root)?;
    Ok(enc.finish()?)
}

fn read_tag_payload<R: Read>(r: &mut R, id: TagId, depth: usize) -> Result<NbtTag, NbtError> {
    if depth > 512 {
        return Err(NbtError::DepthExceeded(depth));
    }
    Ok(match id {
        TagId::End => return Err(NbtError::UnknownTag(0)),
        TagId::Byte => NbtTag::Byte(r.read_i8()?),
        TagId::Short => NbtTag::Short(r.read_i16::<BigEndian>()?),
        TagId::Int => NbtTag::Int(r.read_i32::<BigEndian>()?),
        TagId::Long => NbtTag::Long(r.read_i64::<BigEndian>()?),
        TagId::Float => NbtTag::Float(f32::from_bits(r.read_u32::<BigEndian>()?)),
        TagId::Double => NbtTag::Double(f64::from_bits(r.read_u64::<BigEndian>()?)),
        TagId::ByteArray => {
            let len = r.read_i32::<BigEndian>()?;
            if len < 0 {
                return Err(NbtError::NegativeLength(len));
            }
            let mut buf = vec![0_u8; len as usize];
            r.read_exact(&mut buf)?;
            NbtTag::ByteArray(buf.into_iter().map(|b| b as i8).collect())
        }
        TagId::String => NbtTag::String(read_mutf8_string(r)?),
        TagId::List => {
            let elem_raw = r.read_u8()?;
            let len = r.read_i32::<BigEndian>()?;
            if len < 0 {
                return Err(NbtError::NegativeLength(len));
            }
            if elem_raw == 0 && len > 0 {
                return Err(NbtError::MissingListType);
            }
            let mut items = Vec::with_capacity(len as usize);
            if len > 0 {
                let elem_id = TagId::from_u8(elem_raw)?;
                for _ in 0..len {
                    items.push(read_tag_payload(r, elem_id, depth + 1)?);
                }
            }
            NbtTag::List {
                elem_type: elem_raw,
                items,
            }
        }
        TagId::Compound => {
            let mut entries = Vec::new();
            loop {
                let child_raw = r.read_u8()?;
                if child_raw == 0 {
                    break;
                }
                let child_id = TagId::from_u8(child_raw)?;
                let key = read_mutf8_string(r)?;
                let val = read_tag_payload(r, child_id, depth + 1)?;
                entries.push((key, val));
            }
            NbtTag::Compound(entries)
        }
        TagId::IntArray => {
            let len = r.read_i32::<BigEndian>()?;
            if len < 0 {
                return Err(NbtError::NegativeLength(len));
            }
            let mut arr = Vec::with_capacity(len as usize);
            for _ in 0..len {
                arr.push(r.read_i32::<BigEndian>()?);
            }
            NbtTag::IntArray(arr)
        }
        TagId::LongArray => {
            let len = r.read_i32::<BigEndian>()?;
            if len < 0 {
                return Err(NbtError::NegativeLength(len));
            }
            let mut arr = Vec::with_capacity(len as usize);
            for _ in 0..len {
                arr.push(r.read_i64::<BigEndian>()?);
            }
            NbtTag::LongArray(arr)
        }
    })
}

fn write_tag_payload<W: Write>(w: &mut W, tag: &NbtTag) -> Result<(), NbtError> {
    match tag {
        NbtTag::Byte(v) => w.write_i8(*v)?,
        NbtTag::Short(v) => w.write_i16::<BigEndian>(*v)?,
        NbtTag::Int(v) => w.write_i32::<BigEndian>(*v)?,
        NbtTag::Long(v) => w.write_i64::<BigEndian>(*v)?,
        NbtTag::Float(v) => w.write_u32::<BigEndian>(v.to_bits())?,
        NbtTag::Double(v) => w.write_u64::<BigEndian>(v.to_bits())?,
        NbtTag::ByteArray(arr) => {
            w.write_i32::<BigEndian>(arr.len() as i32)?;
            for &b in arr {
                w.write_i8(b)?;
            }
        }
        NbtTag::String(s) => write_mutf8_string(w, s)?,
        NbtTag::List { elem_type, items } => {
            let actual_elem = if items.is_empty() { *elem_type } else { items[0].id() };
            w.write_u8(actual_elem)?;
            w.write_i32::<BigEndian>(items.len() as i32)?;
            for item in items {
                write_tag_payload(w, item)?;
            }
        }
        NbtTag::Compound(entries) => {
            for (k, v) in entries {
                w.write_u8(v.id())?;
                write_mutf8_string(w, k)?;
                write_tag_payload(w, v)?;
            }
            w.write_u8(0)?;
        }
        NbtTag::IntArray(arr) => {
            w.write_i32::<BigEndian>(arr.len() as i32)?;
            for &x in arr {
                w.write_i32::<BigEndian>(x)?;
            }
        }
        NbtTag::LongArray(arr) => {
            w.write_i32::<BigEndian>(arr.len() as i32)?;
            for &x in arr {
                w.write_i64::<BigEndian>(x)?;
            }
        }
    }
    Ok(())
}
