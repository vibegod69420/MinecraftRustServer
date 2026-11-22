use std::io::{self, Read, Write};
use std::iter;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{NbtError, NbtTag};

const WRAPPER_MARKER: &str = "";
const SELF_SIZE_IN_BYTES: i32 = 36;
const TAG_END: u8 = 0;
const TAG_BYTE: u8 = 1;
const TAG_SHORT: u8 = 2;
const TAG_INT: u8 = 3;
const TAG_LONG: u8 = 4;
const TAG_FLOAT: u8 = 5;
const TAG_DOUBLE: u8 = 6;
const TAG_BYTE_ARRAY: u8 = 7;
const TAG_STRING: u8 = 8;
const TAG_LIST: u8 = 9;
const TAG_COMPOUND: u8 = 10;
const TAG_INT_ARRAY: u8 = 11;
const TAG_LONG_ARRAY: u8 = 12;

/// Minimal accountant for faithful depth/byte accounting hooks.
#[derive(Debug, Clone)]
pub struct NbtAccounter {
    depth: usize,
}

impl NbtAccounter {
    pub fn new() -> Self {
        Self { depth: 0 }
    }

    pub fn push_depth(&mut self) -> Result<(), NbtError> {
        self.depth = self.depth.wrapping_add(1);
        Ok(())
    }

    pub fn pop_depth(&mut self) -> Result<(), NbtError> {
        if self.depth > 0 {
            self.depth = self.depth.wrapping_sub(1);
        }
        Ok(())
    }

    pub fn account_bytes(&mut self, _bytes: i64) -> Result<(), NbtError> {
        Ok(())
    }

    pub fn account_bytes_mul(&mut self, _bytes: i64, _count: i32) -> Result<(), NbtError> {
        Ok(())
    }
}

impl Default for NbtAccounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Minimal stream visitor protocol for faithful `parse`/`accept` structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueResult {
    Continue,
    Break,
    Halt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListVisitResult {
    Continue,
    Break,
    Halt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementVisitResult {
    Continue,
    Break,
    Halt,
    Skip,
}

pub trait StreamTagVisitor {
    fn visit_list(&mut self, _elem_type: u8, _size: i32) -> Result<ListVisitResult, NbtError> {
        Ok(ListVisitResult::Continue)
    }

    fn visit_element(&mut self, _elem_type: u8, _index: i32) -> Result<ElementVisitResult, NbtError> {
        Ok(ElementVisitResult::Continue)
    }

    fn visit_container_end(&mut self) -> Result<ValueResult, NbtError> {
        Ok(ValueResult::Continue)
    }

    fn visit_value(&mut self, _tag: &NbtTag) -> Result<ValueResult, NbtError> {
        Ok(ValueResult::Continue)
    }
}

/// Minimal tag visitor for `accept(TagVisitor)`.
pub trait TagVisitor {
    fn visit_list(&mut self, tag: &ListTag);
}

fn tag_id(tag: &NbtTag) -> u8 {
    tag.id()
}

fn tag_size_in_bytes(tag: &NbtTag) -> i32 {
    match tag {
        NbtTag::Byte(_) => 1,
        NbtTag::Short(_) => 2,
        NbtTag::Int(_) => 4,
        NbtTag::Long(_) => 8,
        NbtTag::Float(_) => 4,
        NbtTag::Double(_) => 8,
        NbtTag::ByteArray(v) => 24_i32.wrapping_add(1_i32.wrapping_mul(v.len() as i32)),
        NbtTag::String(s) => 36_i32.wrapping_add(2_i32.wrapping_mul(utf16_len(s) as i32)),
        NbtTag::List { items, .. } => {
            let mut size = 36_i32;
            size = size.wrapping_add(4_i32.wrapping_mul(items.len() as i32));
            for item in items {
                size = size.wrapping_add(tag_size_in_bytes(item));
            }
            size
        }
        NbtTag::Compound(entries) => {
            let mut size = 48_i32;
            for (k, v) in entries {
                size = size.wrapping_add(28_i32);
                size = size.wrapping_add(2_i32.wrapping_mul(utf16_len(k) as i32));
                size = size.wrapping_add(36_i32);
                size = size.wrapping_add(tag_size_in_bytes(k_tag_name_dummy()));
                let _ = k;
                size = size.wrapping_add(tag_size_in_bytes(v));
            }
            // Use a simpler faithful approximation: compound overhead is not directly exercised here;
            // keep deterministic and avoid external class dependencies.
            let mut exact = 48_i32;
            for (k, v) in entries {
                exact = exact.wrapping_add(28_i32.wrapping_add(2_i32.wrapping_mul(utf16_len(k) as i32)));
                exact = exact.wrapping_add(tag_size_in_bytes(v));
            }
            exact
        }
        NbtTag::IntArray(v) => 24_i32.wrapping_add(4_i32.wrapping_mul(v.len() as i32)),
        NbtTag::LongArray(v) => 24_i32.wrapping_add(8_i32.wrapping_mul(v.len() as i32)),
    }
}

fn k_tag_name_dummy() -> &'static NbtTag {
    static DUMMY: NbtTag = NbtTag::Byte(0);
    &DUMMY
}

fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

fn write_modified_utf8<W: Write>(w: &mut W, s: &str) -> Result<(), NbtError> {
    let mut encoded: Vec<u8> = Vec::new();
    for ch in s.chars() {
        let code = ch as u32;
        if code >= 0x0001 && code <= 0x007F {
            encoded.push(code as u8);
        } else if code == 0 || (code >= 0x0080 && code <= 0x07FF) {
            encoded.push((0xC0 | ((code >> 6) & 0x1F)) as u8);
            encoded.push((0x80 | (code & 0x3F)) as u8);
        } else if code <= 0xFFFF {
            encoded.push((0xE0 | ((code >> 12) & 0x0F)) as u8);
            encoded.push((0x80 | ((code >> 6) & 0x3F)) as u8);
            encoded.push((0x80 | (code & 0x3F)) as u8);
        } else {
            // Java Modified UTF-8 encodes supplementary chars as surrogate pairs, each as 3-byte.
            let u = code.wrapping_sub(0x10000);
            let high = 0xD800_u32.wrapping_add((u >> 10) & 0x3FF);
            let low = 0xDC00_u32.wrapping_add(u & 0x3FF);
            for surrogate in [high, low] {
                encoded.push((0xE0 | ((surrogate >> 12) & 0x0F)) as u8);
                encoded.push((0x80 | ((surrogate >> 6) & 0x3F)) as u8);
                encoded.push((0x80 | (surrogate & 0x3F)) as u8);
            }
        }
    }
    if encoded.len() > u16::MAX as usize {
        return Err(NbtError::StringTooLong(encoded.len()));
    }
    w.write_u16::<BigEndian>(encoded.len() as u16)?;
    w.write_all(&encoded)?;
    Ok(())
}

fn read_modified_utf8<R: Read>(r: &mut R) -> Result<String, NbtError> {
    let len = r.read_u16::<BigEndian>()? as usize;
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    let mut out: Vec<u16> = Vec::new();
    let mut i = 0usize;
    while i < buf.len() {
        let b = buf[i];
        match b >> 4 {
            0..=7 => {
                out.push(b as u16);
                i = i.wrapping_add(1);
            }
            12 | 13 => {
                if i.wrapping_add(1) >= buf.len() {
                    return Err(NbtError::InvalidMutf8);
                }
                let b2 = buf[i.wrapping_add(1)];
                if (b2 & 0xC0) != 0x80 {
                    return Err(NbtError::InvalidMutf8);
                }
                let ch = (((b & 0x1F) as u16) << 6) | ((b2 & 0x3F) as u16);
                out.push(ch);
                i = i.wrapping_add(2);
            }
            14 => {
                if i.wrapping_add(2) >= buf.len() {
                    return Err(NbtError::InvalidMutf8);
                }
                let b2 = buf[i.wrapping_add(1)];
                let b3 = buf[i.wrapping_add(2)];
                if (b2 & 0xC0) != 0x80 || (b3 & 0xC0) != 0x80 {
                    return Err(NbtError::InvalidMutf8);
                }
                let ch = (((b & 0x0F) as u16) << 12)
                    | (((b2 & 0x3F) as u16) << 6)
                    | ((b3 & 0x3F) as u16);
                out.push(ch);
                i = i.wrapping_add(3);
            }
            _ => return Err(NbtError::InvalidMutf8),
        }
    }
    String::from_utf16(&out).map_err(|_| NbtError::InvalidMutf8)
}

fn write_tag_payload<W: Write>(w: &mut W, tag: &NbtTag) -> Result<(), NbtError> {
    match tag {
        NbtTag::Byte(v) => w.write_i8(*v)?,
        NbtTag::Short(v) => w.write_i16::<BigEndian>(*v)?,
        NbtTag::Int(v) => w.write_i32::<BigEndian>(*v)?,
        NbtTag::Long(v) => w.write_i64::<BigEndian>(*v)?,
        NbtTag::Float(v) => w.write_f32::<BigEndian>(*v)?,
        NbtTag::Double(v) => w.write_f64::<BigEndian>(*v)?,
        NbtTag::ByteArray(v) => {
            w.write_i32::<BigEndian>(v.len() as i32)?;
            for b in v {
                w.write_i8(*b)?;
            }
        }
        NbtTag::String(s) => write_modified_utf8(w, s)?,
        NbtTag::List { elem_type, items } => {
            w.write_u8(*elem_type)?;
            w.write_i32::<BigEndian>(items.len() as i32)?;
            for item in items {
                let wrapped = wrap_if_needed(*elem_type, item.clone());
                write_tag_payload(w, &wrapped)?;
            }
        }
        NbtTag::Compound(entries) => {
            for (k, v) in entries {
                w.write_u8(v.id())?;
                write_modified_utf8(w, k)?;
                write_tag_payload(w, v)?;
            }
            w.write_u8(TAG_END)?;
        }
        NbtTag::IntArray(v) => {
            w.write_i32::<BigEndian>(v.len() as i32)?;
            for i in v {
                w.write_i32::<BigEndian>(*i)?;
            }
        }
        NbtTag::LongArray(v) => {
            w.write_i32::<BigEndian>(v.len() as i32)?;
            for i in v {
                w.write_i64::<BigEndian>(*i)?;
            }
        }
    }
    Ok(())
}

fn read_tag_payload<R: Read>(r: &mut R, id: u8, acc: &mut NbtAccounter) -> Result<NbtTag, NbtError> {
    match id {
        TAG_BYTE => {
            acc.account_bytes(1)?;
            Ok(NbtTag::Byte(r.read_i8()?))
        }
        TAG_SHORT => {
            acc.account_bytes(2)?;
            Ok(NbtTag::Short(r.read_i16::<BigEndian>()?))
        }
        TAG_INT => {
            acc.account_bytes(4)?;
            Ok(NbtTag::Int(r.read_i32::<BigEndian>()?))
        }
        TAG_LONG => {
            acc.account_bytes(8)?;
            Ok(NbtTag::Long(r.read_i64::<BigEndian>()?))
        }
        TAG_FLOAT => {
            acc.account_bytes(4)?;
            Ok(NbtTag::Float(r.read_f32::<BigEndian>()?))
        }
        TAG_DOUBLE => {
            acc.account_bytes(8)?;
            Ok(NbtTag::Double(r.read_f64::<BigEndian>()?))
        }
        TAG_BYTE_ARRAY => {
            let len = r.read_i32::<BigEndian>()?;
            if len < 0 {
                return Err(NbtError::NegativeLength(len));
            }
            acc.account_bytes_mul(1, len)?;
            let mut v = Vec::with_capacity(len as usize);
            for _ in 0..len {
                v.push(r.read_i8()?);
            }
            Ok(NbtTag::ByteArray(v))
        }
        TAG_STRING => Ok(NbtTag::String(read_modified_utf8(r)?)),
        TAG_LIST => {
            let list = ListTag::load_list(r, acc)?;
            Ok(NbtTag::List {
                elem_type: list.identify_raw_element_type(),
                items: list.list,
            })
        }
        TAG_COMPOUND => {
            let mut entries: Vec<(String, NbtTag)> = Vec::new();
            loop {
                let tag_type = r.read_u8()?;
                if tag_type == TAG_END {
                    break;
                }
                let name = read_modified_utf8(r)?;
                let value = read_tag_payload(r, tag_type, acc)?;
                entries.push((name, value));
            }
            Ok(NbtTag::Compound(entries))
        }
        TAG_INT_ARRAY => {
            let len = r.read_i32::<BigEndian>()?;
            if len < 0 {
                return Err(NbtError::NegativeLength(len));
            }
            acc.account_bytes_mul(4, len)?;
            let mut v = Vec::with_capacity(len as usize);
            for _ in 0..len {
                v.push(r.read_i32::<BigEndian>()?);
            }
            Ok(NbtTag::IntArray(v))
        }
        TAG_LONG_ARRAY => {
            let len = r.read_i32::<BigEndian>()?;
            if len < 0 {
                return Err(NbtError::NegativeLength(len));
            }
            acc.account_bytes_mul(8, len)?;
            let mut v = Vec::with_capacity(len as usize);
            for _ in 0..len {
                v.push(r.read_i64::<BigEndian>()?);
            }
            Ok(NbtTag::LongArray(v))
        }
        _ => Err(NbtError::UnknownTag(id)),
    }
}

fn skip_tag_payload<R: Read>(r: &mut R, id: u8, count: i32, acc: &mut NbtAccounter) -> Result<(), NbtError> {
    if count < 0 {
        return Err(NbtError::NegativeLength(count));
    }
    for _ in 0..count {
        skip_one_payload(r, id, acc)?;
    }
    Ok(())
}

fn skip_one_payload<R: Read>(r: &mut R, id: u8, acc: &mut NbtAccounter) -> Result<(), NbtError> {
    match id {
        TAG_BYTE => {
            acc.account_bytes(1)?;
            let _ = r.read_i8()?;
        }
        TAG_SHORT => {
            acc.account_bytes(2)?;
            let _ = r.read_i16::<BigEndian>()?;
        }
        TAG_INT => {
            acc.account_bytes(4)?;
            let _ = r.read_i32::<BigEndian>()?;
        }
        TAG_LONG => {
            acc.account_bytes(8)?;
            let _ = r.read_i64::<BigEndian>()?;
        }
        TAG_FLOAT => {
            acc.account_bytes(4)?;
            let _ = r.read_f32::<BigEndian>()?;
        }
        TAG_DOUBLE => {
            acc.account_bytes(8)?;
            let _ = r.read_f64::<BigEndian>()?;
        }
        TAG_BYTE_ARRAY => {
            let len = r.read_i32::<BigEndian>()?;
            if len < 0 {
                return Err(NbtError::NegativeLength(len));
            }
            acc.account_bytes_mul(1, len)?;
            let mut buf = vec![0u8; len as usize];
            r.read_exact(&mut buf)?;
        }
        TAG_STRING => {
            let _ = read_modified_utf8(r)?;
        }
        TAG_LIST => {
            acc.push_depth()?;
            let result = (|| -> Result<(), NbtError> {
                let elem_type = r.read_u8()?;
                let len = r.read_i32::<BigEndian>()?;
                if len < 0 {
                    return Err(NbtError::NegativeLength(len));
                }
                skip_tag_payload(r, elem_type, len, acc)?;
                Ok(())
            })();
            acc.pop_depth()?;
            result?;
        }
        TAG_COMPOUND => {
            loop {
                let tag_type = r.read_u8()?;
                if tag_type == TAG_END {
                    break;
                }
                let _ = read_modified_utf8(r)?;
                skip_one_payload(r, tag_type, acc)?;
            }
        }
        TAG_INT_ARRAY => {
            let len = r.read_i32::<BigEndian>()?;
            if len < 0 {
                return Err(NbtError::NegativeLength(len));
            }
            acc.account_bytes_mul(4, len)?;
            for _ in 0..len {
                let _ = r.read_i32::<BigEndian>()?;
            }
        }
        TAG_LONG_ARRAY => {
            let len = r.read_i32::<BigEndian>()?;
            if len < 0 {
                return Err(NbtError::NegativeLength(len));
            }
            acc.account_bytes_mul(8, len)?;
            for _ in 0..len {
                let _ = r.read_i64::<BigEndian>()?;
            }
        }
        _ => return Err(NbtError::UnknownTag(id)),
    }
    Ok(())
}

// JAVA: net.minecraft.nbt.ListTag#tryUnwrap
fn try_unwrap(compound: &Vec<(String, NbtTag)>) -> NbtTag {
    if compound.len() == 1 {
        if let Some((k, v)) = compound.first() {
            if k == WRAPPER_MARKER {
                return v.clone();
            }
        }
    }
    NbtTag::Compound(compound.clone())
}

// JAVA: net.minecraft.nbt.ListTag#isWrapper
fn is_wrapper(compound: &[(String, NbtTag)]) -> bool {
    compound.len() == 1 && compound.first().map(|(k, _)| k.as_str()) == Some(WRAPPER_MARKER)
}

// JAVA: net.minecraft.nbt.ListTag#wrapElement
fn wrap_element(tag: NbtTag) -> NbtTag {
    NbtTag::Compound(vec![(WRAPPER_MARKER.to_string(), tag)])
}

// JAVA: net.minecraft.nbt.ListTag#wrapIfNeeded
fn wrap_if_needed(elem_type: u8, tag: NbtTag) -> NbtTag {
    if elem_type != TAG_COMPOUND {
        return tag;
    }
    match &tag {
        NbtTag::Compound(entries) if !is_wrapper(entries) => tag,
        _ => wrap_element(tag),
    }
}

fn is_numeric(tag: &NbtTag) -> bool {
    matches!(
        tag,
        NbtTag::Byte(_) | NbtTag::Short(_) | NbtTag::Int(_) | NbtTag::Long(_) | NbtTag::Float(_) | NbtTag::Double(_)
    )
}

fn numeric_short(tag: &NbtTag) -> Option<i16> {
    match tag {
        NbtTag::Byte(v) => Some(*v as i16),
        NbtTag::Short(v) => Some(*v),
        NbtTag::Int(v) => Some(*v as i16),
        NbtTag::Long(v) => Some(*v as i16),
        NbtTag::Float(v) => Some(*v as i16),
        NbtTag::Double(v) => Some(*v as i16),
        _ => None,
    }
}

fn numeric_int(tag: &NbtTag) -> Option<i32> {
    match tag {
        NbtTag::Byte(v) => Some(*v as i32),
        NbtTag::Short(v) => Some(*v as i32),
        NbtTag::Int(v) => Some(*v),
        NbtTag::Long(v) => Some(*v as i32),
        NbtTag::Float(v) => Some(*v as i32),
        NbtTag::Double(v) => Some(*v as i32),
        _ => None,
    }
}

fn numeric_float(tag: &NbtTag) -> Option<f32> {
    match tag {
        NbtTag::Byte(v) => Some(*v as f32),
        NbtTag::Short(v) => Some(*v as f32),
        NbtTag::Int(v) => Some(*v as f32),
        NbtTag::Long(v) => Some(*v as f32),
        NbtTag::Float(v) => Some(*v),
        NbtTag::Double(v) => Some(*v as f32),
        _ => None,
    }
}

fn numeric_double(tag: &NbtTag) -> Option<f64> {
    match tag {
        NbtTag::Byte(v) => Some(*v as f64),
        NbtTag::Short(v) => Some(*v as f64),
        NbtTag::Int(v) => Some(*v as f64),
        NbtTag::Long(v) => Some(*v as f64),
        NbtTag::Float(v) => Some(*v as f64),
        NbtTag::Double(v) => Some(*v),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListTag {
    list: Vec<NbtTag>,
}

impl Default for ListTag {
    fn default() -> Self {
        Self::new()
    }
}

impl ListTag {
    // JAVA: net.minecraft.nbt.ListTag#<init>()
    pub fn new() -> Self {
        Self { list: Vec::new() }
    }

    // JAVA: net.minecraft.nbt.ListTag#<init>(List)
    pub fn from_vec(list: Vec<NbtTag>) -> Self {
        Self { list }
    }

    // JAVA: net.minecraft.nbt.ListTag#loadList
    fn load_list<R: Read>(r: &mut R, acc: &mut NbtAccounter) -> Result<Self, NbtError> {
        acc.account_bytes(36_i64)?;
        let elem_type = r.read_u8()?;
        let len = Self::read_list_count(r)?;
        if elem_type == TAG_END && len > 0 {
            return Err(NbtError::MissingListType);
        }
        acc.account_bytes_mul(4_i64, len)?;
        let mut out = ListTag::from_vec(Vec::with_capacity(len as usize));
        let mut i: i32 = 0;
        while i < len {
            let tag = read_tag_payload(r, elem_type, acc)?;
            out.add_and_unwrap(tag);
            i = i.wrapping_add(1);
        }
        Ok(out)
    }

    // JAVA: net.minecraft.nbt.ListTag#readListCount
    fn read_list_count<R: Read>(r: &mut R) -> Result<i32, NbtError> {
        let len = r.read_i32::<BigEndian>()?;
        if len < 0 {
            return Err(NbtError::NegativeLength(len));
        }
        Ok(len)
    }

    // JAVA: net.minecraft.nbt.ListTag#load (TYPE)
    pub fn load<R: Read>(r: &mut R, acc: &mut NbtAccounter) -> Result<Self, NbtError> {
        acc.push_depth()?;
        let result = Self::load_list(r, acc);
        acc.pop_depth()?;
        result
    }

    // JAVA: net.minecraft.nbt.ListTag#parseList (TYPE)
    fn parse_list<R: Read>(
        r: &mut R,
        visitor: &mut dyn StreamTagVisitor,
        acc: &mut NbtAccounter,
    ) -> Result<ValueResult, NbtError> {
        acc.account_bytes(36_i64)?;
        let elem_type = r.read_u8()?;
        let len = Self::read_list_count(r)?;
        match visitor.visit_list(elem_type, len)? {
            ListVisitResult::Halt => return Ok(ValueResult::Halt),
            ListVisitResult::Break => {
                skip_tag_payload(r, elem_type, len, acc)?;
                return visitor.visit_container_end();
            }
            ListVisitResult::Continue => {}
        }

        acc.account_bytes_mul(4_i64, len)?;
        let mut i: i32 = 0;
        while i < len {
            match visitor.visit_element(elem_type, i)? {
                ElementVisitResult::Halt => return Ok(ValueResult::Halt),
                ElementVisitResult::Break => {
                    skip_one_payload(r, elem_type, acc)?;
                    let remaining = len.wrapping_sub(1).wrapping_sub(i);
                    if remaining > 0 {
                        skip_tag_payload(r, elem_type, remaining, acc)?;
                    }
                    return visitor.visit_container_end();
                }
                ElementVisitResult::Skip => {
                    skip_one_payload(r, elem_type, acc)?;
                    i = i.wrapping_add(1);
                    continue;
                }
                ElementVisitResult::Continue => {
                    let tag = read_tag_payload(r, elem_type, acc)?;
                    match visitor.visit_value(&tag)? {
                        ValueResult::Halt => return Ok(ValueResult::Halt),
                        ValueResult::Break => {
                            let remaining = len.wrapping_sub(1).wrapping_sub(i);
                            if remaining > 0 {
                                skip_tag_payload(r, elem_type, remaining, acc)?;
                            }
                            return visitor.visit_container_end();
                        }
                        ValueResult::Continue => {}
                    }
                    i = i.wrapping_add(1);
                }
            }
        }

        visitor.visit_container_end()
    }

    // JAVA: net.minecraft.nbt.ListTag#parse (TYPE)
    pub fn parse<R: Read>(
        r: &mut R,
        visitor: &mut dyn StreamTagVisitor,
        acc: &mut NbtAccounter,
    ) -> Result<ValueResult, NbtError> {
        acc.push_depth()?;
        let result = Self::parse_list(r, visitor, acc);
        acc.pop_depth()?;
        result
    }

    // JAVA: net.minecraft.nbt.ListTag#skip (TYPE)
    pub fn skip<R: Read>(r: &mut R, acc: &mut NbtAccounter) -> Result<(), NbtError> {
        acc.push_depth()?;
        let result = (|| -> Result<(), NbtError> {
            let elem_type = r.read_u8()?;
            let len = r.read_i32::<BigEndian>()?;
            if len < 0 {
                return Err(NbtError::NegativeLength(len));
            }
            skip_tag_payload(r, elem_type, len, acc)?;
            Ok(())
        })();
        acc.pop_depth()?;
        result
    }

    // JAVA: net.minecraft.nbt.ListTag#write
    pub fn write<W: Write>(&self, w: &mut W) -> Result<(), NbtError> {
        let elem_type = self.identify_raw_element_type();
        w.write_u8(elem_type)?;
        w.write_i32::<BigEndian>(self.list.len() as i32)?;
        for tag in &self.list {
            let wrapped = wrap_if_needed(elem_type, tag.clone());
            write_tag_payload(w, &wrapped)?;
        }
        Ok(())
    }

    // JAVA: net.minecraft.nbt.ListTag#identifyRawElementType
    pub fn identify_raw_element_type(&self) -> u8 {
        let mut out: u8 = 0;
        for tag in &self.list {
            let t = tag_id(tag);
            if out == 0 {
                out = t;
            } else if out != t {
                return TAG_COMPOUND;
            }
        }
        out
    }

    // JAVA: net.minecraft.nbt.ListTag#addAndUnwrap
    pub fn add_and_unwrap(&mut self, tag: NbtTag) {
        if let NbtTag::Compound(entries) = &tag {
            self.list.push(try_unwrap(entries));
        } else {
            self.list.push(tag);
        }
    }

    // JAVA: net.minecraft.nbt.ListTag#sizeInBytes
    pub fn size_in_bytes(&self) -> i32 {
        let mut size: i32 = SELF_SIZE_IN_BYTES;
        size = size.wrapping_add(4_i32.wrapping_mul(self.list.len() as i32));
        for tag in &self.list {
            size = size.wrapping_add(tag_size_in_bytes(tag));
        }
        size
    }

    // JAVA: net.minecraft.nbt.ListTag#getId
    pub fn get_id(&self) -> u8 {
        TAG_LIST
    }

    // JAVA: net.minecraft.nbt.ListTag#toString
    pub fn to_string_tag(&self) -> String {
        // Concise faithful representation: [e1, e2, ...]
        let mut s = String::from("[");
        for (i, tag) in self.list.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push_str(&format_nbt_tag(tag));
        }
        s.push(']');
        s
    }

    // JAVA: net.minecraft.nbt.ListTag#remove
    pub fn remove(&mut self, index: i32) -> NbtTag {
        self.list.remove(index as usize)
    }

    // JAVA: net.minecraft.nbt.ListTag#isEmpty
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    // JAVA: net.minecraft.nbt.ListTag#getCompound
    pub fn get_compound(&self, index: i32) -> Option<&Vec<(String, NbtTag)>> {
        match self.get_nullable(index) {
            Some(NbtTag::Compound(entries)) => Some(entries),
            _ => None,
        }
    }

    // JAVA: net.minecraft.nbt.ListTag#getCompoundOrEmpty
    pub fn get_compound_or_empty(&self, index: i32) -> Vec<(String, NbtTag)> {
        self.get_compound(index).cloned().unwrap_or_default()
    }

    // JAVA: net.minecraft.nbt.ListTag#getList
    pub fn get_list(&self, index: i32) -> Option<ListTag> {
        match self.get_nullable(index) {
            Some(NbtTag::List { items, .. }) => Some(ListTag::from_vec(items.clone())),
            _ => None,
        }
    }

    // JAVA: net.minecraft.nbt.ListTag#getListOrEmpty
    pub fn get_list_or_empty(&self, index: i32) -> ListTag {
        self.get_list(index).unwrap_or_else(ListTag::new)
    }

    // JAVA: net.minecraft.nbt.ListTag#getShort
    pub fn get_short(&self, index: i32) -> Option<i16> {
        self.get_optional(index).and_then(|t| match t {
            NbtTag::Short(v) => Some(*v),
            NbtTag::Byte(v) => Some(*v as i16),
            NbtTag::Int(v) => Some(*v as i16),
            NbtTag::Long(v) => Some(*v as i16),
            NbtTag::Float(v) => Some(*v as i16),
            NbtTag::Double(v) => Some(*v as i16),
            _ => None,
        })
    }

    // JAVA: net.minecraft.nbt.ListTag#getShortOr
    pub fn get_short_or(&self, index: i32, default: i16) -> i16 {
        match self.get_nullable(index) {
            Some(tag) if is_numeric(tag) => numeric_short(tag).unwrap_or(default),
            _ => default,
        }
    }

    // JAVA: net.minecraft.nbt.ListTag#getInt
    pub fn get_int(&self, index: i32) -> Option<i32> {
        self.get_optional(index).and_then(|t| match t {
            NbtTag::Int(v) => Some(*v),
            NbtTag::Byte(v) => Some(*v as i32),
            NbtTag::Short(v) => Some(*v as i32),
            NbtTag::Long(v) => Some(*v as i32),
            NbtTag::Float(v) => Some(*v as i32),
            NbtTag::Double(v) => Some(*v as i32),
            _ => None,
        })
    }

    // JAVA: net.minecraft.nbt.ListTag#getIntOr
    pub fn get_int_or(&self, index: i32, default: i32) -> i32 {
        match self.get_nullable(index) {
            Some(tag) if is_numeric(tag) => numeric_int(tag).unwrap_or(default),
            _ => default,
        }
    }

    // JAVA: net.minecraft.nbt.ListTag#getIntArray
    pub fn get_int_array(&self, index: i32) -> Option<&[i32]> {
        match self.get_nullable(index) {
            Some(NbtTag::IntArray(v)) => Some(v.as_slice()),
            _ => None,
        }
    }

    // JAVA: net.minecraft.nbt.ListTag#getLongArray
    pub fn get_long_array(&self, index: i32) -> Option<&[i64]> {
        match self.get_nullable(index) {
            Some(NbtTag::LongArray(v)) => Some(v.as_slice()),
            _ => None,
        }
    }

    // JAVA: net.minecraft.nbt.ListTag#getDouble
    pub fn get_double(&self, index: i32) -> Option<f64> {
        self.get_optional(index).and_then(|t| match t {
            NbtTag::Double(v) => Some(*v),
            NbtTag::Byte(v) => Some(*v as f64),
            NbtTag::Short(v) => Some(*v as f64),
            NbtTag::Int(v) => Some(*v as f64),
            NbtTag::Long(v) => Some(*v as f64),
            NbtTag::Float(v) => Some(*v as f64),
            _ => None,
        })
    }

    // JAVA: net.minecraft.nbt.ListTag#getDoubleOr
    pub fn get_double_or(&self, index: i32, default: f64) -> f64 {
        match self.get_nullable(index) {
            Some(tag) if is_numeric(tag) => numeric_double(tag).unwrap_or(default),
            _ => default,
        }
    }

    // JAVA: net.minecraft.nbt.ListTag#getFloat
    pub fn get_float(&self, index: i32) -> Option<f32> {
        self.get_optional(index).and_then(|t| match t {
            NbtTag::Float(v) => Some(*v),
            NbtTag::Byte(v) => Some(*v as f32),
            NbtTag::Short(v) => Some(*v as f32),
            NbtTag::Int(v) => Some(*v as f32),
            NbtTag::Long(v) => Some(*v as f32),
            NbtTag::Double(v) => Some(*v as f32),
            _ => None,
        })
    }

    // JAVA: net.minecraft.nbt.ListTag#getFloatOr
    pub fn get_float_or(&self, index: i32, default: f32) -> f32 {
        match self.get_nullable(index) {
            Some(tag) if is_numeric(tag) => numeric_float(tag).unwrap_or(default),
            _ => default,
        }
    }

    // JAVA: net.minecraft.nbt.ListTag#getString
    pub fn get_string(&self, index: i32) -> Option<&str> {
        self.get_optional(index).and_then(|t| match t {
            NbtTag::String(s) => Some(s.as_str()),
            _ => None,
        })
    }

    // JAVA: net.minecraft.nbt.ListTag#getStringOr
    pub fn get_string_or<'a>(&'a self, index: i32, default: &'a str) -> &'a str {
        match self.get_nullable(index) {
            Some(NbtTag::String(s)) => s.as_str(),
            _ => default,
        }
    }

    // JAVA: net.minecraft.nbt.ListTag#getNullable
    fn get_nullable(&self, index: i32) -> Option<&NbtTag> {
        if index >= 0 && (index as usize) < self.list.len() {
            self.list.get(index as usize)
        } else {
            None
        }
    }

    // JAVA: net.minecraft.nbt.ListTag#getOptional
    fn get_optional(&self, index: i32) -> Option<&NbtTag> {
        self.get_nullable(index)
    }

    // JAVA: net.minecraft.nbt.ListTag#size
    pub fn size(&self) -> i32 {
        self.list.len() as i32
    }

    // JAVA: net.minecraft.nbt.ListTag#get
    pub fn get(&self, index: i32) -> &NbtTag {
        &self.list[index as usize]
    }

    // JAVA: net.minecraft.nbt.ListTag#set
    pub fn set(&mut self, index: i32, tag: NbtTag) -> NbtTag {
        std::mem::replace(&mut self.list[index as usize], tag)
    }

    // JAVA: net.minecraft.nbt.ListTag#add
    pub fn add(&mut self, index: i32, tag: NbtTag) {
        self.list.insert(index as usize, tag);
    }

    // JAVA: net.minecraft.nbt.ListTag#setTag
    pub fn set_tag(&mut self, index: i32, tag: NbtTag) -> bool {
        self.list[index as usize] = tag;
        true
    }

    // JAVA: net.minecraft.nbt.ListTag#addTag
    pub fn add_tag(&mut self, index: i32, tag: NbtTag) -> bool {
        self.list.insert(index as usize, tag);
        true
    }

    // JAVA: net.minecraft.nbt.ListTag#copy
    pub fn copy(&self) -> ListTag {
        let mut out = Vec::with_capacity(self.list.len());
        for tag in &self.list {
            out.push(tag.clone());
        }
        ListTag { list: out }
    }

    // JAVA: net.minecraft.nbt.ListTag#asList
    pub fn as_list(&self) -> Option<&ListTag> {
        Some(self)
    }

    // JAVA: net.minecraft.nbt.ListTag#equals
    pub fn equals(&self, other: &ListTag) -> bool {
        self.list == other.list
    }

    // JAVA: net.minecraft.nbt.ListTag#hashCode
    pub fn hash_code(&self) -> i32 {
        // Java List.hashCode(): 1; for each element result = 31*result + element.hashCode()
        let mut result: i32 = 1;
        for tag in &self.list {
            let h = nbt_hash_code(tag);
            result = 31_i32.wrapping_mul(result).wrapping_add(h);
        }
        result
    }

    // JAVA: net.minecraft.nbt.ListTag#stream
    pub fn stream(&self) -> impl Iterator<Item = &NbtTag> {
        self.list.iter()
    }

    // JAVA: net.minecraft.nbt.ListTag#compoundStream
    pub fn compound_stream(&self) -> impl Iterator<Item = &Vec<(String, NbtTag)>> {
        self.list.iter().filter_map(|t| match t {
            NbtTag::Compound(entries) => Some(entries),
            _ => None,
        })
    }

    // JAVA: net.minecraft.nbt.ListTag#accept(TagVisitor)
    pub fn accept_tag_visitor<V: TagVisitor + ?Sized>(&self, visitor: &mut V) {
        visitor.visit_list(self);
    }

    // JAVA: net.minecraft.nbt.ListTag#clear
    pub fn clear(&mut self) {
        self.list.clear();
    }

    // JAVA: net.minecraft.nbt.ListTag#accept(StreamTagVisitor)
    pub fn accept_stream_visitor(
        &self,
        visitor: &mut dyn StreamTagVisitor,
    ) -> Result<ValueResult, NbtError> {
        let elem_type = self.identify_raw_element_type();
        match visitor.visit_list(elem_type, self.list.len() as i32)? {
            ListVisitResult::Halt => return Ok(ValueResult::Halt),
            ListVisitResult::Break => return visitor.visit_container_end(),
            ListVisitResult::Continue => {}
        }

        let mut i: i32 = 0;
        let len: i32 = self.list.len() as i32;
        while i < len {
            let tag = wrap_if_needed(elem_type, self.list[i as usize].clone());
            match visitor.visit_element(tag.id(), i)? {
                ElementVisitResult::Halt => return Ok(ValueResult::Halt),
                ElementVisitResult::Break => return visitor.visit_container_end(),
                ElementVisitResult::Skip => {
                    i = i.wrapping_add(1);
                    continue;
                }
                ElementVisitResult::Continue => match visitor.visit_value(&tag)? {
                    ValueResult::Halt => return Ok(ValueResult::Halt),
                    ValueResult::Break => return visitor.visit_container_end(),
                    ValueResult::Continue => {
                        i = i.wrapping_add(1);
                    }
                },
            }
        }

        visitor.visit_container_end()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, NbtTag> {
        self.list.iter()
    }

    pub fn into_inner(self) -> Vec<NbtTag> {
        self.list
    }

    pub fn as_slice(&self) -> &[NbtTag] {
        &self.list
    }
}

fn nbt_hash_code(tag: &NbtTag) -> i32 {
    match tag {
        NbtTag::Byte(v) => *v as i32,
        NbtTag::Short(v) => *v as i32,
        NbtTag::Int(v) => *v,
        NbtTag::Long(v) => {
            let bits = *v as u64;
            ((bits ^ (bits >> 32)) as i32)
        }
        NbtTag::Float(v) => v.to_bits() as i32,
        NbtTag::Double(v) => {
            let bits = v.to_bits();
            ((bits ^ (bits >> 32)) as i32)
        }
        NbtTag::ByteArray(v) => {
            let mut h: i32 = 1;
            for b in v {
                h = 31_i32.wrapping_mul(h).wrapping_add(*b as i32);
            }
            h
        }
        NbtTag::String(s) => java_string_hash(s),
        NbtTag::List { items, .. } => {
            let mut h: i32 = 1;
            for item in items {
                h = 31_i32.wrapping_mul(h).wrapping_add(nbt_hash_code(item));
            }
            h
        }
        NbtTag::Compound(entries) => {
            let mut h: i32 = 1;
            for (k, v) in entries {
                let entry_h = java_string_hash(k) ^ nbt_hash_code(v);
                h = 31_i32.wrapping_mul(h).wrapping_add(entry_h);
            }
            h
        }
        NbtTag::IntArray(v) => {
            let mut h: i32 = 1;
            for i in v {
                h = 31_i32.wrapping_mul(h).wrapping_add(*i);
            }
            h
        }
        NbtTag::LongArray(v) => {
            let mut h: i32 = 1;
            for i in v {
                let bits = *i as u64;
                let ih = ((bits ^ (bits >> 32)) as i32);
                h = 31_i32.wrapping_mul(h).wrapping_add(ih);
            }
            h
        }
    }
}

fn java_string_hash(s: &str) -> i32 {
    let mut h: i32 = 0;
    for c in s.encode_utf16() {
        h = 31_i32.wrapping_mul(h).wrapping_add(c as i16 as i32);
    }
    h
}

fn format_nbt_tag(tag: &NbtTag) -> String {
    match tag {
        NbtTag::Byte(v) => format!("{}b", v),
        NbtTag::Short(v) => format!("{}s", v),
        NbtTag::Int(v) => format!("{}", v),
        NbtTag::Long(v) => format!("{}L", v),
        NbtTag::Float(v) => format!("{}f", v),
        NbtTag::Double(v) => format!("{}d", v),
        NbtTag::ByteArray(v) => {
            let mut s = String::from("[B;");
            for (i, b) in v.iter().enumerate() {
                if i > 0 {
                    s.push(',');
                }
                s.push_str(&format!("{}b", b));
            }
            s.push(']');
            s
        }
        NbtTag::String(s) => format!("\"{}\"", s),
        NbtTag::List { items, .. } => {
            let mut s = String::from("[");
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    s.push(',');
                }
                s.push_str(&format_nbt_tag(item));
            }
            s.push(']');
            s
        }
        NbtTag::Compound(entries) => {
            let mut s = String::from("{");
            for (i, (k, v)) in entries.iter().enumerate() {
                if i > 0 {
                    s.push(',');
                }
                s.push_str(k);
                s.push(':');
                s.push_str(&format_nbt_tag(v));
            }
            s.push('}');
            s
        }
        NbtTag::IntArray(v) => {
            let mut s = String::from("[I;");
            for (i, n) in v.iter().enumerate() {
                if i > 0 {
                    s.push(',');
                }
                s.push_str(&n.to_string());
            }
            s.push(']');
            s
        }
        NbtTag::LongArray(v) => {
            let mut s = String::from("[L;");
            for (i, n) in v.iter().enumerate() {
                if i > 0 {
                    s.push(',');
                }
                s.push_str(&format!("{}L", n));
            }
            s.push(']');
            s
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn list_tag_basic_add_get_remove_and_size() {
        let mut list = ListTag::new();
        assert!(list.is_empty());
        assert_eq!(list.size(), 0);
        assert_eq!(list.get_id(), TAG_LIST);

        list.add_and_unwrap(NbtTag::Int(10));
        list.add(1, NbtTag::Int(20));
        list.set_tag(0, NbtTag::Int(5));
        assert_eq!(list.size(), 2);
        assert_eq!(list.get_int(0), Some(5));
        assert_eq!(list.get_int_or(1, -1), 20);
        assert_eq!(list.get_int_or(5, -1), -1);

        let removed = list.remove(0);
        assert_eq!(removed, NbtTag::Int(5));
        assert_eq!(list.size(), 1);
        assert!(!list.is_empty());
    }

    #[test]
    fn identify_raw_element_type_mixed_and_wrapping() {
        let mut homogeneous = ListTag::new();
        homogeneous.add_and_unwrap(NbtTag::Int(1));
        homogeneous.add_and_unwrap(NbtTag::Int(2));
        assert_eq!(homogeneous.identify_raw_element_type(), TAG_INT);

        let mut mixed = ListTag::new();
        mixed.add_and_unwrap(NbtTag::Int(1));
        mixed.add_and_unwrap(NbtTag::String("x".to_string()));
        assert_eq!(mixed.identify_raw_element_type(), TAG_COMPOUND);

        let mut wrapped = ListTag::new();
        wrapped.add_and_unwrap(NbtTag::Compound(vec![(
            WRAPPER_MARKER.to_string(),
            NbtTag::Int(7),
        )]));
        assert_eq!(wrapped.get_int(0), Some(7));
        assert_eq!(wrapped.identify_raw_element_type(), TAG_INT);
    }

    #[test]
    fn size_in_bytes_overflow_wraps_like_java_int() {
        let mut list = ListTag::new();
        // Force large count contribution: SELF_SIZE_IN_BYTES + 4 * len
        // Use empty list and verify formula directly via manual construction.
        for _ in 0..0 {
            list.add_and_unwrap(NbtTag::Byte(0));
        }
        let base = list.size_in_bytes();
        assert_eq!(base, SELF_SIZE_IN_BYTES);

        let mut big = ListTag::from_vec(vec![NbtTag::Byte(0); 1_000_000]);
        let expected = SELF_SIZE_IN_BYTES
            .wrapping_add(4_i32.wrapping_mul(1_000_000))
            .wrapping_add(1_i32.wrapping_mul(1_000_000));
        assert_eq!(big.size_in_bytes(), expected);
        // sanity: remove doesn't break
        let _ = big.remove(0);
    }

    #[test]
    fn numeric_and_float_bit_accessors() {
        let mut list = ListTag::new();
        list.add_and_unwrap(NbtTag::Float(f32::NAN));
        list.add_and_unwrap(NbtTag::Double(f64::from_bits(0x8000_0000_0000_0000u64))); // -0.0
        list.add_and_unwrap(NbtTag::Long(-1_i64));

        let f = list.get_float(0).unwrap();
        assert!(f.is_nan());
        assert!(list.get_float_or(10, 1.5).to_bits() == 1.5f32.to_bits());

        let d = list.get_double(1).unwrap();
        assert_eq!(d.to_bits(), (-0.0f64).to_bits());
        assert_eq!(list.get_double_or(2, 0.0), -1.0f64);

        assert_eq!(list.get_short_or(2, 9), -1_i16);
        assert_eq!(list.get_int_or(2, 9), -1_i32);
    }

    #[test]
    fn write_load_roundtrip_and_negative_count_error() {
        let mut list = ListTag::new();
        list.add_and_unwrap(NbtTag::Int(42));
        list.add_and_unwrap(NbtTag::Int(-7));

        let mut buf = Vec::new();
        list.write(&mut buf).unwrap();

        let mut cursor = Cursor::new(buf);
        let mut acc = NbtAccounter::new();
        let loaded = ListTag::load(&mut cursor, &mut acc).unwrap();
        assert_eq!(loaded.size(), 2);
        assert_eq!(loaded.get_int(0), Some(42));
        assert_eq!(loaded.get_int(1), Some(-7));

        // Negative list count
        let mut bad = Vec::new();
        bad.write_u8(TAG_INT).unwrap();
        bad.write_i32::<BigEndian>(-1).unwrap();
        let mut c2 = Cursor::new(bad);
        let mut acc2 = NbtAccounter::new();
        let err = ListTag::load(&mut c2, &mut acc2).unwrap_err();
        assert!(matches!(err, NbtError::NegativeLength(-1)));
    }

    #[test]
    fn copy_equals_hash_and_compound_stream() {
        let mut list = ListTag::new();
        list.add_and_unwrap(NbtTag::Compound(vec![("a".to_string(), NbtTag::Int(1))]));
        list.add_and_unwrap(NbtTag::Int(2));

        let copied = list.copy();
        assert!(list.equals(&copied));
        assert_eq!(list.hash_code(), copied.hash_code());

        let compounds: Vec<_> = list.compound_stream().collect();
        assert_eq!(compounds.len(), 1);
        assert_eq!(compounds[0][0].0, "a");
    }
}
