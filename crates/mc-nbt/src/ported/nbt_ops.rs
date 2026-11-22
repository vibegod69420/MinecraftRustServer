// === FILE: nbt_ops.rs ===

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundTag {
    pub entries: Vec<(String, Tag)>,
}

impl CompoundTag {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn shallow_copy(&self) -> Self {
        self.clone()
    }

    pub fn put(&mut self, key: String, value: Tag) {
        if let Some(pos) = self.entries.iter().position(|(k, _)| k == &key) {
            self.entries[pos].1 = value;
        } else {
            self.entries.push((key, value));
        }
    }

    pub fn get(&self, key: &str) -> Option<&Tag> {
        self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn remove(&mut self, key: &str) {
        if let Some(pos) = self.entries.iter().position(|(k, _)| k == key) {
            self.entries.remove(pos);
        }
    }

    pub fn entry_set(&self) -> &[(String, Tag)] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for CompoundTag {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
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
    List(Vec<Tag>),
    Compound(CompoundTag),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Tag::End, Tag::End) => true,
            (Tag::Byte(a), Tag::Byte(b)) => a == b,
            (Tag::Short(a), Tag::Short(b)) => a == b,
            (Tag::Int(a), Tag::Int(b)) => a == b,
            (Tag::Long(a), Tag::Long(b)) => a == b,
            (Tag::Float(a), Tag::Float(b)) => a.to_bits() == b.to_bits(),
            (Tag::Double(a), Tag::Double(b)) => a.to_bits() == b.to_bits(),
            (Tag::ByteArray(a), Tag::ByteArray(b)) => a == b,
            (Tag::String(a), Tag::String(b)) => a == b,
            (Tag::List(a), Tag::List(b)) => a == b,
            (Tag::Compound(a), Tag::Compound(b)) => a == b,
            (Tag::IntArray(a), Tag::IntArray(b)) => a == b,
            (Tag::LongArray(a), Tag::LongArray(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Tag {}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tag::End => write!(f, "END"),
            Tag::Byte(v) => write!(f, "{v}b"),
            Tag::Short(v) => write!(f, "{v}s"),
            Tag::Int(v) => write!(f, "{v}"),
            Tag::Long(v) => write!(f, "{v}L"),
            Tag::Float(v) => write!(f, "{v}f"),
            Tag::Double(v) => write!(f, "{v}d"),
            Tag::ByteArray(arr) => {
                write!(f, "[B;")?;
                for (i, b) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{b}B")?;
                }
                write!(f, "]")
            }
            Tag::String(s) => write!(f, "\"{s}\""),
            Tag::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            Tag::Compound(c) => {
                write!(f, "{{")?;
                for (i, (k, v)) in c.entries.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{k}:{v}")?;
                }
                write!(f, "}}")
            }
            Tag::IntArray(arr) => {
                write!(f, "[I;")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
            Tag::LongArray(arr) => {
                write!(f, "[L;")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NumberValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
}

impl PartialEq for NumberValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (NumberValue::Byte(a), NumberValue::Byte(b)) => a == b,
            (NumberValue::Short(a), NumberValue::Short(b)) => a == b,
            (NumberValue::Int(a), NumberValue::Int(b)) => a == b,
            (NumberValue::Long(a), NumberValue::Long(b)) => a == b,
            (NumberValue::Float(a), NumberValue::Float(b)) => a.to_bits() == b.to_bits(),
            (NumberValue::Double(a), NumberValue::Double(b)) => a.to_bits() == b.to_bits(),
            _ => false,
        }
    }
}

impl NumberValue {
    pub fn double_value(&self) -> f64 {
        match self {
            NumberValue::Byte(v) => f64::from(*v),
            NumberValue::Short(v) => f64::from(*v),
            NumberValue::Int(v) => f64::from(*v),
            NumberValue::Long(v) => *v as f64,
            NumberValue::Float(v) => f64::from(*v),
            NumberValue::Double(v) => *v,
        }
    }
}

impl Tag {
    pub fn as_number(&self) -> Option<NumberValue> {
        match self {
            Tag::Byte(v) => Some(NumberValue::Byte(*v)),
            Tag::Short(v) => Some(NumberValue::Short(*v)),
            Tag::Int(v) => Some(NumberValue::Int(*v)),
            Tag::Long(v) => Some(NumberValue::Long(*v)),
            Tag::Float(v) => Some(NumberValue::Float(*v)),
            Tag::Double(v) => Some(NumberValue::Double(*v)),
            _ => None,
        }
    }

    pub fn float_value_of(v: f32) -> Self {
        if v == 0.0_f32 {
            Tag::Float(0.0_f32)
        } else {
            Tag::Float(v)
        }
    }

    pub fn double_value_of(v: f64) -> Self {
        if v == 0.0_f64 {
            Tag::Double(0.0_f64)
        } else {
            Tag::Double(v)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteBuffer {
    pub data: Vec<i8>,
}

impl ByteBuffer {
    pub fn wrap(data: Vec<i8>) -> Self {
        Self { data }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pair<A, B>(pub A, pub B);

impl<A, B> Pair<A, B> {
    pub fn of(first: A, second: B) -> Self {
        Pair(first, second)
    }

    pub fn get_first(&self) -> &A {
        &self.0
    }

    pub fn get_second(&self) -> &B {
        &self.1
    }
}

#[derive(Debug, Clone)]
pub enum DataResult<T> {
    Success(T),
    Error(String, Option<Tag>),
}

impl<T> DataResult<T> {
    pub fn success(value: T) -> Self {
        DataResult::Success(value)
    }

    pub fn error(message: String) -> Self {
        DataResult::Error(message, None)
    }

    pub fn error_with_partial(message: String, partial: Option<Tag>) -> Self {
        DataResult::Error(message, partial)
    }

    pub fn is_success(&self) -> bool {
        matches!(self, DataResult::Success(_))
    }

    pub fn ok(self) -> Option<T> {
        match self {
            DataResult::Success(v) => Some(v),
            DataResult::Error(_, _) => None,
        }
    }
}

pub trait MapLikeTag {
    fn get(&self, key: &Tag) -> Option<Tag>;
    fn get_str(&self, key: &str) -> Option<Tag>;
    fn entries(&self) -> Vec<Pair<Tag, Tag>>;
}

pub trait DynamicOpsOut<U> {
    fn empty(&self) -> U;
    fn create_byte(&self, value: i8) -> U;
    fn create_short(&self, value: i16) -> U;
    fn create_int(&self, value: i32) -> U;
    fn create_long(&self, value: i64) -> U;
    fn create_float(&self, value: f32) -> U;
    fn create_double(&self, value: f64) -> U;
    fn create_string(&self, value: String) -> U;
    fn create_byte_list(&self, data: Vec<i8>) -> U;
    fn create_int_list(&self, data: Vec<i32>) -> U;
    fn create_long_list(&self, data: Vec<i64>) -> U;
    fn create_list(&self, values: Vec<U>) -> U;
    fn create_map(&self, entries: Vec<(U, U)>) -> U;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NbtOps;

impl NbtOps {
    pub const INSTANCE: NbtOps = NbtOps;
}

impl fmt::Display for NbtOps {
    // JAVA: net.minecraft.nbt.NbtOps#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NBT")
    }
}

#[derive(Debug, Clone)]
pub struct ListConsumer {
    items: Vec<Tag>,
}

impl ListConsumer {
    pub fn accept(&self, mut consumer: impl FnMut(Tag)) {
        for item in &self.items {
            consumer(item.clone());
        }
    }
}

#[derive(Debug, Clone)]
pub struct MapEntriesConsumer {
    entries: Vec<(String, Tag)>,
}

impl MapEntriesConsumer {
    pub fn accept(&self, mut consumer: impl FnMut(Tag, Tag)) {
        let ops = NbtOps::INSTANCE;
        for (k, v) in &self.entries {
            consumer(ops.create_string(k.clone()), v.clone());
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompoundMapLike {
    pub compound: CompoundTag,
}

impl MapLikeTag for CompoundMapLike {
    fn get(&self, key: &Tag) -> Option<Tag> {
        match key {
            Tag::String(s) => self.compound.get(s).cloned(),
            _ => panic!("Cannot get map entry with non-string key: {}", key),
        }
    }

    fn get_str(&self, key: &str) -> Option<Tag> {
        self.compound.get(key).cloned()
    }

    fn entries(&self) -> Vec<Pair<Tag, Tag>> {
        let ops = NbtOps::INSTANCE;
        self.compound
            .entries
            .iter()
            .map(|(k, v)| Pair::of(ops.create_string(k.clone()), v.clone()))
            .collect()
    }
}

impl fmt::Display for CompoundMapLike {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MapLike[{}]", Tag::Compound(self.compound.clone()))
    }
}

pub struct NbtRecordBuilder {
    builder: CompoundTag,
}

impl NbtRecordBuilder {
    // JAVA: net.minecraft.nbt.NbtOps$NbtRecordBuilder#initBuilder
    fn init_builder() -> CompoundTag {
        CompoundTag::new()
    }

    pub fn new() -> Self {
        Self {
            builder: Self::init_builder(),
        }
    }

    // JAVA: net.minecraft.nbt.NbtOps$NbtRecordBuilder#append
    pub fn append(&mut self, key: String, value: Tag) -> &mut Self {
        self.builder.put(key, value);
        self
    }

    // JAVA: net.minecraft.nbt.NbtOps$NbtRecordBuilder#build
    pub fn build(&self, prefix: Tag) -> DataResult<Tag> {
        let built = self.builder.shallow_copy();
        match prefix {
            Tag::End => DataResult::success(Tag::Compound(built)),
            Tag::Compound(c) => {
                let mut merged = c.shallow_copy();
                for (k, v) in built.entries {
                    merged.put(k, v);
                }
                DataResult::success(Tag::Compound(merged))
            }
            other => DataResult::error_with_partial(
                format!("mergeToMap called with not a map: {}", other),
                Some(other),
            ),
        }
    }
}

impl Default for NbtRecordBuilder {
    fn default() -> Self {
        Self::new()
    }
}

enum ListCollector {
    Generic(Vec<Tag>),
    Byte(Vec<i8>),
    Int(Vec<i32>),
    Long(Vec<i64>),
}

impl ListCollector {
    // JAVA: net.minecraft.nbt.NbtOps$ListCollector#accept
    fn accept(self, tag: Tag) -> Self {
        match self {
            ListCollector::Generic(mut items) => {
                items.push(tag);
                ListCollector::Generic(items)
            }
            ListCollector::Byte(mut values) => match tag {
                Tag::Byte(b) => {
                    values.push(b);
                    ListCollector::Byte(values)
                }
                other => {
                    let mut items: Vec<Tag> = values.into_iter().map(Tag::Byte).collect();
                    items.push(other);
                    ListCollector::Generic(items)
                }
            },
            ListCollector::Int(mut values) => match tag {
                Tag::Int(v) => {
                    values.push(v);
                    ListCollector::Int(values)
                }
                other => {
                    let mut items: Vec<Tag> = values.into_iter().map(Tag::Int).collect();
                    items.push(other);
                    ListCollector::Generic(items)
                }
            },
            ListCollector::Long(mut values) => match tag {
                Tag::Long(v) => {
                    values.push(v);
                    ListCollector::Long(values)
                }
                other => {
                    let mut items: Vec<Tag> = values.into_iter().map(Tag::Long).collect();
                    items.push(other);
                    ListCollector::Generic(items)
                }
            },
        }
    }

    // JAVA: net.minecraft.nbt.NbtOps$ListCollector#acceptAll
    fn accept_all(mut self, tags: impl IntoIterator<Item = Tag>) -> Self {
        for t in tags {
            self = self.accept(t);
        }
        self
    }

    // JAVA: net.minecraft.nbt.NbtOps$ListCollector#result
    fn result(self) -> Tag {
        match self {
            ListCollector::Generic(items) => Tag::List(items),
            ListCollector::Byte(values) => Tag::ByteArray(values),
            ListCollector::Int(values) => Tag::IntArray(values),
            ListCollector::Long(values) => Tag::LongArray(values),
        }
    }
}

// JAVA: net.minecraft.nbt.NbtOps#createCollector
fn create_collector(tag: &Tag) -> Option<ListCollector> {
    match tag {
        Tag::End => Some(ListCollector::Generic(Vec::new())),
        Tag::List(items) => {
            if items.is_empty() {
                Some(ListCollector::Generic(Vec::new()))
            } else {
                Some(ListCollector::Generic(items.clone()))
            }
        }
        Tag::ByteArray(arr) => {
            if arr.is_empty() {
                Some(ListCollector::Generic(Vec::new()))
            } else {
                Some(ListCollector::Byte(arr.clone()))
            }
        }
        Tag::IntArray(arr) => {
            if arr.is_empty() {
                Some(ListCollector::Generic(Vec::new()))
            } else {
                Some(ListCollector::Int(arr.clone()))
            }
        }
        Tag::LongArray(arr) => {
            if arr.is_empty() {
                Some(ListCollector::Generic(Vec::new()))
            } else {
                Some(ListCollector::Long(arr.clone()))
            }
        }
        _ => None,
    }
}

fn collection_stream(tag: &Tag) -> Option<Vec<Tag>> {
    match tag {
        Tag::List(items) => Some(items.clone()),
        Tag::ByteArray(arr) => Some(arr.iter().map(|&b| Tag::Byte(b)).collect()),
        Tag::IntArray(arr) => Some(arr.iter().map(|&i| Tag::Int(i)).collect()),
        Tag::LongArray(arr) => Some(arr.iter().map(|&l| Tag::Long(l)).collect()),
        _ => None,
    }
}

fn format_tag_list(tags: &[Tag]) -> String {
    let mut s = String::from("[");
    for (i, t) in tags.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        s.push_str(&t.to_string());
    }
    s.push(']');
    s
}

impl NbtOps {
    // JAVA: net.minecraft.nbt.NbtOps#empty
    pub fn empty(&self) -> Tag {
        Tag::End
    }

    // JAVA: net.minecraft.nbt.NbtOps#emptyList
    pub fn empty_list(&self) -> Tag {
        Tag::List(Vec::new())
    }

    // JAVA: net.minecraft.nbt.NbtOps#emptyMap
    pub fn empty_map(&self) -> Tag {
        Tag::Compound(CompoundTag::new())
    }

    // JAVA: net.minecraft.nbt.NbtOps#convertTo
    pub fn convert_to<U, O: DynamicOpsOut<U>>(&self, ops: &O, tag: Tag) -> U {
        match tag {
            Tag::End => ops.empty(),
            Tag::Byte(v) => ops.create_byte(v),
            Tag::Short(v) => ops.create_short(v),
            Tag::Int(v) => ops.create_int(v),
            Tag::Long(v) => ops.create_long(v),
            Tag::Float(v) => ops.create_float(v),
            Tag::Double(v) => ops.create_double(v),
            Tag::ByteArray(v) => ops.create_byte_list(v),
            Tag::String(v) => ops.create_string(v),
            Tag::List(items) => self.convert_list(ops, items),
            Tag::Compound(c) => self.convert_map(ops, c),
            Tag::IntArray(v) => ops.create_int_list(v),
            Tag::LongArray(v) => ops.create_long_list(v),
        }
    }

    fn convert_list<U, O: DynamicOpsOut<U>>(&self, ops: &O, items: Vec<Tag>) -> U {
        let converted = items.into_iter().map(|e| self.convert_to(ops, e)).collect();
        ops.create_list(converted)
    }

    fn convert_map<U, O: DynamicOpsOut<U>>(&self, ops: &O, compound: CompoundTag) -> U {
        let converted = compound
            .entries
            .into_iter()
            .map(|(k, v)| (ops.create_string(k), self.convert_to(ops, v)))
            .collect();
        ops.create_map(converted)
    }

    // JAVA: net.minecraft.nbt.NbtOps#getNumberValue
    pub fn get_number_value(&self, tag: &Tag) -> DataResult<NumberValue> {
        match tag.as_number() {
            Some(n) => DataResult::success(n),
            None => DataResult::error("Not a number".to_string()),
        }
    }

    // JAVA: net.minecraft.nbt.NbtOps#createNumeric
    pub fn create_numeric(&self, value: NumberValue) -> Tag {
        Tag::double_value_of(value.double_value())
    }

    // JAVA: net.minecraft.nbt.NbtOps#createByte
    pub fn create_byte(&self, value: i8) -> Tag {
        Tag::Byte(value)
    }

    // JAVA: net.minecraft.nbt.NbtOps#createShort
    pub fn create_short(&self, value: i16) -> Tag {
        Tag::Short(value)
    }

    // JAVA: net.minecraft.nbt.NbtOps#createInt
    pub fn create_int(&self, value: i32) -> Tag {
        Tag::Int(value)
    }

    // JAVA: net.minecraft.nbt.NbtOps#createLong
    pub fn create_long(&self, value: i64) -> Tag {
        Tag::Long(value)
    }

    // JAVA: net.minecraft.nbt.NbtOps#createFloat
    pub fn create_float(&self, value: f32) -> Tag {
        Tag::float_value_of(value)
    }

    // JAVA: net.minecraft.nbt.NbtOps#createDouble
    pub fn create_double(&self, value: f64) -> Tag {
        Tag::double_value_of(value)
    }

    // JAVA: net.minecraft.nbt.NbtOps#createBoolean
    pub fn create_boolean(&self, value: bool) -> Tag {
        Tag::Byte(if value { 1_i8 } else { 0_i8 })
    }

    // JAVA: net.minecraft.nbt.NbtOps#getStringValue
    pub fn get_string_value(&self, tag: &Tag) -> DataResult<String> {
        match tag {
            Tag::String(s) => DataResult::success(s.clone()),
            _ => DataResult::error("Not a string".to_string()),
        }
    }

    // JAVA: net.minecraft.nbt.NbtOps#createString
    pub fn create_string(&self, value: String) -> Tag {
        Tag::String(value)
    }

    // JAVA: net.minecraft.nbt.NbtOps#mergeToList
    pub fn merge_to_list(&self, list: Tag, value: Tag) -> DataResult<Tag> {
        match create_collector(&list) {
            Some(c) => DataResult::success(c.accept(value).result()),
            None => DataResult::error_with_partial(
                format!("mergeToList called with not a list: {}", list),
                Some(list),
            ),
        }
    }

    // JAVA: net.minecraft.nbt.NbtOps#mergeToList
    pub fn merge_to_list_many(&self, list: Tag, values: Vec<Tag>) -> DataResult<Tag> {
        match create_collector(&list) {
            Some(c) => DataResult::success(c.accept_all(values).result()),
            None => DataResult::error_with_partial(
                format!("mergeToList called with not a list: {}", list),
                Some(list),
            ),
        }
    }

    // JAVA: net.minecraft.nbt.NbtOps#mergeToMap
    pub fn merge_to_map(&self, map: Tag, key: Tag, value: Tag) -> DataResult<Tag> {
        if !matches!(map, Tag::Compound(_) | Tag::End) {
            return DataResult::error_with_partial(
                format!("mergeToMap called with not a map: {}", map),
                Some(map),
            );
        }
        match key {
            Tag::String(k) => {
                let mut compound = match &map {
                    Tag::Compound(c) => c.shallow_copy(),
                    _ => CompoundTag::new(),
                };
                compound.put(k, value);
                DataResult::success(Tag::Compound(compound))
            }
            other => DataResult::error_with_partial(
                format!("key is not a string: {}", other),
                Some(map),
            ),
        }
    }

    // JAVA: net.minecraft.nbt.NbtOps#mergeToMap
    pub fn merge_to_map_like(&self, map: Tag, map_like: &dyn MapLikeTag) -> DataResult<Tag> {
        if !matches!(map, Tag::Compound(_) | Tag::End) {
            return DataResult::error_with_partial(
                format!("mergeToMap called with not a map: {}", map),
                Some(map),
            );
        }
        let entries = map_like.entries();
        if entries.is_empty() {
            return if map == Tag::End {
                DataResult::success(self.empty_map())
            } else {
                DataResult::success(map)
            };
        }
        let mut compound = match &map {
            Tag::Compound(c) => c.shallow_copy(),
            _ => CompoundTag::new(),
        };
        let mut bad_keys: Vec<Tag> = Vec::new();
        for pair in entries {
            match pair.0 {
                Tag::String(s) => compound.put(s, pair.1),
                other => bad_keys.push(other),
            }
        }
        if !bad_keys.is_empty() {
            DataResult::error_with_partial(
                format!("some keys are not strings: {}", format_tag_list(&bad_keys)),
                Some(Tag::Compound(compound)),
            )
        } else {
            DataResult::success(Tag::Compound(compound))
        }
    }

    // JAVA: net.minecraft.nbt.NbtOps#mergeToMap
    pub fn merge_to_map_pairs(&self, map: Tag, values: Vec<Pair<Tag, Tag>>) -> DataResult<Tag> {
        if !matches!(map, Tag::Compound(_) | Tag::End) {
            return DataResult::error_with_partial(
                format!("mergeToMap called with not a map: {}", map),
                Some(map),
            );
        }
        if values.is_empty() {
            return if map == Tag::End {
                DataResult::success(self.empty_map())
            } else {
                DataResult::success(map)
            };
        }
        let mut compound = match &map {
            Tag::Compound(c) => c.shallow_copy(),
            _ => CompoundTag::new(),
        };
        let mut bad_keys: Vec<Tag> = Vec::new();
        for pair in values {
            match pair.0 {
                Tag::String(s) => compound.put(s, pair.1),
                other => bad_keys.push(other),
            }
        }
        if !bad_keys.is_empty() {
            DataResult::error_with_partial(
                format!("some keys are not strings: {}", format_tag_list(&bad_keys)),
                Some(Tag::Compound(compound)),
            )
        } else {
            DataResult::success(Tag::Compound(compound))
        }
    }

    // JAVA: net.minecraft.nbt.NbtOps#getMapValues
    pub fn get_map_values(&self, tag: &Tag) -> DataResult<Vec<Pair<Tag, Tag>>> {
        match tag {
            Tag::Compound(c) => {
                let pairs = c
                    .entries
                    .iter()
                    .map(|(k, v)| Pair::of(self.create_string(k.clone()), v.clone()))
                    .collect();
                DataResult::success(pairs)
            }
            _ => DataResult::error(format!("Not a map: {}", tag)),
        }
    }

    // JAVA: net.minecraft.nbt.NbtOps#getMapEntries
    pub fn get_map_entries(&self, tag: &Tag) -> DataResult<MapEntriesConsumer> {
        match tag {
            Tag::Compound(c) => DataResult::success(MapEntriesConsumer {
                entries: c.entries.clone(),
            }),
            _ => DataResult::error(format!("Not a map: {}", tag)),
        }
    }

    // JAVA: net.minecraft.nbt.NbtOps#getMap
    pub fn get_map(&self, tag: &Tag) -> DataResult<CompoundMapLike> {
        match tag {
            Tag::Compound(c) => DataResult::success(CompoundMapLike {
                compound: c.clone(),
            }),
            _ => DataResult::error(format!("Not a map: {}", tag)),
        }
    }

    // JAVA: net.minecraft.nbt.NbtOps#createMap
    pub fn create_map(&self, entries: Vec<Pair<Tag, Tag>>) -> Tag {
        let mut compound = CompoundTag::new();
        for pair in entries {
            match pair.0 {
                Tag::String(s) => compound.put(s, pair.1),
                other => panic!("Cannot create map with non-string key: {}", other),
            }
        }
        Tag::Compound(compound)
    }

    // JAVA: net.minecraft.nbt.NbtOps#getStream
    pub fn get_stream(&self, tag: &Tag) -> DataResult<Vec<Tag>> {
        match collection_stream(tag) {
            Some(stream) => DataResult::success(stream),
            None => DataResult::error("Not a list".to_string()),
        }
    }

    // JAVA: net.minecraft.nbt.NbtOps#getList
    pub fn get_list(&self, tag: &Tag) -> DataResult<ListConsumer> {
        match collection_stream(tag) {
            Some(items) => DataResult::success(ListConsumer { items }),
            None => DataResult::error(format!("Not a list: {}", tag)),
        }
    }

    // JAVA: net.minecraft.nbt.NbtOps#getByteBuffer
    pub fn get_byte_buffer(&self, tag: &Tag) -> DataResult<ByteBuffer> {
        match tag {
            Tag::ByteArray(arr) => DataResult::success(ByteBuffer::wrap(arr.clone())),
            _ => match self.get_stream(tag) {
                DataResult::Success(stream) => {
                    let mut bytes = Vec::with_capacity(stream.len());
                    for t in stream {
                        match t {
                            Tag::Byte(b) => bytes.push(b),
                            _ => {
                                return DataResult::error(
                                    "Input is not a list of bytes".to_string(),
                                )
                            }
                        }
                    }
                    DataResult::success(ByteBuffer::wrap(bytes))
                }
                DataResult::Error(msg, partial) => DataResult::Error(msg, partial),
            },
        }
    }

    // JAVA: net.minecraft.nbt.NbtOps#createByteList
    pub fn create_byte_list(&self, buf: &ByteBuffer) -> Tag {
        Tag::ByteArray(buf.data.clone())
    }

    // JAVA: net.minecraft.nbt.NbtOps#getIntStream
    pub fn get_int_stream(&self, tag: &Tag) -> DataResult<Vec<i32>> {
        match tag {
            Tag::IntArray(arr) => DataResult::success(arr.clone()),
            _ => match self.get_stream(tag) {
                DataResult::Success(stream) => {
                    let mut ints = Vec::with_capacity(stream.len());
                    for t in stream {
                        match t {
                            Tag::Int(v) => ints.push(v),
                            _ => {
                                return DataResult::error(
                                    "Input is not a list of ints".to_string(),
                                )
                            }
                        }
                    }
                    DataResult::success(ints)
                }
                DataResult::Error(msg, partial) => DataResult::Error(msg, partial),
            },
        }
    }

    // JAVA: net.minecraft.nbt.NbtOps#createIntList
    pub fn create_int_list(&self, values: Vec<i32>) -> Tag {
        Tag::IntArray(values)
    }

    // JAVA: net.minecraft.nbt.NbtOps#getLongStream
    pub fn get_long_stream(&self, tag: &Tag) -> DataResult<Vec<i64>> {
        match tag {
            Tag::LongArray(arr) => DataResult::success(arr.clone()),
            _ => match self.get_stream(tag) {
                DataResult::Success(stream) => {
                    let mut longs = Vec::with_capacity(stream.len());
                    for t in stream {
                        match t {
                            Tag::Long(v) => longs.push(v),
                            _ => {
                                return DataResult::error(
                                    "Input is not a list of longs".to_string(),
                                )
                            }
                        }
                    }
                    DataResult::success(longs)
                }
                DataResult::Error(msg, partial) => DataResult::Error(msg, partial),
            },
        }
    }

    // JAVA: net.minecraft.nbt.NbtOps#createLongList
    pub fn create_long_list(&self, values: Vec<i64>) -> Tag {
        Tag::LongArray(values)
    }

    // JAVA: net.minecraft.nbt.NbtOps#createList
    pub fn create_list(&self, values: Vec<Tag>) -> Tag {
        Tag::List(values)
    }

    // JAVA: net.minecraft.nbt.NbtOps#remove
    pub fn remove(&self, tag: Tag, key: &str) -> Tag {
        match tag {
            Tag::Compound(c) => {
                let mut copy = c.shallow_copy();
                copy.remove(key);
                Tag::Compound(copy)
            }
            other => other,
        }
    }

    // JAVA: net.minecraft.nbt.NbtOps#mapBuilder
    pub fn map_builder(&self) -> NbtRecordBuilder {
        NbtRecordBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_get_primitives() {
        let ops = NbtOps::INSTANCE;
        assert_eq!(ops.create_byte(-128_i8), Tag::Byte(-128_i8));
        assert_eq!(ops.create_short(-1_i16), Tag::Short(-1_i16));
        assert_eq!(ops.create_int(i32::MIN), Tag::Int(i32::MIN));
        assert_eq!(ops.create_long(i64::MIN), Tag::Long(i64::MIN));
        assert_eq!(ops.create_boolean(true), Tag::Byte(1));
        assert_eq!(ops.create_boolean(false), Tag::Byte(0));
        assert_eq!(
            ops.create_string("hello".to_string()),
            Tag::String("hello".to_string())
        );

        match ops.get_number_value(&Tag::Int(42)) {
            DataResult::Success(NumberValue::Int(v)) => assert_eq!(v, 42),
            other => panic!("expected int, got {:?}", other),
        }
        assert!(matches!(
            ops.get_string_value(&Tag::Int(1)),
            DataResult::Error(_, _)
        ));
        match ops.get_string_value(&Tag::String("ok".to_string())) {
            DataResult::Success(s) => assert_eq!(s, "ok"),
            other => panic!("expected string, got {:?}", other),
        }
    }

    #[test]
    fn test_float_bits_and_zero_normalization() {
        let ops = NbtOps::INSTANCE;
        let neg_zero_f = ops.create_float(-0.0_f32);
        let pos_zero_f = ops.create_float(0.0_f32);
        assert_eq!(neg_zero_f, pos_zero_f);
        assert_eq!(neg_zero_f, Tag::Float(0.0_f32));

        let neg_zero_d = ops.create_double(-0.0_f64);
        let pos_zero_d = ops.create_double(0.0_f64);
        assert_eq!(neg_zero_d, pos_zero_d);

        let nan_f = ops.create_float(f32::NAN);
        match nan_f {
            Tag::Float(v) => assert!(v.is_nan()),
            _ => panic!("expected float"),
        }
        let nan_f2 = ops.create_float(f32::from_bits(f32::NAN.to_bits() ^ 1));
        assert_ne!(nan_f, nan_f2);

        let inf_d = ops.create_double(f64::NEG_INFINITY);
        assert_eq!(inf_d, Tag::Double(f64::NEG_INFINITY));
    }

    #[test]
    fn test_merge_to_list_collectors() {
        let ops = NbtOps::INSTANCE;

        let r = ops.merge_to_list(Tag::End, Tag::Byte(1));
        let list = match r {
            DataResult::Success(t) => t,
            _ => panic!("expected success"),
        };
        assert_eq!(list, Tag::List(vec![Tag::Byte(1)]));

        let r = ops.merge_to_list(Tag::ByteArray(vec![]), Tag::Byte(2));
        let list = match r {
            DataResult::Success(t) => t,
            _ => panic!("expected success"),
        };
        assert_eq!(list, Tag::List(vec![Tag::Byte(2)]));

        let r = ops.merge_to_list(Tag::ByteArray(vec![1_i8, -2_i8]), Tag::Byte(3));
        let arr = match r {
            DataResult::Success(t) => t,
            _ => panic!("expected success"),
        };
        assert_eq!(arr, Tag::ByteArray(vec![1_i8, -2_i8, 3_i8]));

        let r = ops.merge_to_list(Tag::ByteArray(vec![1_i8]), Tag::Int(5));
        let list = match r {
            DataResult::Success(t) => t,
            _ => panic!("expected success"),
        };
        assert_eq!(list, Tag::List(vec![Tag::Byte(1_i8), Tag::Int(5)]));

        let r =
            ops.merge_to_list_many(Tag::IntArray(vec![10]), vec![Tag::Int(20), Tag::Int(-30)]);
        let arr = match r {
            DataResult::Success(t) => t,
            _ => panic!("expected success"),
        };
        assert_eq!(arr, Tag::IntArray(vec![10, 20, -30]));

        let r = ops.merge_to_list(Tag::Compound(CompoundTag::new()), Tag::Byte(1));
        assert!(matches!(r, DataResult::Error(_, Some(Tag::Compound(_)))));
    }

    #[test]
    fn test_merge_to_map_and_errors() {
        let ops = NbtOps::INSTANCE;

        let r = ops.merge_to_map(Tag::End, Tag::String("a".into()), Tag::Int(1));
        let m = match r {
            DataResult::Success(t) => t,
            _ => panic!("expected success"),
        };
        let mut expected = CompoundTag::new();
        expected.put("a".into(), Tag::Int(1));
        assert_eq!(m, Tag::Compound(expected));

        let base = Tag::Compound(CompoundTag::new());
        let r = ops.merge_to_map(base.clone(), Tag::Int(1), Tag::Byte(0));
        match r {
            DataResult::Error(msg, partial) => {
                assert!(msg.contains("key is not a string"));
                assert_eq!(partial, Some(base));
            }
            _ => panic!("expected error"),
        }

        let r = ops.merge_to_map(Tag::Int(0), Tag::String("k".into()), Tag::Int(1));
        assert!(matches!(r, DataResult::Error(_, Some(Tag::Int(0)))));

        let r = ops.merge_to_map_pairs(
            Tag::End,
            vec![
                Pair::of(Tag::String("ok".into()), Tag::Byte(1)),
                Pair::of(Tag::Int(2), Tag::Byte(2)),
            ],
        );
        match r {
            DataResult::Error(msg, partial) => {
                assert!(msg.contains("some keys are not strings"));
                match partial {
                    Some(Tag::Compound(c)) => assert_eq!(c.get("ok"), Some(&Tag::Byte(1))),
                    _ => panic!("expected compound partial"),
                }
            }
            _ => panic!("expected error"),
        }
    }

    #[test]
    fn test_get_remove_buffers_and_builder() {
        let ops = NbtOps::INSTANCE;

        let mut c = CompoundTag::new();
        c.put("x".into(), Tag::Int(7));
        c.put("y".into(), Tag::String("hi".into()));
        let tag = Tag::Compound(c);

        let vals = match ops.get_map_values(&tag) {
            DataResult::Success(v) => v,
            _ => panic!("expected success"),
        };
        assert_eq!(vals.len(), 2);
        assert_eq!(vals[0].0, Tag::String("x".into()));

        let removed = ops.remove(tag.clone(), "x");
        match &removed {
            Tag::Compound(c) => {
                assert!(c.get("x").is_none());
                assert_eq!(c.get("y"), Some(&Tag::String("hi".into())));
            }
            _ => panic!("expected compound"),
        }
        assert_eq!(ops.remove(Tag::Int(5), "x"), Tag::Int(5));

        let stream = match ops.get_stream(&Tag::ByteArray(vec![1_i8, -1_i8])) {
            DataResult::Success(s) => s,
            _ => panic!("expected success"),
        };
        assert_eq!(stream, vec![Tag::Byte(1), Tag::Byte(-1)]);

        let buf = ByteBuffer::wrap(vec![10_i8, -20_i8]);
        let arr_tag = ops.create_byte_list(&buf);
        assert_eq!(arr_tag, Tag::ByteArray(vec![10_i8, -20_i8]));
        let got = match ops.get_byte_buffer(&arr_tag) {
            DataResult::Success(b) => b,
            _ => panic!("expected success"),
        };
        assert_eq!(got.data, vec![10_i8, -20_i8]);

        let mut builder = ops.map_builder();
        builder.append("k".into(), Tag::Int(1));
        let built = match builder.build(Tag::End) {
            DataResult::Success(t) => t,
            _ => panic!("expected success"),
        };
        let mut expected = CompoundTag::new();
        expected.put("k".into(), Tag::Int(1));
        assert_eq!(built, Tag::Compound(expected));
    }
}
