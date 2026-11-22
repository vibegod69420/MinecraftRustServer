use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagType(pub u8);

impl TagType {
    pub const END: Self = Self(0);
    pub const BYTE: Self = Self(1);
    pub const SHORT: Self = Self(2);
    pub const INT: Self = Self(3);
    pub const LONG: Self = Self(4);
    pub const FLOAT: Self = Self(5);
    pub const DOUBLE: Self = Self(6);
    pub const BYTE_ARRAY: Self = Self(7);
    pub const STRING: Self = Self(8);
    pub const LIST: Self = Self(9);
    pub const COMPOUND: Self = Self(10);
    pub const INT_ARRAY: Self = Self(11);
    pub const LONG_ARRAY: Self = Self(12);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueResult {
    Continue,
    Break,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryResult {
    Enter,
    Skip,
    Break,
    Halt,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tag {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List { elem_type: u8, items: Vec<Tag> },
    Compound(Vec<(String, Tag)>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl Tag {
    pub fn id(&self) -> u8 {
        match self {
            Self::End => 0,
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
}

struct RootBuilder {
    result: Option<Tag>,
}

impl RootBuilder {
    // JAVA: net.minecraft.nbt.visitors.CollectToTag$RootBuilder#acceptValue
    fn accept_value(&mut self, value: Tag) {
        self.result = Some(value);
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag$RootBuilder#build
    fn build(&mut self) -> Option<Tag> {
        self.result.clone()
    }
}

struct ListBuilder {
    list: Vec<Tag>,
}

impl ListBuilder {
    // JAVA: net.minecraft.nbt.visitors.CollectToTag$ListBuilder#acceptValue
    fn accept_value(&mut self, value: Tag) {
        self.list.push(value);
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag$ListBuilder#build
    fn build(&mut self) -> Option<Tag> {
        let items = std::mem::take(&mut self.list);
        let elem_type = items.first().map(|t| t.id()).unwrap_or(0);
        Some(Tag::List { elem_type, items })
    }
}

struct CompoundBuilder {
    compound: Vec<(String, Tag)>,
    last_id: String,
}

impl CompoundBuilder {
    // JAVA: net.minecraft.nbt.visitors.CollectToTag$CompoundBuilder#acceptKey
    fn accept_key(&mut self, key: String) {
        self.last_id = key;
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag$CompoundBuilder#acceptValue
    fn accept_value(&mut self, value: Tag) {
        let key = self.last_id.clone();
        if let Some((_, existing)) = self.compound.iter_mut().find(|(k, _)| *k == key) {
            *existing = value;
        } else {
            self.compound.push((key, value));
        }
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag$CompoundBuilder#build
    fn build(&mut self) -> Option<Tag> {
        Some(Tag::Compound(std::mem::take(&mut self.compound)))
    }
}

enum ContainerBuilder {
    Root(RootBuilder),
    List(ListBuilder),
    Compound(CompoundBuilder),
}

impl ContainerBuilder {
    // JAVA: net.minecraft.nbt.visitors.CollectToTag$ContainerBuilder#acceptKey
    fn accept_key(&mut self, key: String) {
        if let Self::Compound(b) = self {
            b.accept_key(key);
        }
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag$ContainerBuilder#acceptValue
    fn accept_value(&mut self, value: Tag) {
        match self {
            Self::Root(b) => b.accept_value(value),
            Self::List(b) => b.accept_value(value),
            Self::Compound(b) => b.accept_value(value),
        }
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag$ContainerBuilder#build
    fn build(&mut self) -> Option<Tag> {
        match self {
            Self::Root(b) => b.build(),
            Self::List(b) => b.build(),
            Self::Compound(b) => b.build(),
        }
    }
}

pub struct CollectToTag {
    container_stack: VecDeque<ContainerBuilder>,
}

impl CollectToTag {
    // JAVA: net.minecraft.nbt.visitors.CollectToTag#<init>
    pub fn new() -> Self {
        let mut container_stack = VecDeque::new();
        container_stack.push_back(ContainerBuilder::Root(RootBuilder { result: None }));
        Self { container_stack }
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag#getResult
    pub fn get_result(&mut self) -> Option<Tag> {
        self.container_stack.front_mut().and_then(|b| b.build())
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag#depth
    pub fn depth(&self) -> i32 {
        (self.container_stack.len() as i32).wrapping_sub(1)
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag#appendEntry
    fn append_entry(&mut self, tag: Tag) {
        if let Some(last) = self.container_stack.back_mut() {
            last.accept_value(tag);
        }
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag#visitEnd
    pub fn visit_end(&mut self) -> ValueResult {
        self.append_entry(Tag::End);
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag#visit(String)
    pub fn visit_string(&mut self, value: String) -> ValueResult {
        self.append_entry(Tag::String(value));
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag#visit(byte)
    pub fn visit_byte(&mut self, value: i8) -> ValueResult {
        self.append_entry(Tag::Byte(value));
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag#visit(short)
    pub fn visit_short(&mut self, value: i16) -> ValueResult {
        self.append_entry(Tag::Short(value));
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag#visit(int)
    pub fn visit_int(&mut self, value: i32) -> ValueResult {
        self.append_entry(Tag::Int(value));
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag#visit(long)
    pub fn visit_long(&mut self, value: i64) -> ValueResult {
        self.append_entry(Tag::Long(value));
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag#visit(float)
    pub fn visit_float(&mut self, value: f32) -> ValueResult {
        let v = if value == 0.0f32 { 0.0f32 } else { value };
        self.append_entry(Tag::Float(v));
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag#visit(double)
    pub fn visit_double(&mut self, value: f64) -> ValueResult {
        let v = if value == 0.0f64 { 0.0f64 } else { value };
        self.append_entry(Tag::Double(v));
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag#visit(byte[])
    pub fn visit_byte_array(&mut self, value: Vec<i8>) -> ValueResult {
        self.append_entry(Tag::ByteArray(value));
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag#visit(int[])
    pub fn visit_int_array(&mut self, value: Vec<i32>) -> ValueResult {
        self.append_entry(Tag::IntArray(value));
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag#visit(long[])
    pub fn visit_long_array(&mut self, value: Vec<i64>) -> ValueResult {
        self.append_entry(Tag::LongArray(value));
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag#visitList
    pub fn visit_list(&mut self, _type: TagType, _size: i32) -> ValueResult {
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag#visitElement
    pub fn visit_element(&mut self, tag_type: TagType, _index: i32) -> EntryResult {
        self.enter_container_if_needed(tag_type);
        EntryResult::Enter
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag#visitEntry(TagType)
    pub fn visit_entry(&mut self, _tag_type: TagType) -> EntryResult {
        EntryResult::Enter
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag#visitEntry(TagType,String)
    pub fn visit_entry_named(&mut self, tag_type: TagType, name: String) -> EntryResult {
        if let Some(last) = self.container_stack.back_mut() {
            last.accept_key(name);
        }
        self.enter_container_if_needed(tag_type);
        EntryResult::Enter
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag#enterContainerIfNeeded
    fn enter_container_if_needed(&mut self, tag_type: TagType) {
        if tag_type == TagType::LIST {
            self.container_stack
                .push_back(ContainerBuilder::List(ListBuilder { list: Vec::new() }));
        } else if tag_type == TagType::COMPOUND {
            self.container_stack
                .push_back(ContainerBuilder::Compound(CompoundBuilder {
                    compound: Vec::new(),
                    last_id: String::new(),
                }));
        }
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag#visitContainerEnd
    pub fn visit_container_end(&mut self) -> ValueResult {
        if let Some(mut builder) = self.container_stack.pop_back() {
            if let Some(tag) = builder.build() {
                if let Some(last) = self.container_stack.back_mut() {
                    last.accept_value(tag);
                }
            }
        }
        ValueResult::Continue
    }

    // JAVA: net.minecraft.nbt.visitors.CollectToTag#visitRootEntry
    pub fn visit_root_entry(&mut self, tag_type: TagType) -> ValueResult {
        self.enter_container_if_needed(tag_type);
        ValueResult::Continue
    }
}

impl Default for CollectToTag {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_scalar_root_and_negative_int() {
        let mut c = CollectToTag::new();
        assert_eq!(c.depth(), 0);
        c.visit_int(i32::MIN);
        assert_eq!(c.get_result(), Some(Tag::Int(i32::MIN)));
    }

    #[test]
    fn compound_preserves_order_and_replaces() {
        let mut c = CollectToTag::new();
        c.visit_root_entry(TagType::COMPOUND);
        c.visit_entry_named(TagType::BYTE, "a".to_string());
        c.visit_byte(1);
        c.visit_entry_named(TagType::BYTE, "b".to_string());
        c.visit_byte(2);
        c.visit_entry_named(TagType::BYTE, "a".to_string());
        c.visit_byte(i8::MIN);
        c.visit_container_end();
        let Tag::Compound(entries) = c.get_result().unwrap() else {
            panic!("expected compound");
        };
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], ("a".to_string(), Tag::Byte(i8::MIN)));
        assert_eq!(entries[1], ("b".to_string(), Tag::Byte(2)));
    }

    #[test]
    fn nested_list_and_compound_with_depth() {
        let mut c = CollectToTag::new();
        assert_eq!(c.depth(), 0);
        c.visit_root_entry(TagType::COMPOUND);
        assert_eq!(c.depth(), 1);
        c.visit_entry_named(TagType::LIST, "xs".to_string());
        c.visit_list(TagType::INT, 2);
        assert_eq!(c.depth(), 2);
        c.visit_element(TagType::INT, 0);
        c.visit_int(10);
        c.visit_element(TagType::INT, 1);
        c.visit_int(-20);
        c.visit_container_end();
        assert_eq!(c.depth(), 1);
        c.visit_container_end();
        assert_eq!(c.depth(), 0);
        let tag = c.get_result().unwrap();
        assert_eq!(
            tag,
            Tag::Compound(vec![(
                "xs".to_string(),
                Tag::List {
                    elem_type: TagType::INT.0,
                    items: vec![Tag::Int(10), Tag::Int(-20)],
                }
            )])
        );
    }

    #[test]
    fn float_double_zero_normalization_and_nan_bits() {
        let mut c = CollectToTag::new();
        c.visit_root_entry(TagType::COMPOUND);
        c.visit_entry_named(TagType::FLOAT, "nz".to_string());
        c.visit_float(-0.0f32);
        c.visit_entry_named(TagType::DOUBLE, "nd".to_string());
        c.visit_double(-0.0f64);
        c.visit_entry_named(TagType::FLOAT, "nan".to_string());
        c.visit_float(f32::NAN);
        c.visit_entry_named(TagType::DOUBLE, "nan_d".to_string());
        c.visit_double(f64::NAN);
        c.visit_container_end();

        let Tag::Compound(entries) = c.get_result().unwrap() else {
            panic!("expected compound");
        };
        match &entries[0].1 {
            Tag::Float(v) => assert_eq!(v.to_bits(), 0.0f32.to_bits()),
            _ => panic!("expected float"),
        }
        match &entries[1].1 {
            Tag::Double(v) => assert_eq!(v.to_bits(), 0.0f64.to_bits()),
            _ => panic!("expected double"),
        }
        match &entries[2].1 {
            Tag::Float(v) => {
                assert!(v.is_nan());
                assert_eq!(v.to_bits(), f32::NAN.to_bits());
            }
            _ => panic!("expected float nan"),
        }
        match &entries[3].1 {
            Tag::Double(v) => {
                assert!(v.is_nan());
                assert_eq!(v.to_bits(), f64::NAN.to_bits());
            }
            _ => panic!("expected double nan"),
        }
    }

    #[test]
    fn extreme_integers_and_arrays() {
        let mut c = CollectToTag::new();
        c.visit_root_entry(TagType::COMPOUND);
        c.visit_entry_named(TagType::BYTE, "b".to_string());
        c.visit_byte(i8::MIN);
        c.visit_entry_named(TagType::SHORT, "s".to_string());
        c.visit_short(i16::MIN);
        c.visit_entry_named(TagType::LONG, "l".to_string());
        c.visit_long(i64::MIN);
        c.visit_entry_named(TagType::BYTE_ARRAY, "ba".to_string());
        c.visit_byte_array(vec![i8::MIN, -1, i8::MAX]);
        c.visit_entry_named(TagType::INT_ARRAY, "ia".to_string());
        c.visit_int_array(vec![i32::MIN, -1, i32::MAX]);
        c.visit_entry_named(TagType::LONG_ARRAY, "la".to_string());
        c.visit_long_array(vec![i64::MIN, 0, i64::MAX]);
        c.visit_container_end();

        let Tag::Compound(entries) = c.get_result().unwrap() else {
            panic!("expected compound");
        };
        assert_eq!(entries[0].1, Tag::Byte(i8::MIN));
        assert_eq!(entries[1].1, Tag::Short(i16::MIN));
        assert_eq!(entries[2].1, Tag::Long(i64::MIN));
        assert_eq!(entries[3].1, Tag::ByteArray(vec![i8::MIN, -1, i8::MAX]));
        assert_eq!(entries[4].1, Tag::IntArray(vec![i32::MIN, -1, i32::MAX]));
        assert_eq!(entries[5].1, Tag::LongArray(vec![i64::MIN, 0, i64::MAX]));
    }

    #[test]
    fn visit_end_and_empty_list_elem_type() {
        let mut c = CollectToTag::new();
        c.visit_root_entry(TagType::COMPOUND);
        c.visit_entry_named(TagType::END, "e".to_string());
        c.visit_end();
        c.visit_entry_named(TagType::LIST, "empty".to_string());
        c.visit_list(TagType::END, 0);
        c.visit_container_end();
        c.visit_container_end();
        let Tag::Compound(entries) = c.get_result().unwrap() else {
            panic!("expected compound");
        };
        assert_eq!(entries[0], ("e".to_string(), Tag::End));
        assert_eq!(
            entries[1],
            (
                "empty".to_string(),
                Tag::List {
                    elem_type: TagType::END.0,
                    items: vec![],
                }
            )
        );
    }
}
